mod common;

use common::{parse_store, pprint_events, store};
use distolocal::Error;
use insta::assert_snapshot;

#[test]
fn get_events_of_stream_that_does_not_exist() {
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
fn get_events_of_empty_stream() {
    let store = parse_store(
        r#"
        "Groceries" open 2026-01-01T00:00:00Z
        "#,
    );

    let events = store
        .get_events("Groceries")
        .expect("reading an existing Stream should succeed");

    assert_snapshot!(pprint_events(events), @"");
}

#[test]
fn get_events_of_stream_with_events() {
    let store = parse_store(
        r#"
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
        "#,
    );

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

#[test]
fn get_events_of_closed_stream() {
    let store = parse_store(
        r#"
        "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:04Z
          2026-01-01T00:00:01Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
          2026-01-01T00:00:02Z TodoCreated {node-a:2}
            data {"todo_id":2,"title":"Wash the dishes"}
            metadata {}
          2026-01-01T00:00:03Z TodoFinished {node-a:3}
            data {"todo_id":1}
            metadata {}
        "#,
    );

    let events = store
        .get_events("Chores")
        .expect("reading a closed Stream should succeed");

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
