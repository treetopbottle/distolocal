mod common;

use common::{parse_store, pprint_stream, store};
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
    let store = parse_store(
        r#"
        "Chores" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
        "#,
    );

    let stream = store
        .get_stream("Chores")
        .expect("getting an existing Stream should succeed");

    assert_snapshot!(pprint_stream(stream), @r#""Chores" open 2026-01-01T00:00:00Z"#);
}

#[test]
fn get_stream_that_is_closed() {
    let store = parse_store(
        r#"
        "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z
        "#,
    );

    let stream = store
        .get_stream("Chores")
        .expect("getting a closed Stream should succeed");

    assert_snapshot!(pprint_stream(stream), @r#""Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z"#);
}
