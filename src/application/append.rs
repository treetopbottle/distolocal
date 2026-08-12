use crate::domain::stream::{Event, Position, StreamId};
use crate::infrastructure::storage::StreamStore;

pub struct AppendCommand {
    pub stream_id: StreamId,
    pub events: Vec<Event>,
}

pub struct AppendHandler<'a, S: StreamStore> {
    store: &'a S,
}

impl<'a, S: StreamStore> AppendHandler<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self { store }
    }

    pub fn handle(&self, command: AppendCommand) -> Vec<Position> {
        self.store.append(&command.stream_id, command.events)
    }
}
