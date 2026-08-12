use crate::domain::stream::{Position, StoredEvent, StreamId};
use crate::infrastructure::storage::StreamStore;

pub struct ReadStreamQuery {
    pub stream_id: StreamId,
}

pub struct ReadStreamHandler<'a, S: StreamStore> {
    store: &'a S,
}

impl<'a, S: StreamStore> ReadStreamHandler<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self { store }
    }

    #[tracing::instrument(skip(self, query), fields(stream_id = query.stream_id.as_str()))]
    pub fn handle(&self, query: ReadStreamQuery) -> Vec<StoredEvent> {
        let stream_id = query.stream_id.clone();
        let events = self.store.read(&query.stream_id);
        let positions: Vec<Position> = events.iter().map(|stored| stored.position).collect();

        tracing::debug!(stream_id = stream_id.as_str(), ?positions, "read stream");

        events
    }
}
