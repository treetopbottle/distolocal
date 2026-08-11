# 0006. List streams

* Status: planned

## Goal

A client can discover which stream IDs exist on a node, so it isn't limited to reading
streams it already knows the ID of.

## Scope

- Application: `ListStreams` query handler.
- Infrastructure: in-memory adapter supports enumerating known stream IDs.

## Out of scope

- Pagination beyond what's needed to prove the query works.
- Filtering/search by stream ID pattern.

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

## Examples

```
Given streams "orders-123" and "orders-456" each with at least one event appended
When ListStreams is called
Then both "orders-123" and "orders-456" are returned, and a stream with no events is not
returned
```

## Observability

- Metric: count of streams returned per call.
- Log: call at debug level, including result count.
- Trace: one span per query handler invocation, consistent with 0001.
