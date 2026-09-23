// The test-only notation that both a `Given` and a `Then` are written in —
// see Plan.md step 7 and DECISIONS.md 0011:
//
//     "chores-3f2a1c" open 2026-01-01T00:00:02Z
//       2026-01-01T00:00:03Z TodoCreated {node-a:1}
//         data {"todo_id":1,"title":"Take out the trash"}
//         metadata {}
//       2026-01-01T00:00:04Z TodoFinished {node-a:2}
//         data {"todo_id":1}
//         metadata {"schema_version":"1.7.2"}
//
// Every Event renders the same three lines, `metadata {}` when it carries
// none — so `{}` is how the notation writes no metadata, and an Event whose
// metadata is literally `{}` reads the same as one with none. They mean the
// same thing to an Application. A hand-written `Given` may leave the line
// out; a dump always writes it.
//
// `dump` renders everything a Store holds in it — a block per Stream, which
// is why headers sit at column 0; `given` parses the same text back and
// replays it through the Store's public API. It is not a serialization,
// import or export format (DECISIONS.md 0011): a payload is rendered as the
// line of text it is in these tests, so one that isn't UTF-8, or that holds
// a newline, has no notation.

use super::{NODE_ID, STEP, ScriptedClock, next_event_id};
use distolocal::{Event, PendingEvent, Store, Stream, StreamStatus};
use std::collections::HashMap;
use std::iter;
use std::str;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

// How far `dump` indents an Event under its Stream's header line, and a
// payload under its Event. A header stays at column 0, so dumped Streams
// concatenate.
const EVENT_INDENT: usize = 2;
const PAYLOAD_INDENT: usize = EVENT_INDENT * 2;

// An Event that carries no metadata still renders its line, holding this.
const NO_METADATA: &str = "{}";

// Both halves of a Stream are looked up by an id the Store itself listed,
// so neither lookup can miss.
const HELD: &str = "dumping a Stream the Store holds";

/// Pretty print a Store. Print every Stream for a Store and every Event in a Stream.
pub fn pprint_store(store: &Store) -> String {
    let mut streams: Vec<&Stream> = store
        .stream_ids()
        .into_iter()
        .map(|stream_id| store.get_stream(stream_id).expect(HELD))
        .collect();
    // The Store hands its ids back in its own order, so the dump puts the
    // Streams back in the order they were created — the order a `Given`
    // describes them in, so a before and an after line up block for block.
    // Two Streams share a `created_at` only on a clock that doesn't step,
    // and there the id settles it: either way the dump is stable.
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

/// One Stream: its header line, then its Events. An empty Stream is its
/// header alone, and headers sit at column 0, so the blocks concatenate.
fn stream_block(store: &Store, stream: &Stream) -> String {
    let events = store.read_stream(&stream.stream_id).expect(HELD);

    let mut lines = vec![stream_line(stream)];
    for event in events {
        lines.push(event_line(event));
        // `data` always renders — an Event without it is `InvalidEvent`
        // (Plan.md step 8) — and so does `metadata`, as `{}` when the Event
        // carries none, so every Event is the same three lines.
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

/// A Store holding the Streams the text describes, replayed through
/// `create_stream`/`append_event` — so a `Given` can only describe a state
/// the Store would really produce (DECISIONS.md 0011). The times the text
/// names drive the Store's clock (DECISIONS.md 0015), which carries on
/// stepping from the last of them once the text runs out.
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

fn stream_line(stream: &Stream) -> String {
    // Quoted, because the Application chooses a Stream's id and nothing
    // stops one holding a space.
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

// The time leads, because it is always the same width: the Event types
// line up in a column under each other.
fn event_line(event: &Event) -> String {
    format!(
        "{}{} {} {}",
        " ".repeat(EVENT_INDENT),
        event.created_at,
        event.event_type,
        vector_clock(&event.vector_clock)
    )
}

// Sorted by node id, so `HashMap` iteration order can't reorder a dump.
fn vector_clock(vector_clock: &HashMap<String, u64>) -> String {
    let mut counts: Vec<_> = vector_clock.iter().collect();
    counts.sort();
    let counts: Vec<_> = counts
        .iter()
        .map(|(node_id, count)| format!("{node_id}:{count}"))
        .collect();

    format!("{{{}}}", counts.join(","))
}

// A payload takes the rest of its line, so nothing inside it can collide
// with a field after it.
fn payload_line(field: &str, payload: &[u8]) -> String {
    let payload = str::from_utf8(payload).expect("a payload the notation can render is text");

    format!("{}{} {}", " ".repeat(PAYLOAD_INDENT), field, payload)
}

fn replay(store: &mut Store, stream: &ParsedStream) {
    store
        .create_stream(&stream.stream_id)
        .expect("creating a new Stream should succeed");

    for event in &stream.events {
        let appended = store
            .append_event(
                &stream.stream_id,
                PendingEvent {
                    // Out of the notation, so made up here: an `event_id` is
                    // the Application's idempotency key, not Stream content
                    // (Plan.md step 7a).
                    event_id: next_event_id(),
                    event_type: event.event_type.clone(),
                    data: event.data.clone(),
                    metadata: event.metadata.clone(),
                },
            )
            .expect("appending to an open Stream should succeed");

        // Why a `Given` is replayed rather than built directly: one that
        // claims a vector clock the Store wouldn't assign is describing a
        // state that cannot happen.
        assert_eq!(
            appended.vector_clock, event.vector_clock,
            "given: the Store assigned {} a vector clock other than the one the text claims",
            event.event_type
        );
    }

    if stream.status == StreamStatus::Closed {
        // Close Stream lands at Plan.md step 9. Until then, replaying a
        // closed Stream as an open one would quietly make the test lie.
        panic!("given: describing a closed Stream needs Close Stream (Plan.md step 9)");
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

    // Every line says which of the three it is by its own shape — a quoted
    // id, a `data`/`metadata` field, or a time — so the indentation is the
    // reader's and not the parser's. A `Given` can sit at whatever depth the
    // code around it does, and a `Then`'s dump goes straight back in.
    for line in text.lines().map(str::trim_start) {
        if line.is_empty() {
            continue;
        }

        if line.starts_with('"') {
            let stream = parse_stream(line);
            assert!(
                !streams
                    .iter()
                    .any(|parsed| parsed.stream_id == stream.stream_id),
                "given: the Stream {:?} appears twice; one block is a Stream's whole state",
                stream.stream_id
            );
            streams.push(stream);
        } else if let Some(data) = line.strip_prefix("data ") {
            last_event(&mut streams, line).data = data.as_bytes().to_vec();
        } else if let Some(metadata) = line.strip_prefix("metadata ") {
            // `{}` stands for no metadata, so it comes back as the nothing
            // it stands for — otherwise replaying a dump would hand back a
            // Store carrying two bytes the dumped one didn't.
            last_event(&mut streams, line).metadata = match metadata {
                NO_METADATA => Vec::new(),
                metadata => metadata.as_bytes().to_vec(),
            };
        } else {
            last_stream(&mut streams, line)
                .events
                .push(parse_event(line));
        }
    }

    streams
}

// An Event line belongs to the Stream above it, a payload line to the Event
// above that — the nesting a dump renders as indentation.
fn last_stream<'a>(streams: &'a mut [ParsedStream], line: &str) -> &'a mut ParsedStream {
    streams
        .last_mut()
        .unwrap_or_else(|| panic!("given: an Event line goes under a Stream: {line:?}"))
}

fn last_event<'a>(streams: &'a mut [ParsedStream], line: &str) -> &'a mut ParsedEvent {
    streams
        .last_mut()
        .and_then(|stream| stream.events.last_mut())
        .unwrap_or_else(|| panic!("given: a payload line goes under an Event: {line:?}"))
}

fn parse_stream(line: &str) -> ParsedStream {
    let (stream_id, rest) = line
        .strip_prefix('"')
        .and_then(|rest| rest.split_once('"'))
        .unwrap_or_else(|| panic!("given: a Stream line starts with a quoted id: {line:?}"));

    let mut fields = rest.split_whitespace();
    let status = match fields.next() {
        Some("open") => StreamStatus::Open,
        Some("closed") => StreamStatus::Closed,
        _ => panic!("given: a Stream is open or closed: {line:?}"),
    };
    let created_at = fields
        .next()
        .unwrap_or_else(|| panic!("given: a Stream line ends with its created_at: {line:?}"));

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
        panic!("given: an Event line is a created_at, a type and a vector clock: {line:?}")
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
        .unwrap_or_else(|| panic!("given: a vector clock is written {{node-a:1}}: {field:?}"));

    counts
        .split(',')
        .map(|entry| {
            let (node_id, count) = entry.split_once(':').unwrap_or_else(|| {
                panic!("given: a vector clock entry is a node id and a count: {entry:?}")
            });
            let count = count.parse().unwrap_or_else(|_| {
                panic!("given: a vector clock counts in whole numbers: {entry:?}")
            });

            (node_id.to_string(), count)
        })
        .collect()
}

fn parse_time(field: &str) -> OffsetDateTime {
    OffsetDateTime::parse(field, &Rfc3339).unwrap_or_else(|_| {
        panic!("given: a time is RFC 3339, like 2026-01-01T00:00:00Z: {field:?}")
    })
}
