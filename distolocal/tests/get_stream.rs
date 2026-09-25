mod common;

use common::{pprint_stream, store, store_with, todo_created};
use distolocal::Error;
use insta::assert_snapshot;

#[test]
fn get_stream_that_does_not_exist() {
    let store = store();

    let result = store.get_stream("Groceries");

    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}

#[test]
fn get_stream_that_is_open() {
    let mut store = store_with(&["Chores"]);
    store
        .append_event("Chores", todo_created(1, "Take out the trash"))
        .expect("appending to an open Stream should succeed");

    let stream = store
        .get_stream("Chores")
        .expect("getting an existing Stream should succeed");

    assert_snapshot!(pprint_stream(stream), @r#""Chores" open 2026-01-01T00:00:00Z"#);
}

#[test]
fn get_stream_that_is_closed() {
    let mut store = store_with(&["Chores"]);
    store
        .close_stream("Chores")
        .expect("closing an open Stream should succeed");

    let stream = store
        .get_stream("Chores")
        .expect("getting a closed Stream should succeed");

    assert_snapshot!(pprint_stream(stream), @r#""Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z"#);
}
