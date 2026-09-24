//! The Store takes every `created_at` from its clock.

mod common;

use common::{NODE_ID, SteppingClock, append, todo_created, todo_finished};
use distolocal::Store;

#[test]
fn store_reads_clock_on_every_call() {
    let mut store = Store::new(NODE_ID, SteppingClock::default());

    let stream = store
        .create_stream("Chores")
        .expect("creating a new Stream should succeed");
    let created_1 = append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    let created_2 = append(&mut store, "Chores", todo_created(2, "Wash the dishes"));
    let finished_1 = append(&mut store, "Chores", todo_finished(1));

    assert_eq!(stream.created_at.to_string(), "2026-01-01T00:00:00Z");
    assert_eq!(created_1.created_at.to_string(), "2026-01-01T00:00:01Z");
    assert_eq!(created_2.created_at.to_string(), "2026-01-01T00:00:02Z");
    assert_eq!(finished_1.created_at.to_string(), "2026-01-01T00:00:03Z");
}
