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

## Core use cases (components)

- Event and Stream management
    + Create Event; create Stream; read Events in Stream; close Stream
- Stream subscription
    + Push Event; Event hooks
- Replicate to Node
    + Add Node; Remove Node

For detailed specifications, see [SPECIFICATION.md](SPECIFICATION.md).
