mod common;

use common::{dump, given, todo_created, todo_finished, todo_list_created};
use distolocal::Error;
use insta::assert_snapshot;

/// AppendEvent.01 — Recording the new list in the catalog.
///
/// The Store returns the Event as stored: the Application's fields as given,
/// plus the `vector_clock` the Store assigned.
#[test]
fn append_event_to_empty_stream() {
    // Given the Stream "Lists" contains: []
    let mut store = given(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
        "#,
    );

    // When the Todo app appends TodoListCreated(name="Chores") to "Lists"
    let appended = store
        .append_event("Lists", todo_list_created("Chores"))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Lists" contains: [TodoListCreated("Chores")] — and
    // what the call returned is what the Store stored, which the dump can't
    // say for itself: it renders the Store's state, not the return value.
    assert_eq!(
        appended,
        store.read_stream("Lists").expect("the Stream exists")[0]
    );
    assert_snapshot!(dump(&store, "Lists"), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "#);
}

/// AppendEvent.02 — Recording the first chore in a new list.
///
/// Vector clocks count per Stream, so "Chores" starts at 1 even though
/// "Lists" has already been appended to.
#[test]
fn append_event_counts_per_stream() {
    // Given the Stream "Chores" contains: [], and "Lists" has already been
    // appended to
    let mut store = given(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoListCreated {node-a:1}
            data {"name":"Chores"}
            metadata {}
        "Chores" open 2026-01-01T00:00:02Z
        "#,
    );

    // When the Todo app appends TodoCreated(todo_id=1, title="Take out the
    // trash") to "Chores"
    store
        .append_event("Chores", todo_created(1, "Take out the trash"))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Chores" contains: [TodoCreated#1] — its own counter,
    // not a continuation of the one "Lists" advanced above.
    assert_snapshot!(dump(&store, "Chores"), @r#"
    "Chores" open 2026-01-01T00:00:02Z
      2026-01-01T00:00:03Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
    "#);
}

/// AppendEvent.03 — Recording a second chore in an existing Stream.
#[test]
fn append_event_to_stream_with_existing_events() {
    // Given the Stream "Chores" contains: [TodoCreated#1]
    let mut store = given(
        r#"
        "Chores" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
        "#,
    );

    // When the Todo app appends TodoCreated(todo_id=2, title="Wash the
    // dishes") to "Chores"
    store
        .append_event("Chores", todo_created(2, "Wash the dishes"))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
    // — the second Event lands after the first, one step further along the
    // Stream's counter.
    assert_snapshot!(dump(&store, "Chores"), @r#"
    "Chores" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
      2026-01-01T00:00:02Z TodoCreated {node-a:2}
        data {"todo_id":2,"title":"Wash the dishes"}
        metadata {}
    "#);
}

/// AppendEvent.04 — Recording a chore as finished.
#[test]
fn append_event_keeps_advancing_the_stream_counter() {
    // Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
    let mut store = given(
        r#"
        "Chores" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
          2026-01-01T00:00:02Z TodoCreated {node-a:2}
            data {"todo_id":2,"title":"Wash the dishes"}
            metadata {}
        "#,
    );

    // When the Todo app appends TodoFinished(todo_id=1) to "Chores"
    store
        .append_event("Chores", todo_finished(1))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2,
    // TodoFinished#1] — the counter keeps advancing whatever the Event type.
    assert_snapshot!(dump(&store, "Chores"), @r#"
    "Chores" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
      2026-01-01T00:00:02Z TodoCreated {node-a:2}
        data {"todo_id":2,"title":"Wash the dishes"}
        metadata {}
      2026-01-01T00:00:03Z TodoFinished {node-a:3}
        data {"todo_id":1}
        metadata {}
    "#);
}

/// AppendEvent.09 — Appending to a Stream that was never created.
///
/// Append does not create the Stream on the Application's behalf.
#[test]
fn append_event_to_stream_that_was_never_created() {
    // Given no Stream named "Groceries" exists
    let mut store = given("");

    // When the Todo app appends TodoListCreated(name="Groceries") to
    // "Groceries"
    let result = store.append_event("Groceries", todo_list_created("Groceries"));

    // Then the append fails with a StreamNotFound error — an outcome, not
    // Stream state, so it stays an assertion (DECISIONS.md 0010).
    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}
