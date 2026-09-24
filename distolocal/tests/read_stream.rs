mod common;

use common::{append, pprint_events, store, store_with, todo_created, todo_finished};
use distolocal::Error;
use insta::assert_snapshot;

#[test]
fn read_stream_that_does_not_exist() {
    let store = store();

    let result = store.get_events("Groceries");

    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}

#[test]
fn read_stream_that_is_empty() {
    let store = store_with(&["Groceries"]);

    let events = store
        .get_events("Groceries")
        .expect("reading an existing Stream should succeed");

    assert_snapshot!(pprint_events(events), @"");
}

#[test]
fn read_stream_with_events() {
    let mut store = store_with(&["Chores"]);
    append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    append(&mut store, "Chores", todo_created(2, "Wash the dishes"));
    append(&mut store, "Chores", todo_finished(1));

    let events = store
        .get_events("Chores")
        .expect("reading an existing Stream should succeed");

    assert_snapshot!(pprint_events(events), @r#"
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
