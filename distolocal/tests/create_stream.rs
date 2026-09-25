mod common;

use common::{parse_store, pprint_store, store};
use distolocal::Error;
use insta::assert_snapshot;

#[test]
fn create_stream_that_already_exists() {
    let mut store = parse_store(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
        "#,
    );
    let existing = store
        .get_stream("Lists")
        .expect("the Stream exists")
        .clone();

    let second = store
        .create_stream("Lists")
        .expect("creating an already-open Stream again should succeed");

    assert_eq!(second, existing);
    assert_snapshot!(pprint_store(&store), @r#""Lists" open 2026-01-01T00:00:00Z"#);
}

#[test]
fn create_stream() {
    let mut store = store();

    store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#""Lists" open 2026-01-01T00:00:00Z"#);
}

#[test]
fn create_stream_that_is_closed() {
    let mut store = parse_store(
        r#"
        "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z
        "#,
    );

    let result = store.create_stream("Chores");

    assert_eq!(
        result,
        Err(Error::StreamClosed {
            stream_id: "Chores".to_string()
        })
    );
    assert_snapshot!(pprint_store(&store), @r#""Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z"#);
}

#[test]
fn create_stream_that_was_deleted() {
    let mut store = parse_store(
        r#"
        "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:02Z
          2026-01-01T00:00:01Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
        "#,
    );
    store
        .delete_stream("Chores")
        .expect("deleting a closed Stream should succeed");

    store
        .create_stream("Chores")
        .expect("recreating a deleted Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#""Chores" open 2026-01-01T00:00:03Z"#);
}
