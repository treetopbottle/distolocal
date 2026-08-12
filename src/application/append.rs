use std::sync::LazyLock;

use opentelemetry::KeyValue;
use opentelemetry::metrics::Counter;

use crate::domain::stream::{Event, Position, StreamId};
use crate::infrastructure::storage::StreamStore;

/// Count of events appended, per stream. See slice 0001's Observability section.
static EVENTS_APPENDED: LazyLock<Counter<u64>> = LazyLock::new(|| {
    opentelemetry::global::meter("distolocal")
        .u64_counter("events_appended")
        .with_description("Count of events appended, per stream")
        .build()
});

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

    #[tracing::instrument(skip(self, command), fields(stream_id = command.stream_id.as_str()))]
    pub fn handle(&self, command: AppendCommand) -> Vec<Position> {
        let event_count = command.events.len() as u64;
        let positions = self.store.append(&command.stream_id, command.events);

        EVENTS_APPENDED.add(
            event_count,
            &[KeyValue::new(
                "stream.id",
                command.stream_id.as_str().to_string(),
            )],
        );
        tracing::debug!(
            stream_id = command.stream_id.as_str(),
            ?positions,
            "appended events"
        );

        positions
    }
}
