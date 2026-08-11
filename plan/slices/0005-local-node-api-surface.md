# 0005. Local node API surface

* Status: planned

## Goal

A client can issue Append and ReadStream over a real process boundary instead of direct
library calls, so a node is a runnable, independently deployable process.

## Scope

- Application: request/response mapping for the existing `Append` and `ReadStream` handlers
  from [0001](0001-append-and-read-single-stream-in-memory.md).
- Infrastructure: a transport adapter (e.g. HTTP) fronting those handlers.

## Out of scope

- Auth — see [0011](0011-access-control.md).
- Sync between nodes — see [0009](0009-two-node-sync-happy-path.md).
- Committing to a final wire protocol; use whatever proves the boundary works.

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

Note: the transport/protocol choice may be worth an ADR if it turns out to constrain later
slices (e.g. sync) — flag before locking it in, per AGENTS.md's stop-and-ask rule.

## Examples

```
Given a running node process with no existing events for stream "orders-123"
When a client sends an Append request over the node's API for 2 events with no expected version
Then a subsequent ReadStream request over the same API returns 2 events at positions 0 and 1
```

## Observability

- Metric: request count and latency, per endpoint.
- Log: request/response at debug level (redact event payloads if large).
- Trace: one span per inbound API request, parenting the existing handler span.
