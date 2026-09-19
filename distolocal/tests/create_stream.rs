mod common;

use common::store;
use distolocal::StreamStatus;

/// CreateStream.01 — Creating a Stream that doesn't exist.
#[test]
fn create_stream_that_does_not_exist() {
    // Given no Stream named "Lists" exists
    let mut store = store();

    // When the Todo app creates a Stream "Lists"
    let stream = store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    // Then a Stream "Lists" exists, containing no Events — that it holds no
    // Events is only checkable once ReadStream lands (Plan.md step 6).
    assert_eq!(stream.stream_id, "Lists");
    assert_eq!(stream.status, StreamStatus::Open);
}

/// CreateStream.02 — Creating a Stream that already exists is idempotent: it
/// succeeds again rather than erroring or resetting the Stream.
#[test]
fn create_stream_that_already_exists_is_idempotent() {
    // Given a Stream "Lists" already exists, containing: []
    let mut store = store();
    let first = store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    // When the Todo app creates a Stream "Lists" again
    let second = store
        .create_stream("Lists")
        .expect("creating an already-open Stream again should succeed");

    // Then it succeeds again, returning the existing "Lists" Stream unchanged
    assert_eq!(second, first);
}
