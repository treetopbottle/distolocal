mod common;

use common::{append, store, store_with, todo_created};
use distolocal::{Error, StreamStatus};

/// GetStream.01 — Getting an open Stream.
///
/// The record is the Stream itself — id, status, `created_at` — and says
/// nothing about the Events it holds; those are `read_stream`'s business.
#[test]
fn get_stream_that_is_open() {
    // Given the Stream "Chores" is open and contains: [TodoCreated#1,
    // TodoCreated#2]
    let mut store = store_with(&["Chores"]);
    append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    append(&mut store, "Chores", todo_created(2, "Wash the dishes"));

    // When the Todo app gets the Stream "Chores"
    let stream = store
        .get_stream("Chores")
        .expect("getting an existing Stream should succeed");

    // Then it receives the record for "Chores", with status open — and the
    // `created_at` from when it was created, not from its latest Event.
    assert_eq!(stream.stream_id, "Chores");
    assert_eq!(stream.status, StreamStatus::Open);
    assert_eq!(stream.created_at.to_string(), "2026-01-01T00:00:00Z");
}

/// GetStream.03 — Getting a Stream that doesn't exist.
///
/// Same distinction `read_stream` draws: never created is an error, not an
/// empty answer.
#[test]
fn get_stream_that_was_never_created() {
    // Given no Stream named "Groceries" exists
    let store = store();

    // When the Todo app gets the Stream "Groceries"
    let result = store.get_stream("Groceries");

    // Then the get fails with a StreamNotFound error
    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}
