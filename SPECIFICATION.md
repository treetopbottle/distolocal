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


## Features per component

### Event and Stream management

Concurrent writers to the same Stream are out of scope for this section — that is handled by Node replication (see below).

#### Narrative: Annabel and the Todo Application

Annabel installs the Todo app. The app creates a single "Lists" Stream that will record every list [Create Stream]. 

Annabel starts a "Chores" list. The app first adds this list "Lists" [Append Event], then creates the "Chores" Stream to hold its items [Create Stream].

Annabel adds "Take out the trash" to her chores list [Append Event]. Annabel adds "Wash the dishes" [Append Event].

Annabel marks "Take out the trash" as done [Append Event].

A while later, Annabel reopens the Todo app. It reads the full Stream to reconstruct her chores list, showing one open and one finished chore [Read Stream].

Annabel is marks the whole "Chores" list as finished, so the app closes the "Chores" Stream [Close Stream].

[Create Stream]: #feature-create-stream
[Append Event]: #feature-append-event
[Read Stream]: #feature-read-stream
[Close Stream]: #feature-close-stream

#### Create Stream

**Rule: idempotent — creating a Stream that already exists returns it unchanged, rather than erroring**

**`CreateStream.01`** Creating a Stream that doesn't exist
```
Given no Stream named "Lists" exists
When the Todo app creates a Stream "Lists"
Then a Stream "Lists" exists, containing no Events
```

**`CreateStream.02`** Creating a Stream that already exists
```
Given a Stream "Lists" already exists, containing: []
When the Todo app, running on a second device, creates a Stream "Lists"
Then it succeeds again, returning the existing "Lists" Stream unchanged
```

#### Append Event

Validates Event shape only, not application-level meaning (e.g. it does not check that a referenced `todo_id` exists).

**Rule: an appended Event is added to the end of the target Stream**

**`AppendEvent.01`** Recording the new list in the catalog
```
Given the Stream "Lists" contains: []
When the Todo app appends TodoListCreated(name="Chores") to "Lists"
Then the Stream "Lists" contains: [TodoListCreated("Chores")]
```

**`AppendEvent.02`** Recording the first chore in a new list
```
Given the Stream "Chores" contains: []
When the Todo app appends TodoCreated(todo_id=1, title="Take out the trash") to "Chores"
Then the Stream "Chores" contains: [TodoCreated#1]
```

**`AppendEvent.03`** Recording a second chore in an existing Stream
```
Given the Stream "Chores" contains: [TodoCreated#1]
When the Todo app appends TodoCreated(todo_id=2, title="Wash the dishes") to "Chores"
Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
```

**`AppendEvent.04`** Recording a chore as finished
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
When the Todo app appends TodoFinished(todo_id=1) to "Chores"
Then the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
```

**Rule: idempotent per `event_id`**

**`AppendEvent.05`** Retrying the same append after an interrupted write
```
Given the Stream "Chores" contains: [TodoCreated#1]
When the Todo app appends TodoCreated(todo_id=2, title="Wash the dishes") to "Chores" twice, both times using the same envelope event_id=X
Then the Stream "Chores" contains TodoCreated#2 exactly once
```
_Rationale: needed because the Todo app can't always tell whether a write landed before a crash._

**Rule: rejected with a `StreamClosed` error if the target Stream is closed**

**`AppendEvent.06`** Appending to a closed Stream
```
Given the Stream "Chores" is closed
When the Todo app appends TodoCreated(todo_id=3, ...) to "Chores"
Then the append fails with a StreamClosed error
```

**Rule: validates Event shape only, not application-level meaning**

**`AppendEvent.07`** Appending an Event referencing a `todo_id` it doesn't recognize
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2]
When the Todo app appends TodoFinished(todo_id=99) to "Chores"
Then the Store still appends it
```

**Rule: "exists but empty" is treated the same as "just created"**

**`AppendEvent.08`** App restarts after creating the Stream but before appending the first Event
```
Given a Stream "Chores" was just created and contains no Events
When the Todo app restarts
Then it appends TodoCreated(todo_id=1, ...) rather than creating "Chores" again
```

#### Read Stream

**Rule: returns all Events in append order**

**`ReadStream.01`** Reading an existing, non-empty Stream
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
When the Todo app reads all Events in "Chores"
Then it receives [TodoCreated#1, TodoCreated#2, TodoFinished#1], in that order
```

**`ReadStream.02`** Reading a Stream that exists but is empty
```
Given the Stream "Groceries" exists and contains: []
When the Todo app reads all Events in "Groceries"
Then it receives []
```

**`ReadStream.03`** Reading a closed Stream
```
Given the Stream "Chores" is closed and contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
When the Todo app reads all Events in "Chores"
Then it receives [TodoCreated#1, TodoCreated#2, TodoFinished#1]
```

**Rule: a Stream that was never created is an error, not an empty result**

**`ReadStream.04`** Reading a Stream that doesn't exist
```
Given no Stream named "Groceries" exists
When the Todo app reads all Events in "Groceries"
Then the read fails with a StreamNotFound error
```

#### Close Stream

**Rule: idempotent — closing an already-closed Stream is a no-op**

**`CloseStream.01`** Closing an open Stream
```
Given the Stream "Chores" contains: [TodoCreated#1, TodoCreated#2, TodoFinished#1]
When the Todo app closes the Stream "Chores"
Then the Stream "Chores" is closed; its Events are unchanged and still readable
```

**`CloseStream.02`** Closing an already-closed Stream
```
Given the Stream "Chores" is closed
When the Todo app closes "Chores" again
Then it succeeds again with no error
```


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
