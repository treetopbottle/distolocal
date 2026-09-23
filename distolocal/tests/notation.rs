mod common;

use common::{
    append, clock, dump, given, store, store_with, todo_created, todo_finished, todo_list_created,
    with_metadata,
};
use distolocal::StreamStatus;

// Plan.md step 7c — the harness, not a FeatureRule. `dump` renders a Store
// in the step 7a notation and `given` parses the same notation back, so from
// step 7d on a test's `Then` can be a snapshot of a dump and its `Given` the
// same text. These tests are what says the two really are one notation; the
// step 7d conversion is the other check, against the field assertions the
// CreateStream and AppendEvent tests carry today.
//
// The expected dumps below sit flush against the left margin, because that
// is where a dump's Stream header starts. A `given` text is indented to sit
// with the code around it: it takes the common indentation off first.

/// A Store holding no Streams renders as nothing at all — the same empty
/// text `given` reads to build one.
#[test]
fn dump_renders_an_empty_store_as_nothing() {
    let store = store();

    assert_eq!(dump(&store), "");
}

/// An empty Stream is its header alone: quoted id, status, `created_at`.
#[test]
fn dump_renders_an_empty_stream_as_its_header() {
    let store = store_with(&["Groceries"]);

    assert_eq!(dump(&store), r#""Groceries" open 2026-01-01T00:00:00Z"#);
}

/// Events indent under their Stream and both payload lines under them —
/// `metadata {}` for the Event that carries none.
#[test]
fn dump_renders_events_under_their_stream() {
    let mut store = store_with(&["Chores"]);
    append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    append(
        &mut store,
        "Chores",
        with_metadata(todo_finished(1), r#"{"schema_version":"1.7.2"}"#),
    );

    assert_eq!(
        dump(&store),
        r#""Chores" open 2026-01-01T00:00:00Z
  2026-01-01T00:00:01Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}
  2026-01-01T00:00:02Z TodoFinished {node-a:2}
    data {"todo_id":1}
    metadata {"schema_version":"1.7.2"}"#
    );
}

/// Every Stream the Store holds renders, one block after another, in the
/// order the Streams were created rather than by id — so a dump reads in the
/// order the Application wrote it, and a `Given` copied out of one describes
/// its Streams in that same order.
#[test]
fn dump_renders_every_stream_in_the_order_they_were_created() {
    let mut store = store_with(&["Lists", "Chores"]);
    append(&mut store, "Chores", todo_created(1, "Take out the trash"));
    append(&mut store, "Lists", todo_list_created("Chores"));

    assert_eq!(
        dump(&store),
        r#""Lists" open 2026-01-01T00:00:00Z
  2026-01-01T00:00:03Z TodoListCreated {node-a:1}
    data {"name":"Chores"}
    metadata {}
"Chores" open 2026-01-01T00:00:01Z
  2026-01-01T00:00:02Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}"#
    );
}

/// `given` replays its text through the Store, so what the text describes is
/// what the Store really holds — down to the times, which drive its clock.
#[test]
fn given_replays_the_text_through_the_store() {
    let store = given(
        r#"
        "Chores" open 2026-01-01T00:00:02Z
          2026-01-01T00:00:03Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
          2026-01-01T00:00:04Z TodoFinished {node-a:2}
            data {"todo_id":1}
            metadata {"schema_version":"1.7.2"}
        "#,
    );

    let stream = store
        .get_stream("Chores")
        .expect("given should have created the Stream");
    assert_eq!(stream.status, StreamStatus::Open);
    assert_eq!(stream.created_at.to_string(), "2026-01-01T00:00:02Z");

    let events = store
        .read_stream("Chores")
        .expect("given should have created the Stream");
    assert_eq!(events.len(), 2);

    assert_eq!(events[0].event_type, "TodoCreated");
    assert_eq!(events[0].created_at.to_string(), "2026-01-01T00:00:03Z");
    assert_eq!(events[0].vector_clock, clock(1));
    assert_eq!(
        events[0].data,
        br#"{"todo_id":1,"title":"Take out the trash"}"#.to_vec()
    );
    // `metadata {}` is no metadata, not two bytes of it.
    assert!(events[0].metadata.is_empty());

    assert_eq!(events[1].event_type, "TodoFinished");
    assert_eq!(events[1].created_at.to_string(), "2026-01-01T00:00:04Z");
    assert_eq!(events[1].vector_clock, clock(2));
    assert_eq!(events[1].data, br#"{"todo_id":1}"#.to_vec());
    assert_eq!(
        events[1].metadata,
        br#"{"schema_version":"1.7.2"}"#.to_vec()
    );
}

/// Replaying is what stops a `Given` from describing a state the Store would
/// never produce: the vector clocks the text claims are asserted against the
/// ones the Store assigns (DECISIONS.md 0011).
#[test]
#[should_panic(expected = "vector clock")]
fn given_rejects_a_vector_clock_the_store_would_not_assign() {
    given(
        r#"
        "Chores" open 2026-01-01T00:00:02Z
          2026-01-01T00:00:03Z TodoCreated {node-a:7}
            data {"todo_id":1,"title":"Take out the trash"}
        "#,
    );
}

/// The clock carries on stepping from the last time the text names, so a
/// `When` lands after everything its `Given` describes (DECISIONS.md 0015).
#[test]
fn given_leaves_the_clock_stepping_where_the_text_ended() {
    let mut store = given(
        r#"
        "Chores" open 2026-01-01T00:00:02Z
          2026-01-01T00:00:03Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
        "#,
    );

    let appended = append(&mut store, "Chores", todo_finished(1));

    assert_eq!(appended.created_at.to_string(), "2026-01-01T00:00:04Z");
}

/// Stream headers sit at column 0, so one text can describe several Streams
/// — each of which dumps back as the block it came from.
#[test]
fn given_replays_every_stream_the_text_describes() {
    let store = given(
        r#"
        "Lists" open 2026-01-01T00:00:00Z
          2026-01-01T00:00:01Z TodoListCreated {node-a:1}
            data {"name":"Chores"}
            metadata {}
        "Chores" open 2026-01-01T00:00:02Z
          2026-01-01T00:00:03Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
            metadata {}
        "#,
    );

    assert_eq!(
        dump(&store),
        r#""Lists" open 2026-01-01T00:00:00Z
  2026-01-01T00:00:01Z TodoListCreated {node-a:1}
    data {"name":"Chores"}
    metadata {}
"Chores" open 2026-01-01T00:00:02Z
  2026-01-01T00:00:03Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}"#
    );
}

/// However the raw string is written — content starting on the `r#"` line,
/// or a dump pasted back in at column 0 — the same text means the same
/// thing.
#[test]
fn given_reads_a_text_however_it_is_written() {
    let store = given(
        r#""Chores" open 2026-01-01T00:00:02Z
          2026-01-01T00:00:03Z TodoCreated {node-a:1}
            data {"todo_id":1,"title":"Take out the trash"}
        "#,
    );

    let dumped = dump(&store);
    assert_eq!(dump(&given(&dumped)), dumped);
}
