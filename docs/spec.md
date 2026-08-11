# Specification

This is the single spec file for now — bounded contexts haven't been decided yet (see
[docs/architecture/decisions/](architecture/decisions/)), so the domain model lives in one
place until a split is warranted.

Each section below answers one question. Keep entries short; put clarifying detail in an
**Example** rather than more prose.

## Why

A distributed, local-first event store. Every client runs its own node, so it keeps working
fully offline; nodes synchronize with each other (and any remote peers) when connectivity is
available.

## Who

Roles that interact with the system.

One line each; expand only when a role's permissions or
responsibilities become relevant to a command/query below.

| Role | Description |
|---|---|
| Client application | Appends and reads events via a local node; may be offline. |
| Node operator | Runs/configures a node (local or remote peer). |

**Permissions**

A client application may be granted append and/or read permission per stream; a node rejects
operations the calling client isn't permitted to perform. Added in
[plan/slices/0011-access-control.md](../plan/slices/0011-access-control.md).

**Example**

```
Given a client without append permission on stream "orders-123"
When the client attempts to append an event
Then the append is rejected with a permission-denied error and no event is appended
```

## What

Domain entities, commands, queries, and events.

Add one subsection per concept as it's
introduced — don't pre-populate placeholders for things that aren't designed yet.

### Entities

#### Stream

An ordered, append-only sequence of events identified by a stream ID. Position is
zero-based and assigned in append order.

### Commands

#### Append

Appends one or more events to a stream. Supports an optional expected-version check for
optimistic concurrency, added in
[plan/slices/0002-expected-version-check-on-append.md](../../plan/slices/0002-expected-version-check-on-append.md).

**Examples**

```
Given a new stream ID "orders-123" with no existing events
When 2 events are appended with no expected version
Then the stream contains 2 events at positions 0 and 1
```

```
Given a stream "orders-123" with 2 events already appended (current version 1)
When 1 event is appended with expected version 0
Then the append is rejected with a version conflict error and no event is appended
```

### Queries

#### ReadStream

Returns the events of a stream in append order. Supports reading from a given position,
added in
[plan/slices/0003-read-stream-from-position.md](../../plan/slices/0003-read-stream-from-position.md).

**Examples**

```
Given a stream "orders-123" with 2 events already appended
When ReadStream is called for "orders-123"
Then the 2 events are returned in append order
```

```
Given a stream "orders-123" with 5 events already appended
When ReadStream is called for "orders-123" from position 3
Then events at positions 3 and 4 are returned in append order
```

#### ListStreams

Returns the IDs of streams that have at least one event appended. Added in
[plan/slices/0006-list-streams.md](../../plan/slices/0006-list-streams.md).

**Example**

```
Given streams "orders-123" and "orders-456" each with at least one event appended
When ListStreams is called
Then both "orders-123" and "orders-456" are returned, and a stream with no events is not
returned
```

#### Subscribe

Delivers a stream's events to a client starting from a given position, catching up on
existing events and then continuing to deliver new events as they're appended. Added in
[plan/slices/0007-catch-up-subscription.md](../../plan/slices/0007-catch-up-subscription.md).

**Examples**

```
Given a stream "orders-123" with 2 events already appended
When a client subscribes to "orders-123" from position 0
Then the subscriber immediately receives the 2 existing events in append order
```

```
Given an active subscription to "orders-123" that has caught up to position 2
When 1 more event is appended to "orders-123"
Then the subscriber receives that event without issuing another ReadStream call
```

### Events

_(none yet — no domain events beyond the appended stream events themselves so far)_

## Where

Deployment shape: one node per client, running locally; nodes may connect to peers to sync.
No servers required for a client to keep working. Diagrams go here once the node's internal
topology (storage, sync, API surface) is settled enough to draw.

## When

Triggers for commands — what causes a command to be issued (user action, another event,
a timer, a sync arriving from a peer). Document per-command as commands are added above.

## How many / how often

Operating limits — inputs for later performance/load tests. Fill in as real numbers become
known or assumed; mark assumptions explicitly so they can be revisited.

| Concern | Assumption | Status |
|---|---|---|
| Events per stream | unknown | assumption needed |
| Concurrent local writers per node | unknown | assumption needed |
| Sync latency tolerance (offline duration) | unbounded (offline-first) | decided |

