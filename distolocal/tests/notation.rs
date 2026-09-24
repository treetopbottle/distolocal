//! Tests for common/notation helper functions.
//!
//! These are internal test functions, so there does not need to be a unit test for every corner
//! case. Generic tests suffice.

mod common;

use common::{
    parse_store, pprint_store, store, store_with, todo_created, todo_finished, todo_list_created,
    with_metadata,
};

#[test]
fn pprint_store_empty_store() {
    let store = store();

    assert_eq!(pprint_store(&store), "");
}

#[test]
fn pprint_store_empty_stream() {
    let store = store_with(&["Groceries"]);

    assert_eq!(
        pprint_store(&store),
        r#""Groceries" open 2026-01-01T00:00:00Z"#
    );
}

#[test]
fn pprint_store_renders_streams_and_events() {
    let mut store = store_with(&["Lists", "Chores"]);
    store
        .append_event("Chores", todo_created(1, "Take out the trash"))
        .expect("appending to an open Stream should succeed");
    store
        .append_event("Lists", todo_list_created("Chores"))
        .expect("appending to an open Stream should succeed");
    store
        .append_event(
            "Chores",
            with_metadata(todo_finished(1), r#"{"schema_version":"1.7.2"}"#),
        )
        .expect("appending to an open Stream should succeed");

    assert_eq!(
        pprint_store(&store),
        r#""Lists" open 2026-01-01T00:00:00Z
  2026-01-01T00:00:03Z TodoListCreated {node-a:1}
    data {"name":"Chores"}
    metadata {}
"Chores" open 2026-01-01T00:00:01Z
  2026-01-01T00:00:02Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}
  2026-01-01T00:00:04Z TodoFinished {node-a:2}
    data {"todo_id":1}
    metadata {"schema_version":"1.7.2"}"#
    );
}

#[test]
fn parse_store_is_compatible_with_pprint_store() {
    let store = parse_store(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoListCreated {node-a:1}
            data {"name":"Chores"}
            metadata {}
        "Chores" open 2026-01-01T00:00:02Z
          2026-01-01T00:00:03Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
        "#,
    );

    assert_eq!(
        pprint_store(&store),
        r#""Lists" open 2026-01-01T00:00:00Z
  2026-01-01T00:00:01Z TodoListCreated {node-a:1}
    data {"name":"Chores"}
    metadata {}
"Chores" open 2026-01-01T00:00:02Z
  2026-01-01T00:00:03Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}"#
    );

    let pp_store = pprint_store(&store);
    assert_eq!(pprint_store(&parse_store(&pp_store)), pp_store);
}
