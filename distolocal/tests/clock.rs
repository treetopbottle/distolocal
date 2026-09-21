mod common;

use common::{
    FixedClock, NODE_ID, START, STEP, SteppingClock, append, todo_created, todo_finished,
    todo_list_created,
};
use distolocal::Store;

// Plan.md step 6 — not a FeatureRule. A Stream's and an Event's `created_at`
// are Store-assigned (SPECIFICATION.md Event and Stream schemas), so these
// are the first tests that can see them at all: the Store reads the clock it was
// built with (DECISIONS.md 0012), so a test can hand it one that reads a
// known instant.

/// A Stream's `created_at` comes from the Store's clock.
#[test]
fn stream_created_at_comes_from_the_store_clock() {
    // Given a Store whose clock always reads 2026-01-01T00:00:00Z
    let mut store = Store::new(NODE_ID, FixedClock::new(START));

    // When the Todo app creates a Stream "Lists"
    let stream = store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    // Then the Stream's created_at is the instant that clock reads
    assert_eq!(stream.created_at.to_string(), "2026-01-01T00:00:00Z");
}

/// An Event's `created_at` comes from the same clock, at append time.
#[test]
fn event_created_at_comes_from_the_store_clock() {
    // Given a Store whose clock always reads 2026-01-01T00:00:00Z, holding
    // an open Stream "Lists"
    let mut store = Store::new(NODE_ID, FixedClock::new(START));
    store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    // When the Todo app appends TodoListCreated(name="Chores") to "Lists"
    let appended = store
        .append_event("Lists", todo_list_created("Chores"))
        .expect("appending to an open Stream should succeed");

    // Then the Event's created_at is the instant that clock reads
    assert_eq!(appended.created_at.to_string(), "2026-01-01T00:00:00Z");
}

/// Every call reads the clock afresh, so a clock that moves between calls
/// gives Events appended in order increasing `created_at`s — what makes the
/// times in a step 7 dump both stable and meaningful.
#[test]
fn every_call_reads_the_clock_again() {
    // Given a Store whose clock starts at 2026-01-01T00:00:00Z and advances
    // one second per reading — the clock `common::store()` hands every other
    // test, spelled out here because it is what's under test
    let mut store = Store::new(NODE_ID, SteppingClock::new(START, STEP));

    // When the Todo app creates "Chores" and appends three Events to it
    let stream = store
        .create_stream("Chores")
        .expect("creating a new Stream should succeed");
    let created_1 = append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    let created_2 = append(&mut store, "Chores", todo_created(2, "Wash the dishes"));
    let finished_1 = append(&mut store, "Chores", todo_finished(1));

    // Then each time is one second after the one before, in call order
    assert_eq!(stream.created_at.to_string(), "2026-01-01T00:00:00Z");
    assert_eq!(created_1.created_at.to_string(), "2026-01-01T00:00:01Z");
    assert_eq!(created_2.created_at.to_string(), "2026-01-01T00:00:02Z");
    assert_eq!(finished_1.created_at.to_string(), "2026-01-01T00:00:03Z");
}
