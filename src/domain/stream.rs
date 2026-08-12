//! `Stream`: an ordered, append-only sequence of events identified by a stream ID.
//! Position is zero-based and assigned in append order. See docs/spec.md#stream.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamId(String);

impl StreamId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub type Position = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub payload: Vec<u8>,
}

impl Event {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEvent {
    pub position: Position,
    pub event: Event,
}
