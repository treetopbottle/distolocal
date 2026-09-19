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

## 0003 — Replace sequence_number with a vector clock on Event

Accepted · 2026-09-18 · Technical · relates to 0002, 0005

Replace the single-integer `sequence_number` with a per-Stream vector clock,
`Event.vector_clock: Map<node_id, u64>` — one counter per Node that has
appended to the Stream.

- **Why:** `sequence_number` breaks when two Nodes append to one Stream. Vector
  clocks order Events within a Stream and flag genuine concurrency for the
  Application to reconcile. Enables offline collaboration on the same Stream.
- **Cost:** clock grows with the number of Nodes that have written; readers
  compare clocks, not integers.
- **Alternatives:** Stream ownership (one Node owns a Stream, others need a live
  connection) — simpler, but no offline collaboration. Owner hierarchy with
  fallback reconciliation or Stream fork — more resilient, much more complex.
- **Scope note:** adopted into the `Event` type now (see Plan.md step 2),
  ahead of Node replication, so the schema doesn't need a breaking change
  later. Under 0005's single-writer-per-Stream assumption the map holds
  exactly one entry — the appending Node's own counter — and behaves like a
  plain position counter; the concurrency-detection payoff (comparing clocks
  across Nodes) only activates once Node replication lands.


## 0004 — Implementation language

Accepted · 2026-09-17 · Technical

Rust, as a single library crate for now (`cargo new --lib distolocal`).

- **Why:** We pick Rust for several reasons. First and foremost, we want to
  learn Rust. Additionally, it is a good fit for systems programming, for
  example because it compiles to a single static binary, it has predictable
  latency, and desktop/server/WASM targets are all plausible.
- **Deferred:** the embeddable-library vs. standalone-Node-process split (and
  whether that needs a workspace with a separate binary crate) is put off
  until Node replication needs it. This entry can be revisited then if the
  split changes the crate layout.

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

Undecided · 2026-09-14 · Domain · relates to 0003, 0005

0005 scopes Create/Append/Read/Close/Delete Stream to a single writer per
Stream, but leaves open what happens when two writers race against the *same*
Node for the *same* Stream — distinct from the cross-Node case that 0002/0003
already cover.

- **Why:** tracked as its own entry so this doesn't stay an implicit loose end
  buried in 0005's Cost line.
- **Open options:** reject the losing writer with an optimistic-concurrency
  error (caller supplies the `vector_clock` count it expected to append
  after); serialize writes to a Stream behind a per-Stream lock at the Node.
  Silently picking a winner (last-write-wins) conflicts with 0002's stance
  against the Store discarding data.

## 0007 — Distolocal supports multiple wire protocols; payload encoding follows the call's protocol

Accepted · 2026-09-18 · Technical · relates to 0002, 0004

The Store's public API isn't tied to one wire protocol. JSON is first; gRPC
and Avro are anticipated. Each protocol is a thin adapter over the Store's
underlying API (plain Rust calls, per 0004) — core types stay
protocol-agnostic. `Event.data`/`metadata` are stored as opaque bytes
(`Vec<u8>`), and each adapter en/decodes them in its own protocol's native
encoding (JSON value, Avro bytes, protobuf bytes) rather than through one
forced canonical format.

- **Why:** it should be easy to swap the Store's protocol later or add
  another one alongside it, without reworking the core.
- **Cost:** no shared codec across adapters, and no stored format tag — an
  Event read through a different protocol than it was written with can't
  tell how to decode `data`/`metadata`. Left open; not a problem yet.

## 0008 — Test data lives in plain helpers, not a fixture crate

Accepted · 2026-09-19 · Technical · relates to 0004

Integration tests share setup and test data through plain helper functions in
`distolocal/tests/common/mod.rs` — no fixture framework.

- **Why:** Rust has no built-in fixtures, and the pain is the shape of the
  test data, not setup/teardown. Named constructors for the SPECIFICATION.md
  narrative Events fix that with no dev-dependency.
- **Alternatives:** `rstest`, which does offer `#[fixture]` injection and
  parameterized cases. Worth revisiting once one assertion needs to run
  across many inputs — e.g. the closed-Stream rules in Plan.md step 8.

## 0009 — Given/When/Then lives in comments, not in a test DSL

Accepted · 2026-09-19 · Technical · relates to 0008

Each test carries its `FeatureRule.NN` id and SPECIFICATION.md's own
Given/When/Then wording as comments above plain arrange/act/assert code.

- **Why:** the comments keep each test anchored to the spec text it covers —
  the part most likely to drift — while the code stays ordinary Rust that
  needs no framework to read or debug.
- **Cost:** nothing ties a comment to the code beneath it. A `Given` that
  says `[TodoCreated#1]` above a helper call using `todo_id=2` still compiles
  and passes.
- **Alternatives:** a `World` DSL — a test-only struct holding the Store, the
  Stream under test and the outcome of the last call, exposing
  `given_*`/`when_*`/`then_*` methods so each phase is a method call instead
  of a comment (`World::given_stream("Chores").containing([...])`, then
  `when_appending(...)`, then `then_stream_contains([...])`). Keeping the
  outcome rather than unwrapping it is what lets a `then_*` assert on errors
  as well as successes. It closes the drift gap above; revisit if the step 11
  narrative test gets unwieldy. The `cucumber` crate (real `.feature` files)
  was rejected as too much machinery, and it would duplicate the spec text
  unless SPECIFICATION.md became the source of those files.
