# 0004. Filesystem storage adapter

* Status: planned

## Goal

Streams can be persisted to disk via a filesystem storage adapter, so data survives a node
restart.

## Scope

- Infrastructure: filesystem adapter implementing the storage port (see
  [docs/architecture/overview.md](../../docs/architecture/overview.md)). Must pass the
  in-memory adapter's approval test suite unchanged, per the adapter-parity rule in
  [CONTRIBUTING.md](../../CONTRIBUTING.md).

## Out of scope

- S3-compatible adapter — see [0008](0008-s3-compatible-storage-adapter.md).
- Choosing this as the node's default adapter, or making the adapter configurable.
- File-format optimization or compaction.

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

## Examples

Reuses 0001's examples against the filesystem adapter, plus a restart check:

```
Given the filesystem adapter and a new stream ID "orders-123" with no existing events
When 2 events are appended with no expected version
Then the stream contains 2 events at positions 0 and 1, and they are still readable after the
node process restarts
```

## Observability

- Metric: disk write latency per append.
- Log: file path and bytes written, at debug level.
- Trace: file I/O as a child span of the append handler span.
