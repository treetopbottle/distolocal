mod common;

use common::{append, store, store_with, todo_created};
use distolocal::{Error, StreamStatus};

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
    append(&mut store, "Chores", todo_created(1, "Take out the trash"));

    let stream = store
        .get_stream("Chores")
        .expect("getting an existing Stream should succeed");

    assert_eq!(stream.stream_id, "Chores");
    assert_eq!(stream.status, StreamStatus::Open);
    assert_eq!(stream.created_at.to_string(), "2026-01-01T00:00:00Z");
}
