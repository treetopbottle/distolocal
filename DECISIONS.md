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
  across many inputs — e.g. the closed-Stream rules in Plan.md step 10.

## 0009 — Given/When/Then lives in comments, not in a test DSL

Superseded by 0017 · 2026-09-19 · Technical · relates to 0008

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
  as well as successes. It closes the drift gap above; revisit if the step 13
  narrative test gets unwieldy. The `cucumber` crate (real `.feature` files)
  was rejected as too much machinery, and it would duplicate the spec text
  unless SPECIFICATION.md became the source of those files.
- **Partly addressed by 0011:** the `given()` parser closes this drift gap on
  the `Given` side. The `When` and `Then` comments stay unenforced.

## 0010 — Approval testing for Stream contents

Accepted · 2026-09-19 · Technical · relates to 0009, 0011, 0012

From Plan.md step 7 on, a test's `Then` is an inline `insta` snapshot of the
Stream's rendered state rather than a list of field assertions.

- **Why:** one snapshot covers ordering, vector clocks and payloads at once,
  and reads close to SPECIFICATION.md's own `[TodoCreated#1, TodoCreated#2]`
  notation. Field assertions fragment that into individually-passing checks
  while the state as a whole goes unreviewed.
- **Cost:** a snapshot is easy to update without reading it. Countered by
  Plan.md's working agreement: snapshots are hand-written at the (a) tests
  step from what the spec says the state should be, and `cargo insta review`
  is only for deliberate updates to an already-approved snapshot — never for
  filling in a blank. Recording and eyeballing would be approval after the
  fact, hollowing out the checkpoint.
- **Exception:** `read_stream`'s own tests keep direct field assertions, since
  the dump is built on `read_stream` and snapshotting them would be circular.
- **Narrowed 2026-09-24:** the ReadStream and GetStream tests now snapshot the
  value the call returns, rendered by `pprint_events` and `pprint_stream`.
  Those only format what they are given and never read the Store, so the check
  is not circular. What stays excluded is snapshotting `pprint_store` in those
  tests, since it is built on both calls. The snapshots leave out `event_id`,
  which the notation does not render and the field assertions used to compare.
- **Alternatives:** `expect-test` (inline only, `UPDATE_EXPECT=1`) — `insta`
  picked for its `.snap` files, redactions and review workflow, should those
  be wanted later. Keeping field assertions throughout — rejected as
  unreadable by the step 13 narrative test.

## 0011 — One test-only notation for both Given and Then

Accepted · 2026-09-19 · Technical · relates to 0008, 0009, 0010

Tests render and construct Stream state through a single text notation:
`dump(&store, stream_id)` renders it, and `given(text)` parses the same
notation and replays it through the public `create_stream`/`append_event`
API.

- **Why:** the reader compares the before and after state in one shape
  instead of translating between setup calls and expected output. Replaying
  through the public API also means `given()` can assert the Store really
  assigns the vector clocks the text claims, so a `Given` cannot describe a
  state the Store would never produce.
- **Cost:** the notation is a second description of the Event schema, to be
  kept in step with the real one.
- **Scope:** test-only. It is not a serialization, import or export format,
  and it does not preempt the persistence slice — building the real export
  format now would let test ergonomics shape a production format ahead of
  that decision. Revisit only as a deliberate choice to dogfood a real
  format.

## 0012 — The Store takes its clock as a dependency

Accepted · 2026-09-19 · Technical · relates to 0010

The Store holds a `Clock` — `SystemClock` in production, a stepping clock in
tests — instead of calling `OffsetDateTime::now_utc()` directly.

- **Why:** `timestamp` and `created_at` are Store-assigned (SPECIFICATION.md
  Event schema), so they belong in the dump, but a snapshot of wall-clock
  output can never be stable. Injection makes them deterministic and, for the
  first time, assertable at all.
- **Cost:** a production design change made for test reasons, and `Store::new`
  grows an argument. Accepted because an injectable clock is independently
  reasonable for a Node that will later have to reason about time, and the
  alternative hides a field the Store is responsible for.
- **Alternatives:** `insta` redactions or filters to blank the timestamps, or
  leaving them out of the notation entirely. Both keep the Store's own
  clock behavior untested.

## 0013 — `read_stream` returns borrowed Events, not owned copies

Accepted · 2026-09-21 · Technical · relates to 0007

`Store::read_stream(&self, stream_id) -> Result<&[Event], Error>` hands out a
slice into the Store's own storage. A caller that needs its own copy — to
keep it across an append, send it to another thread, or outlive the Store —
writes `.to_vec()`.

- **Why:** the Events are in the Store; the common read folds them into
  Application state and drops them immediately, so an owned `Vec` would clone
  every `data`/`metadata` buffer for nothing. The caller decides whether a
  copy is worth it, as `HashMap::get` and friends do. The borrow also stops a
  read from being held across an append, which the borrow checker is right to
  reject.
- **Cost:** a read can't be held across a mutation without `.to_vec()`.
- **Alternatives:** returning `Vec<Event>` — rejected as an allocation on
  every read to pre-pay for a design decision persistence will make anyway.

## 0014 — An Event's Store-assigned time is `created_at`, not `timestamp`

Accepted · 2026-09-21 · Technical · relates to 0003, 0012

The Event schema's `timestamp` is renamed `created_at`, matching the field a
Stream already has. Both mean the same thing: the moment the Store recorded
this thing, read from its own clock.

- **Why:** `timestamp` says a time is present, not which time it is. Once
  Events replicate, an Event has two times worth keeping — when the origin
  Node created it, and when this Node received it — and `created_at` /
  `received_at` name them, where `timestamp` would have to be redefined.
  `created_at` also pairs with `vector_clock` as "when and where this Event
  was created", which is how this specification already describes the two.
- **Cost:** a schema rename, cheap now (no wire protocol, two call sites) and
  expensive later. Test names grow a `stream_`/`event_` prefix to say which
  `created_at` they mean.
- **Alternatives:** keeping `timestamp` as the conventional event-store field
  name — rejected because the convention buys nothing once a second time
  exists, and the ambiguity lands exactly where replication is hardest.

## 0015 — A `Given`'s times drive the Store's clock

Accepted · 2026-09-21 · Technical · relates to 0011, 0012 · its mechanism
superseded by 0022

`given(text)` builds the Store with a clock that reads out the times the text
names, in order, and then carries on stepping from the last one. The vector
clocks the text claims are still asserted against what the Store computes.

- **Why:** a Node has one clock, shared by every Stream on it (see 0012), so
  an Event's `created_at` depends on every earlier reading in the test, not
  just on its own Stream. Asserting the times would tie each `Given` block to
  whatever preceded it, and a block copied out of a dump would only work in
  the position it came from. Time is an input to the Store; the vector clock
  is derived from it — `given()` supplies the input and checks the
  derivation, which is what 0011 is really after.
- **Cost:** the times in a `Given` are checked against nothing, so a block
  can claim an implausible one — an Event before the Stream that holds it. A
  `Then` still renders what the Store really produced.
- **Alternatives:** times optional in a `Given`, asserted when present —
  rejected for the copied-block problem above. A clock per Stream, so each
  one counts from its own creation — rejected: a Node has one clock, and
  pretending otherwise would hide in the test harness exactly what
  replication has to reason about.


## 0016 — `dump` renders a whole Store, on a test-only `stream_ids`

Accepted · 2026-09-23 · Technical · relates to 0010, 0011

`dump(&store)` renders every Stream the Store holds — one block per Stream,
in the order they were created — where 0011's `dump(&store, stream_id)`
rendered one. It reaches them through `Store::stream_ids()`, a `#[doc(hidden)]`
accessor that exists for the harness and has no `FeatureRule` behind it.

- **Why:** a `Then` should show the state the `When` left behind, not the one
  Stream the test remembered to ask for. An append that touches the wrong
  Stream, or a later Delete Stream that takes a neighbour with it
  (`DeleteStream.03`), is invisible to a per-Stream dump and obvious in a
  whole-Store one. The notation was already built for it: headers sit at
  column 0 so blocks concatenate (Plan.md step 7a), and `given` has read
  several blocks since 7c — so this makes the two halves symmetrical, which
  is what 0011 is for.
- **Cost:** a public accessor added for test reasons, and one the Store's own
  features never call. `#[doc(hidden)]` keeps it out of the docs but not out
  of reach. Listing Streams is deliberately *not* a feature: an Application
  that wants to know which Streams exist keeps a catalog Stream of its own —
  the "Lists" Stream in SPECIFICATION.md's narrative — and adding a real
  List Streams feature would contradict that design to serve a test harness.
  Contrast 7b, where Get Stream was specified as a feature because an
  Application genuinely needs it.
- **Ordering:** by `created_at`, with the id settling the ties a non-stepping
  clock can produce — so a dump reads in the order the Application wrote it
  and a `Given` copied out of one describes its Streams in that same order.
  Sorting by id instead — rejected: it would reorder the blocks out of the
  `Given` they came from, so a reader could no longer diff a before against
  an after block for block.
- **Alternatives:** a `test-support` Cargo feature gating the accessor, so it
  is compiled out of the production library — rejected as a Cargo feature and
  a self-referencing dev-dependency to buy what a comment already says.
  Tracking creation order inside the Store, rather than sorting on the way
  out — rejected: a second copy of the Stream index for `delete_stream`
  (step 11) to keep in step, for a test's benefit.

## 0017 — A test says what it covers in its name, not in comments

Accepted · 2026-09-23 · Technical · supersedes 0009 · relates to 0010, 0011

A test carries no `FeatureRule.NN` id and no Given/When/Then comments. Its
name gives the function and the situation — `<function>_<situation>`, as in
`append_event_to_stream_that_does_not_exist` — and its body is plain
arrange/act/assert. Where there is state to show, the `Given` and `Then` are
written in the notation (0010, 0011) through `parse_store` and
`pprint_store`, formerly `given` and `dump`. The spec's own wording lives only
in SPECIFICATION.md. Tests of the test harness stay general: one test per
behavior, not one per corner case.

One exception: the end-to-end narrative test (`tests/narrative.rs`) is
living documentation of the spec's narrative, so its comments retell that
story, one per part, above the calls that act it out.

- **Why:** since 0011 a test's `Given` and `Then` are text the Store checks,
  so the comments repeated what the code already showed — and could drift
  from it, the gap 0009 accepted. Comments that name a Plan.md step or a
  DECISIONS.md number record where a test came from, not what it checks, and
  they go stale: the `dump`/`given` renames left several behind. The harness
  is internal to the tests, so a general test of each behavior is enough.
- **Cost:** nothing ties a test to its `FeatureRule` id any more, so finding
  the test for `AppendEvent.04`, or noticing a rule with no test, means
  reading test names rather than searching for the id. A step's checkpoint is
  where its ids are matched to tests.
- **Alternatives:** 0009's comments, kept or cut down to a bare
  `// AppendEvent.04` id line. 0009 already names the `World` DSL and
  `cucumber` as the heavier options.

## 0018 — Event shape is validated by the API adapters, not the Store

Accepted · 2026-09-25 · Technical · relates to 0007

`InvalidEvent` — an Event missing a required field, or holding one of the
wrong type or format — is raised by the API adapter (HTTP, gRPC, …) that
decodes the request, before anything reaches the Store. The Store's own API
takes a typed `PendingEvent`, and does not check its shape again.

- **Why:** a missing field or a wrongly typed one only exists in untyped
  input. Through the Rust API every field is present and typed, so the Store
  would be checking what the compiler already guarantees. Testing it there
  meant inventing a stand-in — an empty `event_type` for "no `type` field" —
  and a stand-in is a rule of its own, not the one AppendEvent.11 states.
  Each adapter already has to decode its protocol's encoding (0007), so it is
  where a malformed request is first seen.
- **Cost:** every adapter must validate shape itself, with no shared check in
  the core to fall back on. `AppendEvent.11` has no test until the public API
  slice, and the Store accepts an empty `event_type` or `event_id` as given.
- **Alternatives:** the Store rejecting empty strings as "missing" — rejected
  as the stand-in above. Optional fields on `PendingEvent`, so a missing one
  is expressible — rejected as making every caller of the typed API handle a
  case only wire input can produce.

## 0019 — The Store checks `event_id` uniqueness, not Event equality

Accepted · 2026-09-25 · Technical · relates to 0007

An Append whose `event_id` is already used in the target Stream fails with
`EventIdConflict`, whatever the new Event holds. The error carries the Event
already stored. The Store compares `event_id`s and nothing else, so a retry
of an append that landed and an id reused for a different Event get the same
answer; telling them apart is the Application's job, from the stored Event.
An HTTP adapter maps the error to 409 Conflict.

- **Why:** deciding that two Events are "the same" is harder than it looks.
  `data` and `metadata` are opaque bytes (0007), and a client that re-encodes
  JSON on a retry can change key order, turning a genuine retry into a
  conflict. The Application can decode its own payloads; the Store cannot.
  The uniqueness check stays in the Store, though: without it a retry appends
  a duplicate, and the Application's only defence would be reading the Stream
  before every append.
- **Cost:** idempotency is no longer free — every Application has to handle
  `EventIdConflict` on a retry and treat it as success. The error grows from
  two ids to a whole Event.
- **Alternatives:** the Store comparing the rest of the Event and succeeding
  on an exact retry — the spec's original rule, rejected for the encoding
  problem above. No `event_id` check in the Store at all, leaving it to the
  Application or to Node replication — rejected because an interrupted write
  is between an Application and its own Node, which replication never sees.
  Replication will need its own dedup by `event_id`, and can build on this
  check.


## 0020 — A Stream records when it was closed

Accepted · 2026-09-25 · Technical · relates to 0012, 0015 · its representation
superseded by 0021

A Stream carries `closed_at`, the time Close Stream closed it: `null` while
it is open, and set by the Store from its clock when it closes. Closing an
already-closed Stream leaves it as it was, so a Stream keeps the time of the
first close. In the test notation a closed Stream's header reads as a
timeline, `"Chores" open <created_at> closed <closed_at>`, so an open
Stream's header is unchanged.

- **Why:** an Application that cleans up closed Streams after a while, as
  the Todo app does a week after closing "Chores", needs to know when each
  one closed. Without the field it has to record that itself, in another
  Stream, for something the Store already knows.
- **Cost:** `status` and `closed_at` say the same thing twice — a Stream is
  closed exactly when `closed_at` is set — and the Store has to keep the two
  in step. A close now reads the clock, so it takes a time from every reading
  after it, as an append does.
- **Alternatives:** a `closed` status that carries the time
  (`StreamStatus::Closed { closed_at }`), which makes a mismatch impossible to
  write — not taken for now, since the nullable field matches the schema one
  to one. No `closed_at`, leaving it to the Application — rejected for the
  reason above. A header of the status and both times, which leaves the reader
  guessing which time is which.

## 0021 — A closed status carries its `closed_at`

Accepted · 2026-09-25 · Technical · supersedes 0020's representation

`closed_at` moves from a field of `Stream` into its status:
`StreamStatus::Closed { closed_at }`, beside a bare `StreamStatus::Open`. What
0020 decided about the time stands — set from the Store's clock by the first
close, kept by any later one — and SPECIFICATION.md's schema keeps `status`
and a nullable `closed_at` as two fields.

- **Why:** with a separate `closed_at`, `status` and `closed_at` said the same
  thing twice, and the Store had to keep them in step; the test notation
  needed a panic for a Stream where they disagreed. Carrying the time in the
  status makes that mismatch impossible to write. The one-to-one match with
  the schema that 0020 kept the field for bought nothing: nothing serializes a
  `Stream` yet.
- **Cost:** the Rust type no longer mirrors the schema, so whatever
  serializes a Stream has to split the status into the two schema fields.
  Reading the time means matching on the status rather than reading a field.
- **Alternatives:** keep 0020's two fields. Drop `status` and derive it from
  `closed_at` (an `is_closed()` method), which also removes the mismatch but
  leaves the status the spec talks about as a derived idea, and a further
  state as another nullable field.

## 0022 — A `Given` sets the clock to each of its times

Accepted · 2026-09-25 · Technical · supersedes 0015's mechanism · relates to 0012

`parse_store` sets the test clock to each time in the text just before the
call that records it, and after the text ends the clock steps on from the
last one. One test clock, `SteppingClock`, does both: it steps per reading,
and `set_next` sets what the next reading gives. What 0015 decided stands: a
`Given`'s times are inputs, and its vector clocks are checked.

- **Why:** 0015's clock read out the times as a script, one per reading, so a
  `Given` only replayed right if every call read the clock exactly as often
  as it recorded a time. That made the number of readings — whether a refused
  or a repeated call reads the clock — something the tests pinned down and
  the docs kept restating, though the Store promises nothing about it; only
  the times it records are behavior. Setting the time before each call makes
  what earlier calls read irrelevant.
- **Cost:** a call that read the clock more than once before recording would
  still use up the time set for it. With the real clock that call would mix
  two moments, so it is not a case to design for. A test that steps through
  calls made in a row, on `store()` or `store_with`, still counts on each of
  them reading the clock once; that is the stepping those tests show, and a
  `Given` is the way to write times down instead. Replaying needs a handle on
  the clock the Store holds, so the clock keeps its time behind an `Rc`.
- **Alternatives:** freezing the clock at each time, so any number of
  readings gives it, then resuming stepping after the text — tried, and
  dropped: a two-state clock and a resume call to cover only the multi-read
  case above. A clock that is only ever set — the tests that make calls in a
  row would have to set it before each call. Keeping a separate
  `ScriptedClock` beside `SteppingClock` — two clocks for what is one.

## 0023 — Deleting a Stream that doesn't exist is `StreamNotFound`

Accepted · 2026-09-25 · Domain · relates to 0019

Delete Stream on an id the Store doesn't hold fails with `StreamNotFound`,
as Append, Read, Get and Close do. That includes a Stream already deleted:
the Store keeps no record of a deleted Stream, so it can't tell one from an
id that was never created. This replaces the spec's earlier rule that
deleting an already-deleted Stream succeeds.

- **Why:** that rule could only ever be "deleting any unknown id succeeds" —
  a typo in an id included — because nothing distinguishes the deleted id.
  Close can be idempotent and still report `StreamNotFound` only because a
  closed Stream is still stored. The error says exactly what the Store
  knows, keeps Delete in line with every other operation, and needs no
  state.
- **Cost:** a retried delete whose first reply was lost gets an error, not
  a success. The Application treats `StreamNotFound` from a delete as the
  Stream being gone, as it treats an `EventIdConflict` on a retried append
  (0019).
- **Alternatives:** succeed on any unknown id, the earlier rule — rejected
  for the typo above, and for making Delete the one operation that accepts
  an id nobody created. Remember deleted ids (tombstones), so a deleted id
  succeeds and an unknown one fails — rejected for now: the record grows with
  every delete, which works against deleting to reclaim space, and Create
  would have to clear it to recreate an id (CreateStream.04). Replication may
  need tombstones to carry deletes between Nodes; that is the place to
  decide them, on what it needs. It may also need them so a recreated id
  does not restart its vector clock: today the first Event of a recreated
  Stream gets `{node-a:1}`, the same clock as the deleted Stream's first
  Event.

## 0024 — Retention: protecting a Stream from deletion

Undecided · 2026-09-25 · Domain · relates to 0023

Some Streams should never be deleted, e.g. a bank account's transaction
history, while others are meant to be, e.g. keystrokes once their text is
summarized. Today any closed Stream can be deleted, and keeping one is up to
the Application. A retention policy in the Store may come later, but not in
this slice.

- **Why:** until there is a public API, only the Application's own code can
  delete, so a Store-level guard would protect against very little. It
  matters once other callers can reach a Node.
- **Open options:** a policy set when the Stream is created and never
  changed (so every Node agrees on it under replication), e.g. deletable or
  permanent; or a retention period, e.g. N years after `closed_at`, since
  "never" is often a legal minimum and privacy law can require erasure.
  Decide it alongside tombstones (0023): a Stream that can't be deleted
  never needs one.
