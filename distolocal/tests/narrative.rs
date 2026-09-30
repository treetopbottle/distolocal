//! The spec's narrative, "Annabel and the Todo Application", told
//! through the Store's API with the Store's state after each part. Unlike
//! the other tests, its comments tell the story it follows.

mod common;

use common::{
    NODE_ID, SteppingClock, chores_summarized, pprint_events, pprint_store, pprint_stream,
    todo_created, todo_finished, todo_list_created,
};
use distolocal::Store;
use insta::assert_snapshot;
use time::macros::datetime;

#[test]
fn annabel_and_the_todo_application() {
    let clock = SteppingClock::default();
    let mut store = Store::new(NODE_ID, clock.clone());

    // Annabel installs the Todo app. The app creates a single "Lists" Stream
    // that will record every list.
    store
        .create_stream("Lists")
        .expect("creating a new Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#""Lists" open 2026-01-01T00:00:00Z"#);

    // Annabel starts a "Chores" list. The app first adds the list to "Lists",
    // then creates a Stream with an opaque id to hold its items. The list's
    // display name comes from the Event, not the Stream.
    store
        .append_event("Lists", todo_list_created("Chores"))
        .expect("appending to an open Stream should succeed");
    store
        .create_stream("chores-3f2a1c")
        .expect("creating a new Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "chores-3f2a1c" open 2026-01-01T00:00:02Z
    "#);

    // Annabel adds two chores to her list.
    store
        .append_event("chores-3f2a1c", todo_created(1, "Take out the trash"))
        .expect("appending to an open Stream should succeed");
    store
        .append_event("chores-3f2a1c", todo_created(2, "Wash the dishes"))
        .expect("appending to an open Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "chores-3f2a1c" open 2026-01-01T00:00:02Z
      2026-01-01T00:00:03Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
      2026-01-01T00:00:04Z TodoCreated {node-a:2}
        data {"todo_id":2,"title":"Wash the dishes"}
        metadata {}
    "#);

    // Annabel marks "Take out the trash" as done.
    store
        .append_event("chores-3f2a1c", todo_finished(1))
        .expect("appending to an open Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "chores-3f2a1c" open 2026-01-01T00:00:02Z
      2026-01-01T00:00:03Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
      2026-01-01T00:00:04Z TodoCreated {node-a:2}
        data {"todo_id":2,"title":"Wash the dishes"}
        metadata {}
      2026-01-01T00:00:05Z TodoFinished {node-a:3}
        data {"todo_id":1}
        metadata {}
    "#);

    // Two days later, Annabel reopens the Todo app. It checks that the
    // "Chores" list is still open, then reads the whole Stream to show one
    // open and one finished chore. Neither records a time.
    clock.set_next(datetime!(2026-01-03 18:30:00 UTC));

    let chores = store
        .get_stream("chores-3f2a1c")
        .expect("getting an existing Stream should succeed");

    assert_snapshot!(pprint_stream(chores), @r#""chores-3f2a1c" open 2026-01-01T00:00:02Z"#);

    let events = store
        .get_events("chores-3f2a1c")
        .expect("reading an existing Stream should succeed");

    assert_snapshot!(pprint_events(events), @r#"
    2026-01-01T00:00:03Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:04Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
    2026-01-01T00:00:05Z TodoFinished {node-a:3}
      data {"todo_id":1}
      metadata {}
    "#);

    // Annabel marks the whole "Chores" list as finished. The app appends a
    // summary of the chores completed to a "ChoresHistory" Stream, then
    // closes the "Chores" Stream.
    store
        .create_stream("ChoresHistory")
        .expect("creating a new Stream should succeed");
    store
        .append_event("ChoresHistory", chores_summarized(2, 1))
        .expect("appending to an open Stream should succeed");
    store
        .close_stream("chores-3f2a1c")
        .expect("closing an open Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "chores-3f2a1c" open 2026-01-01T00:00:02Z closed 2026-01-03T18:30:02Z
      2026-01-01T00:00:03Z TodoCreated {node-a:1}
        data {"todo_id":1,"title":"Take out the trash"}
        metadata {}
      2026-01-01T00:00:04Z TodoCreated {node-a:2}
        data {"todo_id":2,"title":"Wash the dishes"}
        metadata {}
      2026-01-01T00:00:05Z TodoFinished {node-a:3}
        data {"todo_id":1}
        metadata {}
    "ChoresHistory" open 2026-01-03T18:30:00Z
      2026-01-03T18:30:01Z ChoresSummarized {node-a:1}
        data {"total":2,"completed":1}
        metadata {}
    "#);

    // A week later, the Todo app cleans up the finished "Chores" list by
    // deleting its Stream. A delete records no time, so the week goes
    // unseen.
    store
        .delete_stream("chores-3f2a1c")
        .expect("deleting a closed Stream should succeed");

    assert_snapshot!(pprint_store(&store), @r#"
    "Lists" open 2026-01-01T00:00:00Z
      2026-01-01T00:00:01Z TodoListCreated {node-a:1}
        data {"name":"Chores"}
        metadata {}
    "ChoresHistory" open 2026-01-03T18:30:00Z
      2026-01-03T18:30:01Z ChoresSummarized {node-a:1}
        data {"total":2,"completed":1}
        metadata {}
    "#);
}
