use std::collections::HashMap;
use std::sync::Mutex;

use super::StreamStore;
use crate::domain::stream::{Event, Position, StoredEvent, StreamId};

#[derive(Default)]
pub struct InMemoryStreamStore {
    streams: Mutex<HashMap<StreamId, Vec<Event>>>,
}

impl InMemoryStreamStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StreamStore for InMemoryStreamStore {
    fn append(&self, _stream_id: &StreamId, _events: Vec<Event>) -> Vec<Position> {
        todo!("slice 0001: assign zero-based positions to appended events")
    }

    fn read(&self, _stream_id: &StreamId) -> Vec<StoredEvent> {
        todo!("slice 0001: return stored events in append order")
    }
}
