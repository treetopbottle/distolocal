mod common;

use common::{append, store, store_with, todo_created, todo_finished};
use distolocal::Error;

/// ReadStream.01 — Reading an existing, non-empty Stream.
///
/// The Store hands back the Events exactly as it stored them, in append
/// order.
#[test]
fn read_stream_with_events() {
    // Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2,
    // TodoFinished#1]
    let mut store = store_with(&["Chores"]);
    let created_1 = append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    let created_2 = append(&mut store, "Chores", todo_created(2, "Wash the dishes"));
    let finished_1 = append(&mut store, "Chores", todo_finished(1));

    // When the Todo app reads all Events in "Chores"
    let events = store
        .read_stream("Chores")
        .expect("reading an existing Stream should succeed");

    // Then it receives [TodoCreated#1, TodoCreated#2, TodoFinished#1], in
    // that order — the same Events the appends returned, unchanged.
    assert_eq!(events, vec![created_1, created_2, finished_1]);
}

/// ReadStream.02 — Reading a Stream that exists but is empty.
///
/// An empty Stream is a normal result, not an error: "Groceries" exists, it
/// just has nothing in it yet.
#[test]
fn read_stream_that_is_empty() {
    // Given the Stream "Groceries" exists and contains: []
    let mut store = store_with(&["Groceries"]);

    // When the Todo app reads all Events in "Groceries"
    let events = store
        .read_stream("Groceries")
        .expect("reading an existing Stream should succeed");

    // Then it receives []
    assert!(events.is_empty());
}

/// ReadStream.04 — Reading a Stream that doesn't exist.
///
/// A Stream that was never created is an error, not an empty result — the
/// distinction ReadStream.02 would otherwise hide.
#[test]
fn read_stream_that_was_never_created() {
    // Given no Stream named "Groceries" exists
    let store = store();

    // When the Todo app reads all Events in "Groceries"
    let result = store.read_stream("Groceries");

    // Then the read fails with a StreamNotFound error
    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}
