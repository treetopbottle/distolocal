mod common;

use common::{dump, given, todo_created, todo_finished, todo_list_created};
use distolocal::Error;
use insta::assert_snapshot;

#[test]
fn append_event_to_stream_that_does_not_exist() {
    let mut store = given("");

    let result = store.append_event("Groceries", todo_list_created("Groceries"));

    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}

#[test]
fn append_event_to_empty_stream() {
    let mut store = given(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
        "#,
    );

    let appended = store
        .append_event("Lists", todo_list_created("Chores"))
        .expect("appending to an open Stream should succeed");

    assert_eq!(
        appended,
        store.read_stream("Lists").expect("the Stream exists")[0]
    );
    assert_snapshot!(dump(&store), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "#);
}

#[test]
fn append_event_to_single_stream() {
    let mut store = given(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoListCreated {node-a:1}
            data {"name":"Chores"}
            metadata {}
        "Chores" open 2026-01-01T00:00:02Z
        "#,
    );

    store
        .append_event("Chores", todo_created(1, "Take out the trash"))
        .expect("appending to an open Stream should succeed");

    assert_snapshot!(dump(&store), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "Chores" open 2026-01-01T00:00:02Z
      2026-01-01T00:00:03Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
    "#);
}

#[test]
fn append_event_to_stream_with_existing_events() {
    let mut store = given(
        r#"
        "Chores" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
        "#,
    );

    store
        .append_event("Chores", todo_created(2, "Wash the dishes"))
        .expect("appending to an open Stream should succeed");
    store
        .append_event("Chores", todo_finished(1))
        .expect("appending to an open Stream should succeed");

    assert_snapshot!(dump(&store), @r#"
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
