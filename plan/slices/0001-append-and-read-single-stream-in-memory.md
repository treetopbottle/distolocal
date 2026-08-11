# 0001. Append and read a single stream, in-memory

* Status: planned

## Goal

A client can append events to a stream and read them back, on a single local node, with an
in-memory storage adapter. This is the thinnest possible end-to-end slice through the system.

## Scope

- Domain: `Stream` entity, `Append` command, `ReadStream` query.
- Application: command/query handlers.
- Infrastructure: in-memory storage adapter implementing the storage port
  (see [docs/architecture/overview.md](../../docs/architecture/overview.md)).

## Out of scope

- Any storage adapter other than in-memory (filesystem, S3-compatible) — later slices.
- Sync/distribution between nodes — deferred past v1
  (see [ADR-0002](../../docs/architecture/decisions/0002-local-first-node-per-client.md)).
- Access control — no auth in this slice.

## Depends on

None — this is the first slice.

## Examples

```
Given a new stream ID "orders-123" with no existing events
When 2 events are appended with no expected version
Then the stream contains 2 events at positions 0 and 1

Given a stream "orders-123" with 2 events already appended
When ReadStream is called for "orders-123"
Then the 2 events are returned in append order
```

## Observability

- Metric: count of events appended, per stream.
- Log: append and read operations at debug level, including stream ID and resulting position.
- Trace: one span per command/query handler invocation.
