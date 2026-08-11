# 0009. Two-node sync, happy path

* Status: planned

## Goal

Two local nodes can synchronize a stream's events when connectivity is available, with no
conflicting writes, so a client's data becomes available on a peer node.

## Scope

- Infrastructure: a first adapter for the sync/transport port, which is currently undesigned
  (see [docs/architecture/overview.md](../../docs/architecture/overview.md)).
- Application: a sync handler that pushes/pulls events between two nodes.

## Out of scope

- Conflict resolution when both nodes appended independently while offline — see
  [0010](0010-sync-conflict-resolution.md).
- More than two nodes.
- Access control over which peers may sync — see [0011](0011-access-control.md).

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

**Before implementing:** this is the first slice to implement the sync/transport port named
in [ADR-0002](../../docs/architecture/decisions/0002-local-first-node-per-client.md) but left
undesigned. The transport protocol and sync algorithm change an architectural boundary — per
AGENTS.md, surface this and get an ADR written before committing to an approach, rather than
deciding unilaterally during implementation.

## Examples

Sketch only, pending the ADR above:

```
Given node A has appended 2 events to stream "orders-123" and node B has none
When node A and node B sync with no conflicting appends
Then node B's copy of "orders-123" contains the same 2 events in the same order
```

## Observability

- Metric: sync latency and events transferred, per sync cycle.
- Log: sync start/end with peer identity, at debug level.
- Trace: one span per sync cycle, per peer.
