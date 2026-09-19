// Each integration test file is its own crate and compiles this module
// separately, so helpers a given test file doesn't use would warn.
#![allow(dead_code)]

use distolocal::{Event, NewEvent, Store};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub const NODE_ID: &str = "node-a";

pub fn store() -> Store {
    Store::new(NODE_ID)
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
pub fn append(store: &mut Store, stream_id: &str, event: NewEvent) -> Event {
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
// do too — no test needs to see it until Plan.md step 5 pins one to test
// idempotency.

pub fn todo_list_created(name: &str) -> NewEvent {
    new_event("TodoListCreated", &format!(r#"{{"name":"{name}"}}"#))
}

pub fn todo_created(todo_id: u64, title: &str) -> NewEvent {
    new_event(
        "TodoCreated",
        &format!(r#"{{"todo_id":{todo_id},"title":"{title}"}}"#),
    )
}

pub fn todo_finished(todo_id: u64) -> NewEvent {
    new_event("TodoFinished", &format!(r#"{{"todo_id":{todo_id}}}"#))
}

fn new_event(event_type: &str, data: &str) -> NewEvent {
    static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

    NewEvent {
        event_id: format!("event-{}", NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed)),
        event_type: event_type.to_string(),
        data: data.as_bytes().to_vec(),
        metadata: Vec::new(),
    }
}
