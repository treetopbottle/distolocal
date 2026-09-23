mod common;

use common::{dump, parse_store, store};
use insta::assert_snapshot;

/// CreateStream.01 — Creating a Stream that doesn't exist.
#[test]
fn create_stream_that_does_not_exist() {
    // Given no Stream named "Lists" exists
    let mut store = store();

    // When the Todo app creates a Stream "Lists"
    store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    // Then a Stream "Lists" exists, containing no Events — a header with
    // nothing indented under it.
    assert_snapshot!(dump(&store), @r#""Lists" open 2026-01-01T00:00:00Z"#);
}

/// CreateStream.02 — Creating a Stream that already exists is idempotent: it
/// succeeds again rather than erroring or resetting the Stream.
#[test]
fn create_stream_that_already_exists_is_idempotent() {
    // Given a Stream "Lists" already exists, containing: []
    let mut store = parse_store(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
        "#,
    );
    // The record the Store already holds. A returned value is not Stream
    // state, so no dump can show it — this is what the second call's return
    // is compared against.
    let existing = store
        .get_stream("Lists")
        .expect("given should have created the Stream")
        .clone();

    // When the Todo app creates a Stream "Lists" again
    let second = store
        .create_stream("Lists")
        .expect("creating an already-open Stream again should succeed");

    // Then it succeeds again, returning the existing "Lists" Stream unchanged
    // — and the Store still holds the Stream the Given describes, down to a
    // `created_at` the second call would have moved on had it reset it.
    assert_eq!(second, existing);
    assert_snapshot!(dump(&store), @r#""Lists" open 2026-01-01T00:00:00Z"#);
}
