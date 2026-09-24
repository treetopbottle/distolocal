// Each integration test file is its own crate and compiles this module
// separately, so helpers a given test file doesn't use would warn.
#![allow(dead_code)]

pub mod notation;

#[allow(unused_imports)]
pub use notation::{parse_store, pprint_events, pprint_store, pprint_stream};

use distolocal::{Clock, Event, PendingEvent, Store};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use time::macros::datetime;
use time::{Duration, OffsetDateTime};

pub const NODE_ID: &str = "node-a";

/// The instant a test clock starts at.
pub const START: OffsetDateTime = datetime!(2026-01-01 00:00:00 UTC);

/// How far `SteppingClock` advances per reading.
pub const STEP: Duration = Duration::seconds(1);

/// A clock that reads `START` first and advances `STEP` per reading, so every
/// time the Store records in a test is distinct and in call order.
pub struct SteppingClock {
    next: Cell<OffsetDateTime>,
}

impl Default for SteppingClock {
    fn default() -> Self {
        SteppingClock {
            next: Cell::new(START),
        }
    }
}

impl Clock for SteppingClock {
    fn now(&self) -> OffsetDateTime {
        let now = self.next.get();
        self.next.set(now + STEP);
        now
    }
}

/// A Store on the default test clock: `START`, a `STEP` per reading.
pub fn store() -> Store {
    Store::new(NODE_ID, SteppingClock::default())
}

/// A Store with `stream_ids` already created and open.
pub fn store_with(stream_ids: &[&str]) -> Store {
    let mut store = store();
    for stream_id in stream_ids {
        store
            .create_stream(stream_id)
            .expect("creating a new Stream should succeed");
    }
    store
}

/// Append as an arrange step, where the call itself isn't what's under test.
pub fn append(store: &mut Store, stream_id: &str, event: PendingEvent) -> Event {
    store
        .append_event(stream_id, event)
        .expect("appending to an open Stream should succeed")
}

// The Events from SPECIFICATION.md's Todo narrative.

pub fn todo_list_created(name: &str) -> PendingEvent {
    pending_event("TodoListCreated", &format!(r#"{{"name":"{name}"}}"#))
}

pub fn todo_created(todo_id: u64, title: &str) -> PendingEvent {
    pending_event(
        "TodoCreated",
        &format!(r#"{{"todo_id":{todo_id},"title":"{title}"}}"#),
    )
}

pub fn todo_finished(todo_id: u64) -> PendingEvent {
    pending_event("TodoFinished", &format!(r#"{{"todo_id":{todo_id}}}"#))
}

/// The same Event, carrying `metadata`.
pub fn with_metadata(event: PendingEvent, metadata: &str) -> PendingEvent {
    PendingEvent {
        metadata: metadata.as_bytes().to_vec(),
        ..event
    }
}

/// A distinct `event_id`, for the Events no test pins one on.
pub fn next_event_id() -> String {
    static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

    format!("event-{}", NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed))
}

fn pending_event(event_type: &str, data: &str) -> PendingEvent {
    PendingEvent {
        event_id: next_event_id(),
        event_type: event_type.to_string(),
        data: data.as_bytes().to_vec(),
        metadata: Vec::new(),
    }
}
