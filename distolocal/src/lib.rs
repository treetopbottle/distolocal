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

    // The Stream record and its Events, as the Store holds them together.
    // Every lookup of an existing Stream goes through here or `stream_mut`,
    // so they all report a missing one the same way.
    fn stream(&self, stream_id: &str) -> Result<&(Stream, Vec<Event>), Error> {
        self.streams
            .get(stream_id)
            .ok_or_else(|| Self::stream_not_found(stream_id))
    }

    // The same lookup, for an operation that goes on to change what it
    // finds. Takes the map rather than `&mut self` so the caller can hold
    // the Store's other fields — `node_id`, `clock` — across the borrow.
    //
    // Turning a Stream away for anything but being missing belongs in the
    // operation, not here: a closed Stream still reads and still has a
    // record (ReadStream.03, GetStream.02), and only Append and Create
    // reject one (AppendEvent.06, CreateStream.03).
    fn stream_mut<'a>(
        streams: &'a mut HashMap<String, (Stream, Vec<Event>)>,
        stream_id: &str,
    ) -> Result<&'a mut (Stream, Vec<Event>), Error> {
        streams
            .get_mut(stream_id)
            .ok_or_else(|| Self::stream_not_found(stream_id))
    }

    fn stream_not_found(stream_id: &str) -> Error {
        Error::StreamNotFound {
            stream_id: stream_id.to_string(),
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

    /// GetStream.01/.03 — the Stream's own record: its id, status and
    /// `created_at`, without its Events. A Stream that was never created is
    /// a `StreamNotFound` error, as it is for `read_stream`.
    pub fn get_stream(&self, stream_id: &str) -> Result<&Stream, Error> {
        let (stream, _) = self.stream(stream_id)?;

        Ok(stream)
    }

    /// ReadStream.01/.02/.04 — all the Events in a Stream, in append order;
    /// an empty Stream reads as `[]`, but a Stream that was never created is
    /// a `StreamNotFound` error.
    ///
    /// The Events stay owned by the Store (DECISIONS.md 0013) — a caller that
    /// needs its own copy calls `.to_vec()`.
    pub fn read_stream(&self, stream_id: &str) -> Result<&[Event], Error> {
        let (_, events) = self.stream(stream_id)?;

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
        let (_, events) = Self::stream_mut(streams, stream_id)?;

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

    /// The id of every Stream the Store holds, in no particular order.
    ///
    /// Test-only: the whole-Store dump the notation renders (DECISIONS.md
    /// 0011). An Application that wants to know which Streams exist keeps a
    /// catalog Stream of its own — the "Lists" Stream in SPECIFICATION.md's
    /// narrative — rather than asking the Store what it holds, so this is
    /// not a feature and has no `FeatureRule` behind it.
    #[doc(hidden)]
    pub fn stream_ids(&self) -> Vec<&str> {
        self.streams.keys().map(String::as_str).collect()
    }
}
