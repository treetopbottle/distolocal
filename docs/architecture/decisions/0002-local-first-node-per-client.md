# 0002. Local-first: every client runs its own node

* Status: accepted
* Date: 2026-07-07

## Context and Problem Statement

The event store must be distributed, but should not require connectivity for a client to keep
working. How should client access to the store be architected so offline use is a first-class
case rather than a degraded fallback?

## Decision Drivers

* Offline use must be fully functional, not read-only or degraded.
* The distributed/sync story should not block a usable, testable v1.
* Should map cleanly onto a v1 that is deliberately non-distributed
  (see [plan/slices/](../../plan/slices/)).

## Considered Options

* Thin client talking to a remote server node (server-first)
* Local node per client, syncing with peers/remote when available (local-first)

## Decision Outcome

Chosen option: "local-first, node per client", because it makes offline the default operating
mode instead of an edge case, and lets v1 be built and tested as a single local node with sync
added later as an additive concern rather than a rewrite.

### Consequences

* Good, because offline behavior is exercised by default during development, not just in
  dedicated tests.
* Bad, because conflict resolution / sync semantics become unavoidable eventually and are not
  solved by this decision — only deferred to a later slice.

## Pros and Cons of the Other Considered Options

### Server-first

* Good, because a single source of truth avoids conflict resolution entirely.
* Bad, because offline becomes a special case that's easy to under-test.

