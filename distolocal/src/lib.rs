use std::collections::HashMap;
use std::fmt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

// The source of the times the Store assigns — `created_at`, on both a
// Stream and an Event. A dependency rather than a direct call to
// `OffsetDateTime::now_utc()` so tests can hand the Store a clock that reads
// a known instant; see DECISIONS.md 0012.
pub trait Clock {
    fn now(&self) -> OffsetDateTime;
}

/// The clock a Node runs on in production: the machine's wall clock, in UTC.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

// An RFC 3339 formatted time. Wraps the formatted string rather than
// `String` so a `created_at` field can't hold arbitrary text — the only way
// to build one is from an `OffsetDateTime`.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattedDateTime(String);

impl From<OffsetDateTime> for FormattedDateTime {
    fn from(moment: OffsetDateTime) -> Self {
        FormattedDateTime(
            moment
                .format(&Rfc3339)
                .expect("formatting a time as RFC 3339 should never fail"),
        )
    }
}

impl fmt::Display for FormattedDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub event_id: String,
    pub stream_id: String,
    // Vector clock: one counter per Node that has appended to this Stream,
    // keyed by node id. A Node absent from the map has an implicit count of
    // 0. See DECISIONS.md 0003.
    pub vector_clock: HashMap<String, u64>,
    pub event_type: String,
    // When the Store appended this Event, read from its own clock. Named for
    // the Stream field it mirrors, and to leave room for a `received_at`
    // once Events replicate between Nodes. See DECISIONS.md 0014.
    pub created_at: FormattedDateTime,
    // Opaque to the Store — stored and returned as-is, in whichever
    // encoding the calling protocol used to produce them. See
    // DECISIONS.md 0007.
    pub data: Vec<u8>,
    pub metadata: Vec<u8>,
}

// An Event as the Application supplies it: everything but the `vector_clock`
// and `created_at` the Store assigns at append time. See SPECIFICATION.md's
// Event schema.
#[derive(Debug)]
pub struct PendingEvent {
    pub event_id: String,
    pub event_type: String,
    pub data: Vec<u8>,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub stream_id: String,
    pub status: StreamStatus,
    pub created_at: FormattedDateTime,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    StreamNotFound { stream_id: String },
    StreamClosed { stream_id: String },
    StreamNotClosed { stream_id: String },
    EventIdConflict { stream_id: String, event_id: String },
    InvalidEvent { reason: String },
}

pub struct Store {
    node_id: String,
    // Boxed rather than a `Store<C: Clock>` type parameter: which clock a
    // Store runs on is of no interest to its callers, and a parameter would
    // spread through every signature that names a Store.
    clock: Box<dyn Clock>,
    streams: HashMap<String, (Stream, Vec<Event>)>,
}

impl Store {
    pub fn new(node_id: &str, clock: impl Clock + 'static) -> Self {
        Store {
            node_id: node_id.to_string(),
            clock: Box::new(clock),
            streams: HashMap::new(),
        }
    }

    /// CreateStream.01/.02 — create a Stream if it doesn't exist yet;
    /// creating an already-open Stream again is idempotent and returns it
    /// unchanged.
    pub fn create_stream(&mut self, stream_id: &str) -> Result<Stream, Error> {
        if let Some((stream, _)) = self.streams.get(stream_id) {
            return Ok(stream.clone());
        }

        let stream = Stream {
            stream_id: stream_id.to_string(),
            status: StreamStatus::Open,
            created_at: self.clock.now().into(),
        };
        self.streams
            .insert(stream_id.to_string(), (stream.clone(), Vec::new()));
        Ok(stream)
    }

    /// ReadStream.01/.02/.04 — all the Events in a Stream, in append order;
    /// an empty Stream reads as `[]`, but a Stream that was never created is
    /// a `StreamNotFound` error.
    ///
    /// The Events stay owned by the Store (DECISIONS.md 0013) — a caller that
    /// needs its own copy calls `.to_vec()`.
    pub fn read_stream(&self, stream_id: &str) -> Result<&[Event], Error> {
        let (_, events) = self
            .streams
            .get(stream_id)
            .ok_or_else(|| Error::StreamNotFound {
                stream_id: stream_id.to_string(),
            })?;

        Ok(events)
    }

    /// AppendEvent.01–.04/.09 — append an Event to the end of an existing
    /// Stream, marking it with this Node's next count for that Stream.
    pub fn append_event(&mut self, stream_id: &str, event: PendingEvent) -> Result<Event, Error> {
        let Store {
            node_id,
            clock,
            streams,
        } = self;
        let (_, events) = streams
            .get_mut(stream_id)
            .ok_or_else(|| Error::StreamNotFound {
                stream_id: stream_id.to_string(),
            })?;

        let mut vector_clock = events
            .last()
            .map(|last| last.vector_clock.clone())
            .unwrap_or_default();
        *vector_clock.entry(node_id.clone()).or_insert(0) += 1;

        let appended = Event {
            event_id: event.event_id,
            stream_id: stream_id.to_string(),
            vector_clock,
            event_type: event.event_type,
            created_at: clock.now().into(),
            data: event.data,
            metadata: event.metadata,
        };
        events.push(appended.clone());
        Ok(appended)
    }
}
