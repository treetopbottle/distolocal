use crate::domain::stream::{StoredEvent, StreamId};
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

    pub fn handle(&self, query: ReadStreamQuery) -> Vec<StoredEvent> {
        self.store.read(&query.stream_id)
    }
}
