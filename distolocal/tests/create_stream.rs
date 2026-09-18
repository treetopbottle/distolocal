use distolocal::{Store, StreamStatus};

/// CreateStream.01 — creating a Stream that doesn't exist.
#[test]
fn create_stream_that_does_not_exist() {
    let mut store = Store::new();

    let stream = store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    assert_eq!(stream.stream_id, "Lists");
    assert_eq!(stream.status, StreamStatus::Open);
}

/// CreateStream.02 — creating a Stream that already exists (and is open) is
/// idempotent: it succeeds again and returns the existing Stream unchanged,
/// rather than erroring or resetting it.
#[test]
fn create_stream_that_already_exists_is_idempotent() {
    let mut store = Store::new();
    let first = store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    let second = store
        .create_stream("Lists")
        .expect("creating an already-open Stream again should succeed");

    assert_eq!(second, first);
}
