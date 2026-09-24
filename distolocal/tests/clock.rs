//! The Store takes every `created_at` from its clock.

mod common;

use common::{NODE_ID, SteppingClock, pprint_store, todo_created, todo_finished};
use distolocal::Store;
use insta::assert_snapshot;

#[test]
fn store_reads_clock_on_every_call() {
    let mut store = Store::new(NODE_ID, SteppingClock::default());

    store
        .create_stream("Chores")
        .expect("creating a new Stream should succeed");
    store
        .append_event("Chores", todo_created(1, "Take out the trash"))
        .expect("appending to an open Stream should succeed");
    store
        .append_event("Chores", todo_created(2, "Wash the dishes"))
        .expect("appending to an open Stream should succeed");
    store
        .append_event("Chores", todo_finished(1))
        .expect("appending to an open Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#"
    "Chores" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
      2026-01-01T00:00:02Z TodoCreated {node-a:2}
        data {"todo_id":2,"title":"Wash the dishes"}
        metadata {}
      2026-01-01T00:00:03Z TodoFinished {node-a:3}
        data {"todo_id":1}
        metadata {}
    "#);
}
