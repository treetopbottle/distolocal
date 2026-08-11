# 0011. Access control

* Status: planned

## Goal

A node can restrict which clients may append to or read a given stream, so the store isn't
wide open once it's exposed beyond a single trusted process.

## Scope

- Application: authorization check in front of the `Append` and `ReadStream` handlers.
- Domain: minimal permission concept (append/read per stream, per client). If this grows
  beyond that, stop and add it to the spec first rather than letting it grow in code.

## Out of scope

- Authentication mechanism/identity provider choice — this slice assumes client identity is
  already established; it's about permissions, not login.
- Fine-grained per-event ACLs.

## Depends on

[0001](0001-append-and-read-single-stream-in-memory.md)

Practically most relevant once [0005](0005-local-node-api-surface.md) exists, though not a
hard dependency.

## Examples

```
Given a client without append permission on stream "orders-123"
When the client attempts to append an event
Then the append is rejected with a permission-denied error and no event is appended

Given a client with read-only permission on stream "orders-123"
When the client calls ReadStream for "orders-123"
Then the events are returned
```

## Observability

- Metric: count of permission-denied rejections, per stream.
- Log: denied attempts at warn level, including client identity and requested operation.
- Trace: authorization check as a labeled step within the existing handler span.
