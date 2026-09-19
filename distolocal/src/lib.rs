use std::collections::HashMap;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

// An RFC 3339 formatted timestamp. Wraps the formatted string rather than
// `String` so a `timestamp`/`created_at` field can't hold arbitrary text.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattedDateTime(String);

impl FormattedDateTime {
    pub fn now() -> Self {
        FormattedDateTime(
            OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .expect("formatting the current time as RFC 3339 should never fail"),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Event {
    pub event_id: String,
    pub stream_id: String,
    // Vector clock: one counter per Node that has appended to this Stream,
    // keyed by node id. A Node absent from the map has an implicit count of
    // 0. See DECISIONS.md 0003.
    pub vector_clock: HashMap<String, u64>,
    pub event_type: String,
    pub timestamp: FormattedDateTime,
    // Opaque to the Store — stored and returned as-is, in whichever
    // encoding the calling protocol used to produce them. See
    // DECISIONS.md 0007.
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
    streams: HashMap<String, (Stream, Vec<Event>)>,
}

impl Store {
    pub fn new() -> Self {
        Store {
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
            created_at: FormattedDateTime::now(),
        };
        self.streams
            .insert(stream_id.to_string(), (stream.clone(), Vec::new()));
        Ok(stream)
    }
}
