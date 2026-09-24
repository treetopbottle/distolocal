mod common;

use common::{append, store, store_with, todo_created, todo_finished};
use distolocal::Error;

#[test]
fn read_stream_that_does_not_exist() {
    let store = store();

    let result = store.read_stream("Groceries");

    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}

#[test]
fn read_stream_that_is_empty() {
    let store = store_with(&["Groceries"]);

    let events = store
        .read_stream("Groceries")
        .expect("reading an existing Stream should succeed");

    assert!(events.is_empty());
}

#[test]
fn read_stream_with_events() {
    let mut store = store_with(&["Chores"]);
    let created_1 = append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    let created_2 = append(&mut store, "Chores", todo_created(2, "Wash the dishes"));
    let finished_1 = append(&mut store, "Chores", todo_finished(1));

    let events = store
        .read_stream("Chores")
        .expect("reading an existing Stream should succeed");

    assert_eq!(events, vec![created_1, created_2, finished_1]);
}
