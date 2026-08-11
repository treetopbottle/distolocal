# 0003. Read a stream from a given position

* Status: planned

## Goal

A client can read a stream starting from a given position, so it doesn't have to re-read
events it has already processed.

## Scope

- Application: `ReadStream` handler accepts an optional `fromPosition`.
- Infrastructure: in-memory adapter supports offset reads.

## Out of scope

- Live/catch-up delivery of events appended after the read — see
  [0007](0007-catch-up-subscription.md).
- Reading backward from the end of a stream.

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

## Examples

```
Given a stream "orders-123" with 5 events already appended
When ReadStream is called for "orders-123" from position 3
Then events at positions 3 and 4 are returned in append order
```

## Observability

- Metric: read count, bucketed by requested range size.
- Log: read operations at debug level, including `fromPosition`.
- Trace: `fromPosition` recorded as an attribute on the existing read handler span.
