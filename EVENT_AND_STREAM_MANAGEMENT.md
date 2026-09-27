# Event and Stream management

The features for creating, appending to, reading, closing and deleting Streams, with the rules each one follows and an example of every rule. Part of [SPECIFICATION.md](SPECIFICATION.md), which has the schemas, the errors and the narrative these examples come from.

Concurrent writers to the same Stream are out of scope here — that is handled by Node replication.

## Rules

### [Create Stream]

- [Absent: creates an open Stream, with its `created_at` from the Store's clock, and returns it](#create-new) — CreateStream.01
- [Absent after a delete: starts a fresh, empty Stream — deleting doesn't retire an id](#create-after-delete) — CreateStream.04
- [Open: returns the Stream unchanged](#create-open) — CreateStream.02
- [Closed: fails with `StreamClosed` — Create doesn't reopen a Stream](#create-closed) — CreateStream.03

### [Append Event]

- [Absent: fails with `StreamNotFound` — Append doesn't create Streams](#append-absent) — AppendEvent.09
- [Open: adds the Event at the end, with its `created_at` and `vector_clock` from the Store, and returns it; no other Stream changes](#append-to-end) — AppendEvent.01–.03
- [Open: fails with `EventIdConflict`, carrying the stored Event, if the `event_id` is already used in the Stream](#append-event-id) — AppendEvent.05
- [Open: doesn't check what `data` means](#append-opaque-data) — AppendEvent.07
- [Open, through an API adapter: fails with `InvalidEvent` if the Event is malformed](#append-invalid-event) — AppendEvent.11
- [Closed: fails with `StreamClosed`](#append-closed) — AppendEvent.06

### [Read Stream]

- [Absent: fails with `StreamNotFound`, not an empty result](#read-absent) — ReadStream.04
- [Open: returns all Events in append order](#read-open) — ReadStream.01, .02
- [Closed: still returns its Events](#read-closed) — ReadStream.03

### [Get Stream]

- [Absent: fails with `StreamNotFound`](#get-absent) — GetStream.03
- [Open: returns the Stream's record](#get-open) — GetStream.01
- [Closed: returns the record, with its `closed_at`](#get-closed) — GetStream.02

### [Close Stream]

- [Absent: fails with `StreamNotFound`](#close-absent) — CloseStream.03
- [Open: closes it, with its `closed_at` from the Store's clock, and returns it; its Events are unchanged](#close-open) — CloseStream.02
- [Closed: no-op — returns it unchanged, keeping its first `closed_at`](#close-closed) — CloseStream.01

### [Delete Stream]

- [Absent, never created or already deleted: fails with `StreamNotFound`](#delete-absent) — DeleteStream.04
- [Open: fails with `StreamNotClosed`](#delete-open) — DeleteStream.01
- [Closed: removes it and its Events, and no other Stream](#delete-closed) — DeleteStream.02, .03

## Reading the examples

Each feature is described for the three states a Stream can be in:

- **absent** — never created, or deleted. The Store keeps no record of a deleted Stream, so the two look the same.
- **open** — created, and accepting Events.
- **closed** — closed, and accepting no more Events.

The examples write the Store's Streams in the same plain-text notation the tests use:

```
"Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:03Z
  2026-01-01T00:00:01Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}
  2026-01-01T00:00:02Z TodoFinished {node-a:2}
    data {"todo_id":1}
    metadata {"schema_version":"1.7.2"}
```

A Stream is one header line: its id in quotes, `open` and its `created_at`, and once it is closed, `closed` and its `closed_at`. Each of its Events is three lines beneath it: its `created_at`, `type` and `vector_clock`; its `data`; and its `metadata`, written `{}` when it has none. The `event_id` is left out: it is the Application's key for spotting a repeated append, not part of what the Stream holds. A Store without Streams is written `no Streams`.

The times come from the Store's clock. An example's clock starts at 2026-01-01T00:00:00Z and moves on one second each time the Store records a time; after a `Given`, it carries on from the `Given`'s last time.

## Feature: Create Stream

Creates an open Stream under an id the Application chooses, and returns it.

### Stream absent

<a id="create-new"></a>
**Rule: creates an open Stream, with its `created_at` from the Store's clock**

**`CreateStream.01`** Creating a Stream that doesn't exist
```
Given no Streams
When the Todo app creates the Stream "Lists"
Then the Store holds:
  "Lists" open 2026-01-01T00:00:00Z
```

<a id="create-after-delete"></a>
**Rule: deleting a Stream does not retire its id — Create on a deleted id starts a fresh, empty Stream**

**`CreateStream.04`** Recreating a Stream after it was deleted
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:02Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
When the Todo app deletes the Stream "Chores", then creates the Stream "Chores" again
Then the Store holds:
  "Chores" open 2026-01-01T00:00:03Z
```

### Stream open

<a id="create-open"></a>
**Rule: idempotent — creating a Stream that is already open returns it unchanged, rather than erroring**

**`CreateStream.02`** Creating a Stream that already exists
```
Given
  "Lists" open 2026-01-01T00:00:00Z
When the Todo app creates the Stream "Lists" again
Then it returns the existing "Lists" Stream, and the Store is unchanged
```

### Stream closed

<a id="create-closed"></a>
**Rule: rejected with a `StreamClosed` error — Create is not a way to reopen a Stream**

**`CreateStream.03`** Creating a Stream whose id belongs to a closed Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z
When the Todo app creates the Stream "Chores" again
Then it fails with a StreamClosed error, and the Store is unchanged
```

## Feature: Append Event

Appends an Event to the end of a Stream. The Application supplies the Event's `event_id`, `type`, `data` and `metadata`; the Store adds its `created_at`, from its own clock, and its `vector_clock`, this Node's next count for the Stream. It returns the Event as stored.

The Store never looks inside `data` or `metadata`, and does not check the Event's shape: that is done by the API adapters, before a request reaches the Store (see AppendEvent.11).

### Stream absent

<a id="append-absent"></a>
**Rule: rejected with a `StreamNotFound` error — Append does not implicitly create Streams**

**`AppendEvent.09`** Appending to a Stream that was never created
```
Given no Streams
When the Todo app appends TodoListCreated with data {"name":"Groceries"} and metadata {} to "Groceries"
Then it fails with a StreamNotFound error
```

### Stream open

<a id="append-to-end"></a>
**Rule: an appended Event is added to the end of the target Stream, and no other Stream changes**

**`AppendEvent.01`** Recording the new list in the catalog
```
Given
  "Lists" open 2026-01-01T00:00:00Z
When the Todo app appends TodoListCreated with data {"name":"Chores"} and metadata {} to "Lists"
Then it returns the Event as stored, and the Store holds:
  "Lists" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoListCreated {node-a:1}
      data {"name":"Chores"}
      metadata {}
```

**`AppendEvent.02`** Recording the first chore in a new list
```
Given
  "Lists" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoListCreated {node-a:1}
      data {"name":"Chores"}
      metadata {}
  "Chores" open 2026-01-01T00:00:02Z
When the Todo app appends TodoCreated with data {"todo_id":1,"title":"Take out the trash"} and metadata {} to "Chores"
Then the Store holds:
  "Lists" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoListCreated {node-a:1}
      data {"name":"Chores"}
      metadata {}
  "Chores" open 2026-01-01T00:00:02Z
    2026-01-01T00:00:03Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
```

**`AppendEvent.03`** Recording a second chore, then finishing the first
```
Given
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
When the Todo app appends TodoCreated with data {"todo_id":2,"title":"Wash the dishes"} and metadata {} to "Chores"
And the Todo app appends TodoFinished with data {"todo_id":1} and metadata {} to "Chores"
Then the Store holds:
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
```

`AppendEvent.04` — retired: merged into `AppendEvent.03`, which appends a second Event after the first.

<a id="append-event-id"></a>
**Rule: an `event_id` is unique within its Stream — an Append reusing one is rejected with an `EventIdConflict` error carrying the Event already stored, whatever the new Event holds**

The Store compares `event_id`s only, never the rest of the Event: it cannot tell a retry from a mistake, since `data` is opaque and a retry may encode it differently. The Application can, from the stored Event the error carries — for a retry of an append that already landed it treats the error as success, which is how it gets idempotency (see DECISIONS.md 0019). An HTTP adapter answers it with 409 Conflict.

**`AppendEvent.05`** Retrying the same append after an interrupted write
```
Given
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
When the Todo app appends TodoCreated with data {"todo_id":2,"title":"Wash the dishes"} and metadata {} to "Chores", with event_id "X"
And the Todo app appends the same Event again, with event_id "X"
Then the second append fails with an EventIdConflict error carrying the stored TodoCreated, and the Store holds:
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
```

`AppendEvent.10` — retired: merged into `AppendEvent.05`. Reusing an `event_id` for a different Event gets the same error, because the Store compares nothing but the ids; a retry of the same Event is the stronger example, since a Store that still compared the rest would let it through.

<a id="append-opaque-data"></a>
**Rule: the Store does not check what `data` means**

**`AppendEvent.07`** Appending an Event referencing a `todo_id` it doesn't recognize
```
Given
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
When the Todo app appends TodoFinished with data {"todo_id":99} and metadata {} to "Chores"
Then the Store holds:
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
    2026-01-01T00:00:03Z TodoFinished {node-a:3}
      data {"todo_id":99}
      metadata {}
```

`AppendEvent.08` — retired: it described what the Application does after a restart, not anything the Store does. Creating a Stream again is harmless anyway, since Create is idempotent for an open Stream (`CreateStream.02`).

<a id="append-invalid-event"></a>
**Rule: rejected with an `InvalidEvent` error by the API adapter if the Event's shape is malformed — the request never reaches the Store**

Shape is checked where an Event arrives as untyped input: an HTTP or gRPC request. The Store's own API takes a typed Event, which cannot be missing a field or hold one of the wrong type, so the Store does not check shape again (see DECISIONS.md 0018). Not yet tested: this waits for the API adapters.

**`AppendEvent.11`** Appending an Event missing a required field
```
Given
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
When the Todo app sends an Append request for "Chores" over HTTP, with no `type` field
Then it fails with an InvalidEvent error, and the Store is unchanged
```

### Stream closed

<a id="append-closed"></a>
**Rule: rejected with a `StreamClosed` error, before the `event_id` is checked**

**`AppendEvent.06`** Appending to a closed Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:03Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
When the Todo app appends TodoCreated with data {"todo_id":3,"title":"Water the plants"} and metadata {} to "Chores"
Then it fails with a StreamClosed error, and the Store is unchanged
```

## Feature: Read Stream

Returns a Stream's Events, in append order, without the Stream's own record (that is [Get Stream]). The examples write what it returns as the Event lines of the notation.

### Stream absent

<a id="read-absent"></a>
**Rule: rejected with a `StreamNotFound` error — a Stream that doesn't exist is an error, not an empty result**

**`ReadStream.04`** Reading a Stream that doesn't exist
```
Given no Streams
When the Todo app reads all Events in "Groceries"
Then it fails with a StreamNotFound error
```

### Stream open

<a id="read-open"></a>
**Rule: returns all Events in append order**

**`ReadStream.01`** Reading an existing, non-empty Stream
```
Given
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
When the Todo app reads all Events in "Chores"
Then it receives:
  2026-01-01T00:00:01Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}
  2026-01-01T00:00:02Z TodoCreated {node-a:2}
    data {"todo_id":2,"title":"Wash the dishes"}
    metadata {}
  2026-01-01T00:00:03Z TodoFinished {node-a:3}
    data {"todo_id":1}
    metadata {}
```

**`ReadStream.02`** Reading a Stream that exists but is empty
```
Given
  "Groceries" open 2026-01-01T00:00:00Z
When the Todo app reads all Events in "Groceries"
Then it receives no Events
```

### Stream closed

<a id="read-closed"></a>
**Rule: a closed Stream's Events can still be read**

**`ReadStream.03`** Reading a closed Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:04Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
    2026-01-01T00:00:03Z TodoFinished {node-a:3}
      data {"todo_id":1}
      metadata {}
When the Todo app reads all Events in "Chores"
Then it receives:
  2026-01-01T00:00:01Z TodoCreated {node-a:1}
    data {"todo_id":1,"title":"Take out the trash"}
    metadata {}
  2026-01-01T00:00:02Z TodoCreated {node-a:2}
    data {"todo_id":2,"title":"Wash the dishes"}
    metadata {}
  2026-01-01T00:00:03Z TodoFinished {node-a:3}
    data {"todo_id":1}
    metadata {}
```

## Feature: Get Stream

Returns the Stream's own record — its id, status, `created_at` and, once it is closed, `closed_at` — without its Events. Reading the Events is [Read Stream]; this is how an Application asks whether a Stream is still open before acting on it, rather than discovering it by failing an append. The examples write what it returns as the Stream's header line.

### Stream absent

<a id="get-absent"></a>
**Rule: rejected with a `StreamNotFound` error**

**`GetStream.03`** Getting a Stream that doesn't exist
```
Given no Streams
When the Todo app gets the Stream "Groceries"
Then it fails with a StreamNotFound error
```

### Stream open

<a id="get-open"></a>
**Rule: returns the Stream's record**

**`GetStream.01`** Getting an open Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
When the Todo app gets the Stream "Chores"
Then it receives:
  "Chores" open 2026-01-01T00:00:00Z
```

### Stream closed

<a id="get-closed"></a>
**Rule: a closed Stream's record carries its `closed_at` as well**

**`GetStream.02`** Getting a closed Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z
When the Todo app gets the Stream "Chores"
Then it receives:
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z
```

## Feature: Close Stream

Closes a Stream, so it accepts no more Events, and returns it.

### Stream absent

<a id="close-absent"></a>
**Rule: rejected with a `StreamNotFound` error**

**`CloseStream.03`** Closing a Stream that doesn't exist
```
Given no Streams
When the Todo app closes the Stream "Groceries"
Then it fails with a StreamNotFound error
```

### Stream open

<a id="close-open"></a>
**Rule: closes the Stream, with its `closed_at` from the Store's clock; its Events are unchanged and still readable**

**`CloseStream.02`** Closing an open Stream
```
Given
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
When the Todo app closes the Stream "Chores"
Then it returns the closed Stream, and the Store holds:
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:04Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
    2026-01-01T00:00:03Z TodoFinished {node-a:3}
      data {"todo_id":1}
      metadata {}
```

### Stream closed

<a id="close-closed"></a>
**Rule: idempotent — closing an already-closed Stream is a no-op, and it keeps the time it was first closed**

**`CloseStream.01`** Closing an already-closed Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:01Z
When the Todo app closes the Stream "Chores" again
Then it returns the Stream unchanged, and the Store is unchanged
```

## Feature: Delete Stream

Deletes a closed Stream and every Event in it, returning nothing. The Store keeps no record of a deleted Stream.

### Stream absent

<a id="delete-absent"></a>
**Rule: rejected with a `StreamNotFound` error — whether the Stream was never created or already deleted, since the Store keeps no record of a deleted Stream to tell the two apart (see DECISIONS.md 0023)**

An Application retrying a delete whose reply it lost gets this error, and can treat it as the Stream being gone.

**`DeleteStream.04`** Deleting a Stream that doesn't exist
```
Given no Streams
When the Todo app deletes the Stream "Groceries"
Then it fails with a StreamNotFound error
```

### Stream open

<a id="delete-open"></a>
**Rule: rejected with a `StreamNotClosed` error — only a closed Stream can be deleted**

**`DeleteStream.01`** Deleting an open Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
When the Todo app deletes the Stream "Chores"
Then it fails with a StreamNotClosed error, and the Store is unchanged
```

### Stream closed

<a id="delete-closed"></a>
**Rule: deletes the Stream and every Event in it, and nothing else**

**`DeleteStream.02`** Deleting a closed Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:04Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
    2026-01-01T00:00:03Z TodoFinished {node-a:3}
      data {"todo_id":1}
      metadata {}
When the Todo app deletes the Stream "Chores"
Then the Store holds no Streams
```

**`DeleteStream.03`** Deleting a Stream that has a summary in a separate Stream
```
Given
  "Chores" open 2026-01-01T00:00:00Z closed 2026-01-01T00:00:06Z
    2026-01-01T00:00:01Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:02Z TodoCreated {node-a:2}
      data {"todo_id":2,"title":"Wash the dishes"}
      metadata {}
    2026-01-01T00:00:03Z TodoFinished {node-a:3}
      data {"todo_id":1}
      metadata {}
  "ChoresHistory" open 2026-01-01T00:00:04Z
    2026-01-01T00:00:05Z ChoresSummarized {node-a:1}
      data {"total":2,"completed":1}
      metadata {}
When the Todo app deletes the Stream "Chores"
Then the Store holds:
  "ChoresHistory" open 2026-01-01T00:00:04Z
    2026-01-01T00:00:05Z ChoresSummarized {node-a:1}
      data {"total":2,"completed":1}
      metadata {}
```

[Create Stream]: #feature-create-stream
[Append Event]: #feature-append-event
[Read Stream]: #feature-read-stream
[Get Stream]: #feature-get-stream
[Close Stream]: #feature-close-stream
[Delete Stream]: #feature-delete-stream
