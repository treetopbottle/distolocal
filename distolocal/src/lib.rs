use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Event {
    pub event_id: String,
    pub stream_id: String,
    // Vector clock: one counter per Node that has appended to this Stream,
    // keyed by node id. A Node absent from the map has an implicit count of
    // 0. See DECISIONS.md 0003.
    pub vector_clock: HashMap<String, u64>,
    pub event_type: String,
    pub timestamp: String,
    // Opaque to the Store — stored and returned as-is, in whichever
    // encoding the calling protocol used to produce them. See
    // DECISIONS.md 0007.
    pub data: Vec<u8>,
    pub metadata: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum StreamStatus {
    Open,
    Closed,
}

#[derive(Debug, PartialEq)]
pub struct Stream {
    pub stream_id: String,
    pub status: StreamStatus,
    pub created_at: String,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    StreamNotFound { stream_id: String },
    StreamClosed { stream_id: String },
    StreamNotClosed { stream_id: String },
    EventIdConflict { stream_id: String, event_id: String },
    InvalidEvent { reason: String },
}
