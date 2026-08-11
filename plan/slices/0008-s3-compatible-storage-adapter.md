# 0008. S3-compatible storage adapter

* Status: planned

## Goal

Streams can be persisted to an S3-compatible object store, so a node can use managed or
remote-durable storage instead of local disk.

## Scope

- Infrastructure: S3-compatible adapter implementing the storage port. Must pass the
  in-memory adapter's approval test suite unchanged, per the adapter-parity rule in
  [CONTRIBUTING.md](../../CONTRIBUTING.md).

## Out of scope

- Choice of specific SDK/provider beyond a generic S3-compatible API.
- Encryption-at-rest configuration.
- Cost/performance tuning.

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

## Examples

Reuses 0001's examples against the S3-compatible adapter:

```
Given the S3-compatible adapter and a new stream ID "orders-123" with no existing events
When 2 events are appended with no expected version
Then the stream contains 2 events at positions 0 and 1, retrievable via ReadStream
```

## Observability

- Metric: object store request latency and error rate, per append/read.
- Log: bucket/key at debug level.
- Trace: object store call as a child span of the handler span.
