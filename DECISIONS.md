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


Related note: The `sequence_number` in the Event schema does not work when two
Nodes write to the same stream. One idea: make a Node the owner of a Stream.
Then the Application can decide if concurrent events are allowed because can be
reconciled later or if you need an active connecti on to that Node to order the
events as they come in. Possibly an owner hierarchy: if the original owner No
de is not available then other Nodes should be able to decide to reconcile
events or decide to fork the St ream and continue cooperation. A vector clock
could be a good method to detect concurrent events.


## 0004 — Implementation language

Undecided · 2026-09-08 · Technical

Depends on the delivery model: embeddable library, standalone Node process, or
both. Other forces: single static binary for easy local install, predictable
replication latency, target platforms (desktop, server, WASM?). Decide the
delivery model first.

## 0005 — Event and Stream management assumes a single writer per Stream

Accepted · 2026-09-14 · Domain · relates to 0002, 0003

The Event and Stream management features (Create/Append/Read/Close/Delete
Stream) are specified assuming only one writer touches a given Stream at a
time. Detecting and reconciling concurrent writes to the same Stream — e.g.
two Nodes appending independently while both were offline — is entirely a
Node replication concern, not something these primitives handle.

- **Why:** keeps the core Store operations simple to specify and test in
  isolation from the harder distributed-writes problem, which is addressed
  separately in 0002 (Applications resolve conflicts) and 0003 (vector clocks
  detect them).
- **Cost:** the primitives as specified don't say what happens when two
  writers race against the *same* Node (not just across Nodes) — that case
  isn't covered by this decision or by 0002/0003 yet and needs an explicit
  answer.

## 0006 — Concurrent writers to the same Stream at the same Node

Undecided · 2026-09-14 · Domain · relates to 0005

0005 scopes Create/Append/Read/Close/Delete Stream to a single writer per
Stream, but leaves open what happens when two writers race against the *same*
Node for the *same* Stream — distinct from the cross-Node case that 0002/0003
already cover.

- **Why:** tracked as its own entry so this doesn't stay an implicit loose end
  buried in 0005's Cost line.
- **Open options:** reject the losing writer with an optimistic-concurrency
  error (caller supplies the `sequence_number` it expected to append after);
  serialize writes to a Stream behind a per-Stream lock at the Node. Silently
  picking a winner (last-write-wins) conflicts with 0002's stance against the
  Store discarding data.
