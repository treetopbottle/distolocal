# 0007. Catch-up subscription (tail a stream)

* Status: planned

## Goal

A client can subscribe to a stream and receive new events as they're appended, without
polling ReadStream repeatedly.

## Scope

- Application: `Subscribe` handler combining a catch-up read (from
  [0003](0003-read-stream-from-position.md)) with live delivery of subsequent appends.
- Infrastructure: in-memory adapter notifies active subscribers on append.

## Out of scope

- Subscriptions across a network boundary — depends on [0005](0005-local-node-api-surface.md)
  if/when needed.
- Durable/resumable subscriptions after a node restart.
- Subscription delivery driven by sync from a peer — see
  [0009](0009-two-node-sync-happy-path.md).

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md),
[0003](0003-read-stream-from-position.md)

## Examples

```
Given a stream "orders-123" with 2 events already appended
When a client subscribes to "orders-123" from position 0
Then the subscriber immediately receives the 2 existing events in append order

Given an active subscription to "orders-123" that has caught up to position 2
When 1 more event is appended to "orders-123"
Then the subscriber receives that event without issuing another ReadStream call
```

## Observability

- Metric: active subscription count, per stream.
- Log: subscribe/unsubscribe and delivery events, at debug level.
- Trace: one span per delivered event, linked to the subscription's lifetime.
