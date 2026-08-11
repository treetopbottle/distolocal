# 0002. Expected-version check on Append

* Status: planned

## Goal

A client can supply an expected version when appending, so concurrent writers detect
conflicting appends instead of silently interleaving.

## Scope

- Domain: `Stream` gains a current-version concept used for the check.
- Application: `Append` handler validates the supplied expected version before writing.
- Infrastructure: in-memory adapter enforces the check atomically with the write.

## Out of scope

- Cross-node conflict resolution — that's a sync-time concern, see
  [0010](0010-sync-conflict-resolution.md).
- Client-side retry/backoff policies after a conflict.

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

## Examples

```
Given a stream "orders-123" with 2 events already appended (current version 1)
When 1 event is appended with expected version 1
Then the event is appended at position 2

Given a stream "orders-123" with 2 events already appended (current version 1)
When 1 event is appended with expected version 0
Then the append is rejected with a version conflict error and no event is appended
```

## Observability

- Metric: count of version-conflict rejections, per stream.
- Log: rejected appends at warn level, including expected vs. actual version.
- Trace: version check as a labeled step within the existing append handler span.
