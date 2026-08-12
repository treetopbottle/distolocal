pub mod in_memory;

use crate::domain::stream::{Event, Position, StoredEvent, StreamId};

/// Storage port. Domain and application code depend on this trait only, never on a
/// concrete adapter — see docs/architecture/overview.md#modules--ports.
pub trait StreamStore {
    fn append(&self, stream_id: &StreamId, events: Vec<Event>) -> Vec<Position>;
    fn read(&self, stream_id: &StreamId) -> Vec<StoredEvent>;
}
