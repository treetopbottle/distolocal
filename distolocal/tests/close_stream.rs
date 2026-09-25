mod common;

use common::{parse_store, pprint_store, store};
use distolocal::Error;
use insta::assert_snapshot;

#[test]
fn close_stream_that_does_not_exist() {
    let mut store = store();

    let result = store.close_stream("Groceries");

    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}

#[test]
fn close_stream_that_is_already_closed() {
    let mut store = parse_store(
        r#"
        "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z
        "#,
    );
    let existing = store
        .get_stream("Chores")
        .expect("the Stream exists")
        .clone();

    let closed = store
        .close_stream("Chores")
        .expect("closing an already-closed Stream should succeed");

    assert_eq!(closed, existing);
    assert_snapshot!(pprint_store(&store), @r#""Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z"#);
}

#[test]
fn close_stream_that_is_open() {
    let mut store = parse_store(
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

    let closed = store
        .close_stream("Chores")
        .expect("closing an open Stream should succeed");

    assert_eq!(
        &closed,
        store.get_stream("Chores").expect("the Stream exists")
    );
    assert_snapshot!(pprint_store(&store), @r#"
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
    "#);
}
