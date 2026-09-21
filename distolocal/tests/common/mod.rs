// Each integration test file is its own crate and compiles this module
// separately, so helpers a given test file doesn't use would warn.
#![allow(dead_code)]

use distolocal::{Clock, Event, PendingEvent, Store};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use time::macros::datetime;
use time::{Duration, OffsetDateTime};

pub const NODE_ID: &str = "node-a";

/// The instant a test clock starts at unless the test says otherwise.
pub const START: OffsetDateTime = datetime!(2026-01-01 00:00:00 UTC);

/// How far `SteppingClock` advances per reading unless the test says
/// otherwise.
pub const STEP: Duration = Duration::seconds(1);

/// A clock frozen at one instant: every reading is the same.
pub struct FixedClock {
    reading: OffsetDateTime,
}

impl FixedClock {
    pub fn new(reading: OffsetDateTime) -> Self {
        FixedClock { reading }
    }
}

impl Default for FixedClock {
    fn default() -> Self {
        FixedClock::new(START)
    }
}

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.reading
    }
}

/// A clock that reads `start` first and advances `step` per reading, so
/// every time the Store records in a test is distinct and in call order.
pub struct SteppingClock {
    next: Cell<OffsetDateTime>,
    step: Duration,
}

impl SteppingClock {
    pub fn new(start: OffsetDateTime, step: Duration) -> Self {
        SteppingClock {
            next: Cell::new(start),
            step,
        }
    }
}

impl Default for SteppingClock {
    fn default() -> Self {
        SteppingClock::new(START, STEP)
    }
}

impl Clock for SteppingClock {
    fn now(&self) -> OffsetDateTime {
        let now = self.next.get();
        self.next.set(now + self.step);
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

/// The vector clock this Node assigns to the `count`-th Event in a Stream.
/// Under DECISIONS.md 0005 it is the map's only entry.
pub fn clock(count: u64) -> HashMap<String, u64> {
    HashMap::from([(NODE_ID.to_string(), count)])
}

// The Events from SPECIFICATION.md's "Annabel and the Todo Application"
// narrative, taking the arguments the narrative names. The Application
// generates an `event_id` per Event (SPECIFICATION.md Event schema), so these
// do too — no test needs to see it until Plan.md step 8 pins one to test
// idempotency.

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

fn pending_event(event_type: &str, data: &str) -> PendingEvent {
    static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

    PendingEvent {
        event_id: format!("event-{}", NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed)),
        event_type: event_type.to_string(),
        data: data.as_bytes().to_vec(),
        metadata: Vec::new(),
    }
}
