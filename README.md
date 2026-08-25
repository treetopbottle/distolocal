# Distolocal

Distributed event storage.

## Why?

Mission: Enable applications to store events in a local database to work without an internet connection, that can be replicated for collaboration.

Vision: Developers building local-first apps adopt this store instead of building sync logic themselves, the way they'd adopt SQLite instead of writing a file format.

## Who? (roles)

- Application users
- Synchronization admins
- Developers

## What? (domain entities)

These are the domain entities. The convention is to capitalize them.

- Event
- (Event) Stream
- (Server) Node

## Core use cases (slices)

- Create Event; create Stream; read Events in Stream; close Stream
- Subscribe to Stream
- Replicate to Node

# To do

  + Use cases
- When: use cases triggers
- UI mocks
- Where: command and view model connections to UI mocks
- How: protocols
  + How often / many: performance limits
-  Metrics

