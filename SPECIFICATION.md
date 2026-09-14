# Distolocal Specification

This is the specification for Distolocal. Where possible, specification by example is used.

_To do: decide on styling of domain entities (capitalize?) and examples (italics paragraph?)._

## Schemas

### Event

_Possible inspiration: CloudEvents._

```jsonc
{
  "event_id": "uuid",
  "stream_id": "string",
  "type": "string", // e.g., "TodoFinished"
  "timestamp": "ISO8601",
  "data": { "todo_id": 1, "status": "completed" },
  "metadata": { "schema_version": "1.7.2" }
}
```


## Features

Organized by components; within each component's storyline, behavior is walked through as a sequence of Interactions between Annabel and the Todo app.

### Event and Stream management: Annabel and the Todo Application

The storyline below illustrates four Store operations. Their contracts, in summary:

- **Create Stream** — idempotent: creating a Stream that already exists returns the existing Stream unchanged, rather than erroring.
- **Append Event** — idempotent per `event_id`: appending an Event whose `event_id` was already recorded is a no-op. Rejected with a `StreamClosed` error if the target Stream is closed. Validates Event shape only, not application-level meaning (e.g. it does not check that a referenced `todo_id` exists).
- **Read Stream** — returns all Events in append order, including for a closed Stream. Fails with a `StreamNotFound` error if the Stream doesn't exist; returns `[]` if it exists but is empty.
- **Close Stream** — idempotent: closing an already-closed Stream is a no-op. Stops further appends; does not affect reads.

Concurrent writers to the same Stream are out of scope for this section — that is handled by Node replication (see below).

#### Interaction 1 — Annabel installs the Todo app

Distolocal has no Streams yet.

- **Annabel:** installs the Todo app, before she has created any list.
- **Todo app:** creates a single "Lists" catalog Stream that will record every list Annabel has — this is how the Todo app later knows what all her todo lists are, without the Store needing to support enumerating Streams itself.

Main scenario: Create the Lists catalog Stream
```
Given no Stream named "Lists" exists
When the Todo app creates a Stream "Lists"
Then a Stream "Lists" exists, containing no Events
```

Alternate scenario: Annabel installs the app on a second device
```
Given a Stream "Lists" already exists, containing: []
When the Todo app, running on the second device, creates a Stream "Lists"
Then it succeeds again, returning the existing "Lists" Stream unchanged
```
_Create Stream is idempotent, so installing the app on a second device (or reinstalling) never errors or overwrites the existing catalog._

#### Interaction 2 — Annabel creates her to do list "Chores"

The Stream "Lists" exists and contains: [] (created when Annabel installed the app). One further kind of Stream is involved from here on: a Stream per list (here "Chores") that holds that list's items.

- **Annabel:** starts a "Chores" list.
- **Todo app:** first records the list's existence in the "Lists" catalog, then creates the "Chores" Stream to hold its items.

Main scenario: Record the new list in the catalog
```
Given the Stream "Lists" contains: []
When the Todo app appends TodoListCreated(name="Chores") to "Lists"
Then the Stream "Lists" contains: [TodoListCreated("Chores")]
```

Main scenario: Create the Chores Stream
```
Given the Stream "Lists" contains: [TodoListCreated("Chores")]
And no Stream named "Chores" exists
When the Todo app creates a Stream "Chores"
Then a Stream "Chores" exists, containing no Events
```

Alternate scenario: Todo app crashes between recording TodoListCreated and creating the Chores Stream
```
Given the Stream "Lists" contains: [TodoListCreated("Chores")]
And no Stream named "Chores" exists
When the Todo app restarts and opens the "Chores" list
Then it creates the Stream "Chores" — a list named in the catalog but missing its Stream is repaired, not treated as an error
```
_The event is written before the Stream, not after. Creating the Stream first and crashing before the catalog event would leave "Chores" existing but unreferenced — nothing would ever discover it, since the catalog is the only place list names are enumerated. Writing the event first means every crash leaves a recoverable trail: the catalog already names "Chores", so a missing Stream is simply (re)created on the next attempt, which Create Stream's idempotency makes safe._

#### Interaction 3 — Annabel adds a todo "Take out the trash"

Chores now contains: [] (empty Stream).

- **Annabel:** adds "Take out the trash" to her chores list.
- **Todo app:** records the addition as an Event in "Chores".

Main scenario: Record the first chore
```
Given the Stream "Chores" contains: []
When the Todo app appends TodoCreated(todo_id=1, title="Take out the trash") to "Chores"
Then the Stream "Chores" contains: [TodoCreated#1]
```
_`#1` here and below is shorthand for the Event's `todo_id`, used to keep scenario notation compact._

Alternate scenario: Todo app crashes between creating the Stream and appending the Event
```
Given a Stream "Chores" was just created and contains no Events
When the Todo app restarts
Then it must treat "exists but empty" the same as "just created" — appending TodoCreated(todo_id=1, ...) rather than creating "Chores" again
```

#### Interaction 4 — Annabel adds a todo "Wash the dishes"

Chores now contains: [TodoCreated#1].

- **Annabel:** adds "Wash the dishes".
- **Todo app:** "Chores" already exists, so it appends directly with no Stream creation needed.

Main scenario: Record a second chore in the existing Stream
```
Given the Stream "Chores" contains: [TodoCreated#1]
When the Todo app appends TodoCreated(todo_id=2, title="Wash the dishes") to "Chores"
Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
```

Alternate scenario: Todo app retries the same append after an interrupted write
```
Given the Stream "Chores" contains: [TodoCreated#1]
When the Todo app appends TodoCreated(todo_id=2, title="Wash the dishes") to "Chores" twice, both times using the same envelope event_id=X
Then the Stream "Chores" contains TodoCreated#2 exactly once
```
_Append Event is idempotent on the envelope's event_id — needed because the Todo app can't always tell whether a write landed before a crash._

#### Interaction 5 — Annabel finishes the todo "Take out the trash"

Chores now contains: [TodoCreated#1, TodoCreated#2].

- **Annabel:** marks "Take out the trash" as done.
- **Todo app:** routes the Event to the same Stream the original TodoCreated lives in.

Main scenario: Record a chore as finished
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
When the Todo app appends TodoFinished(todo_id=1) to "Chores"
Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
```

Alternate scenario: Todo app appends an Event referencing a todo_id it doesn't recognize
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
When the Todo app appends TodoFinished(todo_id=99) to "Chores"
Then the Store still appends it — it validates Event shape, not application-level meaning
```
_Worth stating as an explicit boundary: the Store is agnostic to business semantics ("does todo 99 exist?"). That is the Todo app's responsibility._

#### Interaction 6 — Annabel reopens the todo app

Chores now contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1].

- **Annabel:** opens the Todo app again; sees her chores list rebuilt, with one open and one finished chore.
- **Todo app:** reads the full Stream to reconstruct that list.

Main scenario: Rebuild the chores list from the Stream
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
When the Todo app reads all Events in "Chores"
Then it receives [TodoCreated#1, TodoCreated#2, TodoFinished#1], in that order
```
(read-only — the Stream's contents are unchanged)

Alternate scenario: Todo app reads a Stream that was never created
```
Given no Stream named "Groceries" exists
When the Todo app reads all Events in "Groceries"
Then the read fails with a StreamNotFound error
```
_Reading targets a Stream the Todo app expects to already exist; a missing Stream is an error, not an empty result._

Alternate scenario: Todo app reads a Stream that exists but is empty
```
Given the Stream "Groceries" exists and contains: []
When the Todo app reads all Events in "Groceries"
Then it receives []
```

#### Interaction 7 — Annabel closes the "Chores" list

Chores still contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1].

- **Annabel:** finished with this list — for example, she's done with chores for the month and wants to start a fresh one, so Streams don't grow unbounded.
- **Todo app:** closes "Chores" on her behalf.

Main scenario: Close the Stream
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
When the Todo app closes the Stream "Chores"
Then the Stream "Chores" is closed; its Events are unchanged and still readable
```

Alternate scenario: Todo app (bug) appends to a closed Stream
```
Given the Stream "Chores" is closed
When the Todo app appends TodoCreated(todo_id=3, ...) to "Chores"
Then the append fails with a StreamClosed error — the Todo app must route new Events to a new Stream instead
```

Alternate scenario: Reading a closed Stream still works
```
Given the Stream "Chores" is closed and contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
When the Todo app reads all Events in "Chores"
Then it receives [TodoCreated#1, TodoCreated#2, TodoFinished#1]
```
_Closing a Stream stops writes, not reads — history stays available for a "past lists" view in the app._

Alternate scenario: Todo app closes an already-closed Stream
```
Given the Stream "Chores" is closed
When the Todo app closes "Chores" again
Then it succeeds again with no error
```
_Close Stream is idempotent, mirroring Create Stream — closing an already-closed Stream is a no-op._

### Stream subscription

_To be added._

### Node replication

_To be added._


## Interaction design

_To be added. Includes UI designs and mockups. Includes commands, and view models. This is the visual representation of the software. Take inspiration from event model._


## Non-functional requirements

_To be added._


## Technical design

### Component diagram

_To be added._

### Data storage

_To be added._

### APIs

_To be added. Includes protocols (HTTP, gRPC, etc.). Also internal APIs?_


## Metrics

_To be added._
