# Decisions

A lightweight log of choices made and why. If we throw the code away, this is
what we keep.

Guidelines: one entry per decision, keep the *why* and the rejected
alternatives (that's the valuable part), don't renumber. Mark a decision
`Superseded by NNNN` rather than editing it.

---

## 0001 — One Node per Application, no multi-tenancy

Accepted · 2026-09-08 · Product

A Node serves exactly one Application. Multi-tenancy (one Node hosting Streams
for many Applications) is out of scope.

- **Why:** isolation, quotas, and noisy-neighbour handling are a big surface we
  don't need to prove the local-first thesis.
- **Cost:** one process per Application; revisit if a hosted/shared-Node
  offering becomes a goal.

## 0002 — Applications resolve conflicts and merges, not Distolocal

Accepted · 2026-09-08 · Domain · relates to 0003

Distolocal detects concurrency and preserves all Events. Deciding the merged
result is the Application's job, like a CRDT library leaving the data model to
the caller.

- **Why:** picking a winner (e.g. last-write-wins on timestamp) silently
  discards data and bakes a domain policy into the store.
- **Cost:** Applications must supply merge logic; higher adoption cost for
  simple apps. We still owe a concurrency-detection mechanism (0003).

## 0003 — Detect concurrent Events with vector clocks

Proposed · 2026-09-08 · Technical · relates to 0002

Augment or replace the single-integer `sequence_number` with a per-Stream
vector clock keyed by `source_node`.

- **Why:** `sequence_number` breaks when two Nodes append to one Stream. Vector
  clocks order Events within a Stream and flag genuine concurrency for the
  Application to reconcile. Enables offline collaboration on the same Stream.
- **Cost:** clock grows with the number of Nodes that have written; readers
  compare clocks, not integers.
- **Alternatives:** Stream ownership (one Node owns a Stream, others need a live
  connection) — simpler, but no offline collaboration. Owner hierarchy with
  fallback reconciliation or Stream fork — more resilient, much more complex.

## 0004 — Implementation language

Undecided · 2026-09-08 · Technical

Depends on the delivery model: embeddable library, standalone Node process, or
both. Other forces: single static binary for easy local install, predictable
replication latency, target platforms (desktop, server, WASM?). Decide the
delivery model first.
