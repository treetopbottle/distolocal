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
  "type": "string", // e.g., "ToDoFinished"
  "timestamp": "ISO8601",
  "data": { "todo_id": "uuid", "status": "completed" },
  "metadata" { "schema_version": "1.7.2" },
}
```


## Features

Organized by components.

### Event and Stream management

- Create Stream
- Create Event: Todo application sends ToDoFinished event to event store for the chores event Stream.
- Read all Events in a Stream
- Read all Events in a Node
- Close a Stream (closing the books pattern)

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
