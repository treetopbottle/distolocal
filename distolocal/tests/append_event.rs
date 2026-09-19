mod common;

use common::{append, clock, store, store_with, todo_created, todo_finished, todo_list_created};
use distolocal::Error;

/// AppendEvent.01 — Recording the new list in the catalog.
///
/// The Store returns the Event as stored: the Application's fields as given,
/// plus the `vector_clock` the Store assigned.
#[test]
fn append_event_to_empty_stream() {
    // Given the Stream "Lists" contains: []
    let mut store = store_with(&["Lists"]);

    // When the Todo app appends TodoListCreated(name="Chores") to "Lists"
    let appended = store
        .append_event("Lists", todo_list_created("Chores"))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Lists" contains: [TodoListCreated("Chores")]
    // — until ReadStream lands (Plan.md step 6), checked through the Event
    // the Store returns rather than by reading the Stream back.
    assert_eq!(appended.stream_id, "Lists");
    assert_eq!(appended.event_type, "TodoListCreated");
    assert_eq!(appended.data, br#"{"name":"Chores"}"#.to_vec());
    assert_eq!(appended.vector_clock, clock(1));
}

/// AppendEvent.02 — Recording the first chore in a new list.
///
/// Vector clocks count per Stream, so "Chores" starts at 1 even though
/// "Lists" has already been appended to.
#[test]
fn append_event_counts_per_stream() {
    // Given the Stream "Chores" contains: [], and "Lists" has already been
    // appended to
    let mut store = store_with(&["Lists", "Chores"]);
    append(&mut store, "Lists", todo_list_created("Chores"));

    // When the Todo app appends TodoCreated(todo_id=1, title="Take out the
    // trash") to "Chores"
    let appended = store
        .append_event("Chores", todo_created(1, "Take out the trash"))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Chores" contains: [TodoCreated#1] — its own counter,
    // not a continuation of the one "Lists" advanced.
    assert_eq!(appended.stream_id, "Chores");
    assert_eq!(appended.vector_clock, clock(1));
}

/// AppendEvent.03 — Recording a second chore in an existing Stream.
#[test]
fn append_event_to_stream_with_existing_events() {
    // Given the Stream "Chores" contains: [TodoCreated#1]
    let mut store = store_with(&["Chores"]);
    let first = append(&mut store, "Chores", todo_created(1, "Take out the trash"));

    // When the Todo app appends TodoCreated(todo_id=2, title="Wash the
    // dishes") to "Chores"
    let second = store
        .append_event("Chores", todo_created(2, "Wash the dishes"))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
    // — the second Event lands after the first, one step further along the
    // Stream's counter.
    assert_eq!(first.vector_clock, clock(1));
    assert_eq!(second.vector_clock, clock(2));
}

/// AppendEvent.04 — Recording a chore as finished.
#[test]
fn append_event_keeps_advancing_the_stream_counter() {
    // Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
    let mut store = store_with(&["Chores"]);
    append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    append(&mut store, "Chores", todo_created(2, "Wash the dishes"));

    // When the Todo app appends TodoFinished(todo_id=1) to "Chores"
    let finished = store
        .append_event("Chores", todo_finished(1))
        .expect("appending to an open Stream should succeed");

    // Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2,
    // TodoFinished#1] — the counter keeps advancing whatever the Event type.
    assert_eq!(finished.event_type, "TodoFinished");
    assert_eq!(finished.vector_clock, clock(3));
}

/// AppendEvent.09 — Appending to a Stream that was never created.
///
/// Append does not create the Stream on the Application's behalf.
#[test]
fn append_event_to_stream_that_was_never_created() {
    // Given no Stream named "Groceries" exists
    let mut store = store();

    // When the Todo app appends TodoListCreated(name="Groceries") to
    // "Groceries"
    let result = store.append_event("Groceries", todo_list_created("Groceries"));

    // Then the append fails with a StreamNotFound error
    assert_eq!(
        result,
        Err(Error::StreamNotFound {
            stream_id: "Groceries".to_string()
        })
    );
}
