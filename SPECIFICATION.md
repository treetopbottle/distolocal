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
  "sequence_number": "int",
  "source_node": "uuid", // Where the event was created
  "type": "string", // Example: "ToDoFinished"
  "timestamp": "ISO8601",
  "data": { "todo_id": "uuid", "status": "string" },
  "metadata": { "schema_version": "string" },
}
```


## Features

Organized by components.

### Event and Stream management

- Create Stream
- Create Event: Todo application sends ToDoFinished event to event store for the chores event Stream.
- Read all Events in a Stream
- Read all Events in an Application
- Read all Streams in an Application
- Close a Stream (summarize events)

### Stream subscription

_To be added._

### Node replication

_To be added._

TODO: Decide how to handle concurrent writes. The `sequence_number` in the Event schema does not work when two Nodes write to the same stream. One idea: make a Node the owner of a Stream. Then the Application can decide if concurrent events are allowed because can be reconciled later or if you need an active connection to that Node to order the events as they come in. Possibly an owner hierarchy: if the original owner Node is not available then other Nodes should be able to decide to reconcile events or decide to fork the Stream and continue cooperation. A vector clock could be a good method to detect concurrent events.


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
