# Decisions

A lightweight log of choices made and why. If we throw the code away, this is
what we keep.

Guidelines: one short entry per decision, keeping the *why*, the cost and the
rejected alternatives. Don't renumber. Mark a decision `Superseded by NNNN`
rather than changing what it decided. Add every new entry to the index.

## Index

### Scope and domain

- [0001 — One Node per Application, no multi-tenancy](#0001)
- [0002 — Applications resolve conflicts, not Distolocal](#0002)
- [0005 — Event and Stream management assumes a single writer per Stream](#0005)
- [0006 — Concurrent writers to one Stream at the same Node](#0006) — _Undecided_
- [0024 — Retention: protecting a Stream from deletion](#0024) — _Undecided_

### Schema

The fields themselves are in [spec/schemas.md](spec/schemas.md).

- [0003 — A vector clock on Event replaces `sequence_number`](#0003)
- [0014 — An Event's Store-assigned time is `created_at`, not `timestamp`](#0014)
- [0020 — A Stream records when it was closed](#0020) — _representation superseded by 0021_
- [0021 — A closed status carries its `closed_at`](#0021)

### Store behavior and API

- [0013 — `read_stream` returns borrowed Events](#0013) — _renamed by 0025_
- [0018 — Event shape is validated by the API adapters, not the Store](#0018)
- [0019 — The Store checks `event_id` uniqueness, not Event equality](#0019)
- [0023 — Deleting a Stream that doesn't exist is `StreamNotFound`](#0023)
- [0025 — Read Stream's function is `get_events`](#0025)

### Specification and docs

- [0026 — A feature is specified per Stream state, in the test notation](#0026)
- [0027 — Rule ids are retired, never reused](#0027)
- [0028 — Docs are split by how long they stay true](#0028)

### Architecture

- [0004 — Rust, as a single library crate](#0004)
- [0007 — Multiple wire protocols; payload encoding follows the protocol](#0007)
- [0012 — The Store takes its clock as a dependency](#0012)

### Testing

- [0008 — Test data lives in plain helpers, not a fixture crate](#0008)
- [0009 — Given/When/Then lives in comments](#0009) — _superseded by 0017_
- [0010 — Approval testing for Stream contents](#0010)
- [0011 — One test-only notation for both Given and Then](#0011)
- [0015 — A `Given`'s times drive the Store's clock](#0015) — _mechanism superseded by 0022_
- [0016 — `dump` renders the whole Store, via a test-only `stream_ids`](#0016)
- [0017 — A test says what it covers in its name, not in comments](#0017)
- [0022 — A `Given` sets the clock to each of its times](#0022)

---

<a id="0001"></a>
## 0001 — One Node per Application, no multi-tenancy

Accepted · 2026-09-08 · Product

A Node serves exactly one Application.

- **Why:** isolation, quotas and noisy neighbours are a big surface we don't
  need to prove the local-first thesis.
- **Cost:** one process per Application. Revisit for a hosted, shared-Node
  offering.

<a id="0002"></a>
## 0002 — Applications resolve conflicts, not Distolocal

Accepted · 2026-09-08 · Domain · relates to 0003

Distolocal detects concurrency and keeps every Event. The Application decides
the merged result, as with a CRDT library.

- **Why:** picking a winner (e.g. last-write-wins) silently discards data and
  bakes a domain policy into the store.
- **Cost:** Applications must supply merge logic, a higher bar for simple apps.

<a id="0003"></a>
## 0003 — A vector clock on Event replaces `sequence_number`

Accepted · 2026-09-18 · Technical · relates to 0002, 0005

`Event.vector_clock: Map<node_id, u64>`: one counter per Node that has
appended to the Stream.

- **Why:** a single `sequence_number` breaks when two Nodes append to one
  Stream. A vector clock orders Events and flags real concurrency, which
  allows offline collaboration on one Stream.
- **Cost:** the clock grows with the number of writing Nodes; readers compare
  clocks, not integers.
- **Alternatives:** Stream ownership, where other Nodes need a live connection
  — simpler, but no offline collaboration. An owner hierarchy with fallback
  reconciliation or forks — much more complex.
- **Note:** adopted ahead of replication to avoid a breaking schema change.
  Under 0005 the map holds one entry and acts as a position counter.

<a id="0004"></a>
## 0004 — Rust, as a single library crate

Accepted · 2026-09-17 · Technical

- **Why:** first, we want to learn Rust. It also suits systems work: a single
  static binary, predictable latency, and desktop, server and WASM targets.
- **Deferred:** embeddable library vs. standalone Node process, and whether
  that needs a workspace with a binary crate, until replication needs it.

<a id="0005"></a>
## 0005 — Event and Stream management assumes a single writer per Stream

Accepted · 2026-09-14 · Domain · relates to 0002, 0003

Create, Append, Read, Close and Delete Stream assume one writer per Stream at
a time. Concurrent writes from different Nodes are Node replication's concern
(0002, 0003).

- **Why:** keeps the core operations simple to specify and test.
- **Cost:** leaves open two writers racing on the same Node (0006).

<a id="0006"></a>
## 0006 — Concurrent writers to one Stream at the same Node

Undecided · 2026-09-14 · Domain · relates to 0003, 0005

What happens when two writers race for the same Stream on the same Node.

- **Options:** reject the loser with an optimistic-concurrency error, the
  caller passing the `vector_clock` count it expects to follow; or serialize
  writes behind a per-Stream lock. Last-write-wins is out (0002).

<a id="0007"></a>
## 0007 — Multiple wire protocols; payload encoding follows the protocol

Accepted · 2026-09-18 · Technical · relates to 0002, 0004

JSON first, gRPC and Avro later, each a thin adapter over the Rust API.
`data` and `metadata` are stored as opaque bytes, which each adapter en- and
decodes in its protocol's native encoding.

- **Why:** protocols can be swapped or added without touching the core.
- **Cost:** no shared codec and no stored format tag, so an Event read through
  a different protocol than it was written with can't be decoded. Open; not a
  problem yet.

<a id="0008"></a>
## 0008 — Test data lives in plain helpers, not a fixture crate

Accepted · 2026-09-19 · Technical · relates to 0004

Shared setup and test data live in `distolocal/tests/common/mod.rs`.

- **Why:** the pain is the shape of the test data, not setup and teardown.
  Named constructors fix that without a dev-dependency.
- **Alternatives:** `rstest`, for fixtures and parameterized cases — revisit
  when one assertion has to run across many inputs.

<a id="0009"></a>
## 0009 — Given/When/Then lives in comments

Superseded by 0017 · 2026-09-19 · Technical · relates to 0008

Each test carried its `FeatureRule.NN` id and the spec's Given/When/Then as
comments above plain arrange/act/assert code.

- **Why:** anchors each test to the spec text; the code stays plain Rust.
- **Cost:** nothing ties a comment to the code beneath it. 0011 closed that
  gap for the `Given`.
- **Alternatives:** a `World` DSL, holding the Store and the last outcome
  behind `given_*`/`when_*`/`then_*` methods — closes the gap, but more
  machinery. `cucumber` — too heavy, and duplicates the spec text.

<a id="0010"></a>
## 0010 — Approval testing for Stream contents

Accepted · 2026-09-19 · Technical · relates to 0009, 0011, 0012

A test's `Then` is an inline `insta` snapshot of the rendered state, not a
list of field assertions.

- **Why:** one snapshot covers order, vector clocks and payloads, and reads
  like the spec's notation. Field assertions pass one by one while the state
  as a whole goes unreviewed.
- **Cost:** a snapshot is easy to accept unread. So snapshots are written by
  hand from the spec, and `cargo insta review` is only for deliberate updates
  to an approved snapshot, never for filling in a blank.
- **Exception:** the Read and Get Stream tests snapshot what the call returns
  (`pprint_events`, `pprint_stream`), not `pprint_store`, which is built on
  those calls.
- **Alternatives:** `expect-test` — `insta` has `.snap` files, redactions and
  a review workflow. Field assertions throughout — unreadable in the
  narrative test.

<a id="0011"></a>
## 0011 — One test-only notation for both Given and Then

Accepted · 2026-09-19 · Technical · relates to 0008, 0009, 0010

`dump` renders Store state as text; `given(text)` parses the same text and
replays it through the public API. They are now `pprint_store` and
`parse_store` (0017).

- **Why:** the before and after read in one shape. Replaying through the API
  lets `given` check the vector clocks the Store assigns, so a `Given` can't
  describe a state the Store would never produce.
- **Cost:** a second description of the Event schema to keep in step.
- **Scope:** test-only. Not a serialization or export format, and it doesn't
  preempt the persistence slice.

<a id="0012"></a>
## 0012 — The Store takes its clock as a dependency

Accepted · 2026-09-19 · Technical · relates to 0010

The Store holds a `Clock`: `SystemClock` in production, a stepping clock in
tests.

- **Why:** Store-assigned times belong in snapshots, which need them
  deterministic. Injecting the clock also makes them assertable.
- **Cost:** a production change made for tests, and `Store::new` takes an
  argument. Accepted since a Node will have to reason about time anyway.
- **Alternatives:** redact the times in snapshots, or leave them out of the
  notation — both leave the Store's clock untested.

<a id="0013"></a>
## 0013 — `read_stream` returns borrowed Events

Accepted · 2026-09-21 · Technical · relates to 0007 · renamed `get_events` by 0025

`read_stream(&self, stream_id) -> Result<&[Event], Error>`. A caller that
wants its own copy writes `.to_vec()`.

- **Why:** the common read folds the Events into state and drops them, so an
  owned `Vec` would clone every payload for nothing. The caller decides, as
  with `HashMap::get`.
- **Cost:** a read can't be held across an append without a copy.
- **Alternatives:** `Vec<Event>` — an allocation on every read, ahead of
  whatever persistence decides.

<a id="0014"></a>
## 0014 — An Event's Store-assigned time is `created_at`, not `timestamp`

Accepted · 2026-09-21 · Technical · relates to 0003, 0012

The same name a Stream uses for the same thing: when the Store recorded it.

- **Why:** `timestamp` doesn't say which time. Once Events replicate there
  are two, `created_at` at the origin Node and `received_at` here. It also
  pairs with `vector_clock` as "when and where".
- **Cost:** a schema rename, cheap now and expensive later.
- **Alternatives:** `timestamp`, the event-store convention — ambiguous
  exactly where replication is hardest.

<a id="0015"></a>
## 0015 — A `Given`'s times drive the Store's clock

Accepted · 2026-09-21 · Technical · relates to 0011, 0012 · mechanism superseded by 0022

The times in a `Given` are inputs to the Store's clock, not assertions. Its
vector clocks are still checked.

- **Why:** a Node has one clock across all its Streams, so asserting times
  would tie each `Given` block to whatever came before it. A block copied out
  of a dump would only work in its original position.
- **Cost:** a `Given` can claim an implausible time, e.g. an Event older than
  its Stream.
- **Alternatives:** optional times, asserted when present — the same
  copied-block problem. A clock per Stream — hides what replication has to
  reason about.

<a id="0016"></a>
## 0016 — `dump` renders the whole Store, via a test-only `stream_ids`

Accepted · 2026-09-23 · Technical · relates to 0010, 0011

`dump(&store)` renders every Stream, ordered by `created_at` then id, through
a `#[doc(hidden)]` `Store::stream_ids()`.

- **Why:** a `Then` should show everything the `When` left behind; an append
  or delete that touches the wrong Stream is otherwise invisible. Creation
  order keeps a dump in the order of the `Given` it came from.
- **Cost:** a public accessor no feature uses. Listing Streams is deliberately
  not a feature: an Application keeps its own catalog Stream, like "Lists" in
  the narrative.
- **Alternatives:** a `test-support` Cargo feature to compile it out — too
  much for what a comment says. Ordering by id — pulls blocks out of their
  `Given`'s order. Tracking creation order in the Store — a second index kept
  up for tests.

<a id="0017"></a>
## 0017 — A test says what it covers in its name, not in comments

Accepted · 2026-09-23 · Technical · supersedes 0009 · relates to 0010, 0011

No `FeatureRule.NN` ids and no Given/When/Then comments. A test is named
`<function>_<situation>`, e.g. `append_event_to_stream_that_does_not_exist`,
and shows state through `parse_store` and `pprint_store`. The harness gets one
test per behavior. Exception: the comments in `tests/narrative.rs` retell the
spec's narrative.

- **Why:** since 0011 the `Given` and `Then` are text the Store checks, so the
  comments only repeated the code and could drift from it. Comments naming a
  Plan.md step or a decision go stale.
- **Cost:** finding the test for a rule means reading test names. A step's
  checkpoint is where rule ids get matched to tests.
- **Alternatives:** 0009's comments, or a bare `// AppendEvent.04` line.

<a id="0018"></a>
## 0018 — Event shape is validated by the API adapters, not the Store

Accepted · 2026-09-25 · Technical · relates to 0007

`InvalidEvent` is raised by the adapter that decodes the request. The Store
takes a typed `PendingEvent` and doesn't check its shape again.

- **Why:** a missing or mistyped field only exists in untyped input; in Rust
  the compiler rules it out. Testing it in the Store meant inventing a
  stand-in, like an empty `event_type` for a missing one, which is a
  different rule.
- **Cost:** each adapter validates on its own. `AppendEvent.11` has no test
  until the API slice, and the Store accepts an empty `event_type` or
  `event_id`.
- **Alternatives:** rejecting empty strings — the stand-in problem. Optional
  fields on `PendingEvent` — a burden on every typed caller.

<a id="0019"></a>
## 0019 — The Store checks `event_id` uniqueness, not Event equality

Accepted · 2026-09-25 · Technical · relates to 0007

An Append whose `event_id` is already in the Stream fails with
`EventIdConflict`, carrying the stored Event, whatever the new one holds. The
Application tells a retry from a reused id. An HTTP adapter answers 409.

- **Why:** payloads are opaque bytes (0007), and a retry that re-encodes JSON
  can change key order, so the Store can't judge equality. It still checks
  uniqueness, or a retry would append a duplicate.
- **Cost:** Applications must treat `EventIdConflict` on a retry as success.
- **Alternatives:** succeeding on an exact retry, the original rule — the
  encoding problem. No check, leaving it to replication — an interrupted write
  is between an Application and its own Node, which replication never sees.

<a id="0020"></a>
## 0020 — A Stream records when it was closed

Accepted · 2026-09-25 · Technical · relates to 0012, 0015 · representation superseded by 0021

A Stream has `closed_at`: `null` while open, set from the Store's clock by
the first close and kept by later ones. In the notation a closed Stream's
header reads `"Chores" open <created_at> closed <closed_at>`.

- **Why:** an Application that cleans up closed Streams after a while, as the
  narrative's does after a week, needs the time, and the Store already knows
  it.
- **Cost:** `status` and `closed_at` say the same thing twice. A close now
  reads the clock.
- **Alternatives:** `StreamStatus::Closed { closed_at }` — taken later, in
  0021. Leaving it to the Application.

<a id="0021"></a>
## 0021 — A closed status carries its `closed_at`

Accepted · 2026-09-25 · Technical · supersedes 0020's representation

`StreamStatus::Closed { closed_at }`, beside `StreamStatus::Open`. The spec's
schema keeps `status` and a nullable `closed_at` as two fields.

- **Why:** a status that disagrees with `closed_at` becomes impossible to
  write. Matching the schema one to one bought nothing: nothing serializes a
  Stream yet.
- **Cost:** serializing has to split the status into the two schema fields,
  and reading the time takes a `match`.
- **Alternatives:** 0020's two fields. Deriving the status from `closed_at` —
  makes the status a derived idea, and each further state another nullable
  field.

<a id="0022"></a>
## 0022 — A `Given` sets the clock to each of its times

Accepted · 2026-09-25 · Technical · supersedes 0015's mechanism · relates to 0012

`parse_store` sets the clock to each time just before the call that records
it; after the text, the clock steps on. One `SteppingClock` does both, with
`set_next`.

- **Why:** 0015's clock read out the times as a script, so a `Given` only
  replayed right if each call read the clock exactly once. That pinned down
  read counts the Store doesn't promise.
- **Cost:** a call that read the clock twice before recording would use up
  its time — not a case worth designing for. Tests that step through calls in
  a row still assume one read each. The clock sits behind an `Rc` so the
  harness can reach it.
- **Alternatives:** freezing the clock at each time — a two-state clock just
  for the multi-read case. A clock that is only ever set — every test making
  calls in a row would have to set it. A separate `ScriptedClock` — two
  clocks for one job.

<a id="0023"></a>
## 0023 — Deleting a Stream that doesn't exist is `StreamNotFound`

Accepted · 2026-09-25 · Domain · relates to 0019

That includes a Stream already deleted: the Store keeps no record of it.
This replaces the earlier rule that a repeated delete succeeds.

- **Why:** that rule could only mean "any unknown id succeeds", typos
  included. The error says exactly what the Store knows, matches every other
  operation, and needs no state.
- **Cost:** a retried delete whose reply was lost gets an error. The
  Application treats it as the Stream being gone, as with 0019.
- **Alternatives:** succeeding on any unknown id — the typo problem.
  Tombstones — they grow with every delete, and Create would have to clear
  them. Replication may need them anyway, to carry deletes between Nodes and
  so a recreated Stream doesn't restart its vector clock at `{node-a:1}`;
  decide them there.

<a id="0024"></a>
## 0024 — Retention: protecting a Stream from deletion

Undecided · 2026-09-25 · Domain · relates to 0023

Some Streams must never be deleted, like a bank's transaction history; others
should be, like keystrokes once summarized. Today any closed Stream can be
deleted, and keeping one is up to the Application.

- **Why wait:** until there is a public API, only the Application's own code
  can delete.
- **Options:** a policy fixed at creation, deletable or permanent, so every
  Node agrees on it under replication. Or a retention period after
  `closed_at`, since the law can require both a minimum and erasure. Decide
  it with tombstones (0023).

<a id="0025"></a>
## 0025 — Read Stream's function is `get_events`

Accepted · 2026-09-26 · Technical · relates to 0013, 0017

The feature keeps its name, Read Stream; its tests live in
`tests/get_events.rs`.

- **Why:** `get_stream` and `get_events` pair up, each naming the half of a
  Stream it returns. Beside `get_stream`, `read_stream` sounds like a second
  way to get the same thing. This records the rename made in 195aa05.
- **Cost:** the function's name no longer matches the feature's.
- **Alternatives:** going back to `read_stream`.

<a id="0026"></a>
## 0026 — A feature is specified per Stream state, in the test notation

Accepted · 2026-09-27 · Technical · relates to 0010, 0011

Each feature has a section per state a Stream can be in — absent, open,
closed — with its rules and examples under each. An example's `Given` and
`Then` are written in the notation `parse_store` and `pprint_store` use,
copied from its test, so it pastes straight into a test. Every Event in the
notation has a `metadata` line, `{}` when empty.

- **Why:** the states are what every rule turns on, so each feature reads the
  same way and a missing case shows as an empty section. One notation means
  the spec and the tests can't describe the same state differently.
- **Cost:** examples are copied from tests by hand, and nothing checks they
  stay in step.
- **Alternatives:** a state × operation table — needs a rendering step; the
  spec should read as plain text. Examples in their own prose notation, as
  before — they can't be pasted into a test or checked against one.

<a id="0027"></a>
## 0027 — Rule ids are retired, never reused

Accepted · 2026-09-27 · Technical · relates to 0017, 0026

A rule id like `AppendEvent.04` names one rule forever. A rule that is merged
or dropped leaves a gap: `AppendEvent.04` merged into `.03`, `.10` into `.05`,
and `.08` dropped, since it described the Application, not the Store.

- **Why:** commits, plans and decisions cite ids. Reusing one would make an
  old reference point at a different rule.
- **Cost:** gaps in the numbering.

<a id="0028"></a>
## 0028 — Docs are split by how long they stay true

Accepted · 2026-09-30 · Technical · relates to 0026

Under `docs/`: `spec/` says what Distolocal does now, one file per component
plus an index; `decisions.md` says why, append-only; `plans/` says how we get
there, one file per slice, kept as history once done, with the backlog in
their index. References point only toward the longer-lived: plans cite the
spec and decisions, the spec cites decisions, and neither cites a plan. When a
plan finishes, what should outlive it moves into the spec or here.

- **Why:** finished plans sat in the root beside the spec, holding the
  backlog and step 14's decisions, and went stale. The spec carried
  rationale in its schema comments and placeholder sections.
- **Cost:** old commits and plans name the files as they were.
- **Alternatives:** one file per decision — the index works at this size.
  Deleting finished plans — their steps explain how the code got its shape.
