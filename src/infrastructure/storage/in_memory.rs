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
    fn append(&self, stream_id: &StreamId, events: Vec<Event>) -> Vec<Position> {
        if events.is_empty() {
            return Vec::new();
        }

        let mut streams = self.streams.lock().unwrap();
        let stream = streams.entry(stream_id.clone()).or_default();
        let mut positions = Vec::with_capacity(events.len());
        for event in events {
            positions.push(stream.len() as Position);
            stream.push(event);
        }
        positions
    }

    fn read(&self, stream_id: &StreamId) -> Vec<StoredEvent> {
        let streams = self.streams.lock().unwrap();
        streams
            .get(stream_id)
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(position, event)| StoredEvent {
                position: position as Position,
                event: event.clone(),
            })
            .collect()
    }
}
