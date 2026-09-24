mod common;

use common::{parse_store, pprint_store, store};
use insta::assert_snapshot;

#[test]
fn create_stream_that_does_not_exist() {
    let mut store = store();

    store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#""Lists" open 2026-01-01T00:00:00Z"#);
}

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
