//! A test-only text form of a Store: `pprint_store` renders it and
//! `parse_store` reads it back.
//!
//! ```text
//! "chores-3f2a1c" open 2026-01-01T00:00:02Z
//!   2026-01-01T00:00:03Z TodoCreated {node-a:1}
//!     data {"todo_id":1,"title":"Take out the trash"}
//!     metadata {}
//!   2026-01-01T00:00:04Z TodoFinished {node-a:2}
//!     data {"todo_id":1}
//!     metadata {"schema_version":"1.7.2"}
//! ```
//!
//! `metadata {}` means no metadata, and a hand-written text may leave the line
//! out. Payloads are one line of text each.

use super::{NODE_ID, START, STEP, next_event_id};
use distolocal::{Clock, Event, PendingEvent, Store, Stream, StreamStatus};
use std::cell::Cell;
use std::collections::HashMap;
use std::iter;
use std::str;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

const NO_METADATA: &str = "{}";

/// A clock that reads out `script` in order, then carries on stepping `step`
/// from where the script left off.
struct ScriptedClock {
    script: Vec<OffsetDateTime>,
    readings: Cell<usize>,
    next: Cell<OffsetDateTime>,
    step: Duration,
}

impl ScriptedClock {
    fn new(script: Vec<OffsetDateTime>, step: Duration) -> Self {
        ScriptedClock {
            script,
            readings: Cell::new(0),
            next: Cell::new(START),
            step,
        }
    }
}

impl Clock for ScriptedClock {
    fn now(&self) -> OffsetDateTime {
        let readings = self.readings.get();
        let now = self
            .script
            .get(readings)
            .copied()
            .unwrap_or_else(|| self.next.get());

        self.readings.set(readings + 1);
        self.next.set(now + self.step);
        now
    }
}

/// Pretty print a Store. Print every Stream for a Store and every Event in a Stream.
pub fn pprint_store(store: &Store) -> String {
    let mut streams: Vec<&Stream> = store
        .stream_ids()
        .into_iter()
        .map(|stream_id| {
            store
                .get_stream(stream_id)
                .expect("the Store holds the Stream")
        })
        .collect();
    // The Store lists ids in HashMap order; sort by creation, then id.
    streams.sort_by_key(|stream| {
        (
            parse_time(&stream.created_at.to_string()),
            stream.stream_id.clone(),
        )
    });

    streams
        .iter()
        .map(|stream| stream_block(store, stream))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A Store holding the Streams the text describes, replayed through
/// `create_stream`/`append_event`. The times in the text drive the Store's
/// clock, which keeps stepping after the text ends.
pub fn parse_store(text: &str) -> Store {
    let streams = parse(text);
    let script = streams
        .iter()
        .flat_map(|stream| {
            iter::once(stream.created_at).chain(stream.events.iter().map(|event| event.created_at))
        })
        .collect();

    let mut store = Store::new(NODE_ID, ScriptedClock::new(script, STEP));
    for stream in &streams {
        replay(&mut store, stream);
    }

    store
}

fn stream_block(store: &Store, stream: &Stream) -> String {
    let events = store
        .read_stream(&stream.stream_id)
        .expect("the Store holds the Stream");

    let mut lines = vec![stream_line(stream)];
    for event in events {
        lines.push(event_line(event));
        lines.push(payload_line("data", &event.data));
        let metadata = if event.metadata.is_empty() {
            NO_METADATA.as_bytes()
        } else {
            &event.metadata
        };
        lines.push(payload_line("metadata", metadata));
    }

    lines.join("\n")
}

fn stream_line(stream: &Stream) -> String {
    format!(
        "\"{}\" {} {}",
        stream.stream_id,
        status(&stream.status),
        stream.created_at
    )
}

fn status(status: &StreamStatus) -> &'static str {
    match status {
        StreamStatus::Open => "open",
        StreamStatus::Closed => "closed",
    }
}

fn event_line(event: &Event) -> String {
    format!(
        "  {} {} {}",
        event.created_at,
        event.event_type,
        vector_clock(&event.vector_clock)
    )
}

fn vector_clock(vector_clock: &HashMap<String, u64>) -> String {
    let mut counts: Vec<_> = vector_clock
        .iter()
        .map(|(node_id, count)| format!("{node_id}:{count}"))
        .collect();
    counts.sort();

    format!("{{{}}}", counts.join(","))
}

fn payload_line(field: &str, payload: &[u8]) -> String {
    let payload = str::from_utf8(payload).expect("a payload the notation can render is text");

    format!("    {field} {payload}")
}

fn replay(store: &mut Store, stream: &ParsedStream) {
    // `create_stream` returns an existing Stream without reading the clock,
    // which would leave that block's time in the script for the next reading.
    assert!(
        store.get_stream(&stream.stream_id).is_err(),
        "parse_store: the Stream {:?} appears twice",
        stream.stream_id
    );
    store
        .create_stream(&stream.stream_id)
        .expect("creating a new Stream should succeed");

    for event in &stream.events {
        let appended = store
            .append_event(
                &stream.stream_id,
                PendingEvent {
                    event_id: next_event_id(),
                    event_type: event.event_type.clone(),
                    data: event.data.clone(),
                    metadata: event.metadata.clone(),
                },
            )
            .expect("appending to an open Stream should succeed");

        assert_eq!(
            appended.vector_clock, event.vector_clock,
            "parse_store: the Store assigned {} a different vector clock than the text",
            event.event_type
        );
    }

    if stream.status == StreamStatus::Closed {
        panic!("parse_store: describing a closed Stream needs Close Stream");
    }
}

struct ParsedStream {
    stream_id: String,
    status: StreamStatus,
    created_at: OffsetDateTime,
    events: Vec<ParsedEvent>,
}

struct ParsedEvent {
    created_at: OffsetDateTime,
    event_type: String,
    vector_clock: HashMap<String, u64>,
    data: Vec<u8>,
    metadata: Vec<u8>,
}

fn parse(text: &str) -> Vec<ParsedStream> {
    let mut streams: Vec<ParsedStream> = Vec::new();

    // Each line says what it is by its shape, so indentation is ignored.
    for line in text.lines().map(str::trim_start) {
        if line.is_empty() {
            continue;
        }

        if line.starts_with('"') {
            streams.push(parse_stream(line));
        } else if let Some(data) = line.strip_prefix("data ") {
            last_event(&mut streams, line).data = data.as_bytes().to_vec();
        } else if let Some(metadata) = line.strip_prefix("metadata ") {
            last_event(&mut streams, line).metadata = match metadata {
                NO_METADATA => Vec::new(),
                metadata => metadata.as_bytes().to_vec(),
            };
        } else {
            streams
                .last_mut()
                .unwrap_or_else(|| invalid(line))
                .events
                .push(parse_event(line));
        }
    }

    streams
}

fn last_event<'a>(streams: &'a mut [ParsedStream], line: &str) -> &'a mut ParsedEvent {
    streams
        .last_mut()
        .and_then(|stream| stream.events.last_mut())
        .unwrap_or_else(|| invalid(line))
}

fn parse_stream(line: &str) -> ParsedStream {
    let (stream_id, rest) = line
        .strip_prefix('"')
        .and_then(|rest| rest.split_once('"'))
        .unwrap_or_else(|| invalid(line));

    let mut fields = rest.split_whitespace();
    let status = match fields.next() {
        Some("open") => StreamStatus::Open,
        Some("closed") => StreamStatus::Closed,
        _ => invalid(line),
    };
    let created_at = fields.next().unwrap_or_else(|| invalid(line));

    ParsedStream {
        stream_id: stream_id.to_string(),
        status,
        created_at: parse_time(created_at),
        events: Vec::new(),
    }
}

fn parse_event(line: &str) -> ParsedEvent {
    let fields: Vec<_> = line.split_whitespace().collect();
    let [created_at, event_type, vector_clock] = fields[..] else {
        invalid(line)
    };

    ParsedEvent {
        created_at: parse_time(created_at),
        event_type: event_type.to_string(),
        vector_clock: parse_vector_clock(vector_clock),
        data: Vec::new(),
        metadata: Vec::new(),
    }
}

fn parse_vector_clock(field: &str) -> HashMap<String, u64> {
    let counts = field
        .strip_prefix('{')
        .and_then(|field| field.strip_suffix('}'))
        .unwrap_or_else(|| invalid(field));

    counts
        .split(',')
        .map(|entry| {
            let (node_id, count) = entry.split_once(':').unwrap_or_else(|| invalid(entry));
            let count = count.parse().unwrap_or_else(|_| invalid(entry));

            (node_id.to_string(), count)
        })
        .collect()
}

fn parse_time(field: &str) -> OffsetDateTime {
    OffsetDateTime::parse(field, &Rfc3339).unwrap_or_else(|_| invalid(field))
}

fn invalid(text: &str) -> ! {
    panic!("parse_store: can't read {text:?}")
}
