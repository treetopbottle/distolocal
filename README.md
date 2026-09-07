# Distolocal

Distributed event storage.

This is a specification of the software using a why? who? what? approach. Where possible, specification by example is used.

_To do: decide on styling of domain entities (capitalize?) and examples (italics paragraph?)._

## Why?

Mission: Enable applications to store events in a local database to work without an internet connection, that can be replicated for collaboration.

Vision: Developers building local-first apps adopt this store instead of building sync logic themselves, the way they'd adopt SQLite instead of writing a file format.


## Who? (roles)

- Application users; Annabel user of Todo application
- Synchronization admins
- Developers


## What? (domain entities)

These are the domain entities. The convention is to capitalize them.

- Event - A single record of state change; ToDoFinished Event
- Stream - A collection of Events in order of occurrence; Chores Stream
- Application - A client application running locally that directly uses the event store; To do app
- Node - A remote server that can receive events (push or pull) and replicate them; Family Todo Node

### Schemas

#### Event

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


## Core use cases (components)

- Event and Stream management
    + Create Event; create Stream; read Events in Stream; close Stream
- Stream subscription
    + Push Event; Event hooks
- Replicate to Node
    + Add Node; Remove Node


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
