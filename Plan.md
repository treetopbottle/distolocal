# Plan: Event and Stream management (v1)

Scope: the first vertical slice from README's "Core use cases" — Event and
Stream management only (Create/Append/Read/Close/Delete Stream), in-memory,
single process. Stream subscription and Node replication come later, once this
slice is solid.

Language: Rust (resolves DECISIONS.md 0004 for now — see step 1). Single
crate, in-memory store to start; persistence and the standalone-Node process
are separate, later concerns.

## Working agreement

- **Tests before implementation, and a checkpoint in between.** From step 2
  onward, each step is split into an **(a) tests** commit and a
  **(b) implementation** commit:
  1. Write the test(s) for the step's `FeatureRule.NN` id(s) — they won't
     compile or will fail, since the behavior doesn't exist yet.
  2. **Stop and show you the test code before writing any implementation.**
     You can steer — rename things, add/drop a case, change an assertion —
     while it's still cheap to change.
  3. Once you've okayed it, commit the tests, then implement the minimal code
     to pass them, then commit the implementation separately.
- **One step = two small commits** (tests, then implementation), so a review
  never spans both "what's being tested" and "how it's implemented" at once.
- **Snapshots are written, not recorded.** From step 7 on, a step's `Then` is
  an inline `insta` snapshot. It is hand-written at (a) from what
  SPECIFICATION.md says the state should be, and fails until the
  implementation matches. `cargo insta review` is for deliberate updates to
  an already-approved snapshot — never for filling in a blank left at (a).
  Recording a snapshot from a run and eyeballing it is approval *after* the
  fact, which would make the checkpoint meaningless.
- **Tests carry no spec comments** (DECISIONS.md 0017). A test is named
  `<function>_<situation>`, with no `FeatureRule` id or Given/When/Then
  comments. At the checkpoint, the step's ids are matched to its tests by
  name.
- **The dump is test-only.** It exists to make `Given` and `Then` readable.
  It is not a serialization, import or export format, and nothing in it
  commits us to one — persistence stays a later slice (see "After this
  plan").
- **If the design doesn't hold up while implementing a step** (a schema field
  doesn't fit, a rule is ambiguous, an error case is missing) — stop, don't
  route around it. Flag it, update SPECIFICATION.md/DECISIONS.md, and adjust
  this plan before continuing. Recording that pivot is more valuable than
  pushing through.
- Steps are ordered by real dependency (e.g. "closed Stream" behaviors come
  after Close Stream exists), not strictly by the feature order in
  SPECIFICATION.md.

---

### Step 1 — Scaffold the Rust crate; record the language decision [DONE]

No behavior to test yet, so this step is a single commit:

- Update DECISIONS.md 0004 from `Undecided` to `Accepted`: Rust, single
  library crate for now (`cargo new --lib distolocal`); standalone-binary /
  embeddable-library split deferred until Node replication needs it.
- `cargo init`, empty `src/lib.rs`, one placeholder test to prove
  `cargo test` runs in this environment.
- No production code yet.

### Step 2 — Domain types: Event, Stream, Error [DONE]

No behavior or invariants yet — `Event`/`Stream`/`Error` are plain data with
no methods, so a construction test would only check what `cargo build`
already verifies for free. Skipped the tests split for this step:

- `Event`: `event_id`, `stream_id`, `vector_clock: HashMap<node_id, u64>`
  (adopted directly per DECISIONS.md 0003, in place of a `sequence_number`
  field), `event_type`, `created_at` (named `timestamp` here originally;
  renamed at step 6 per DECISIONS.md 0014), and opaque
  `data`/`metadata: Vec<u8>`
  (per DECISIONS.md 0007 — raw bytes, not the `serde_json::Value` originally
  proposed here).
- `Stream`: `stream_id`, `status` (`Open`/`Closed`), `created_at`.
- `Error`: the five variants (`StreamNotFound`, `StreamClosed`,
  `StreamNotClosed`, `EventIdConflict`, `InvalidEvent`), each carrying the
  ids needed to describe the failure.

No Store yet.

### Step 3 — Create Stream: new and idempotent-open [DONE]

- **3a (tests):** `CreateStream.01` (create a Stream that doesn't exist),
  `CreateStream.02` (create an already-open Stream again → returns it
  unchanged). This also proposes the `Store` shape (in-memory: a map from
  `stream_id` to its `Stream` + ordered `Vec<Event>`) via the test setup/API
  calls used.
  → **checkpoint.**
- **3b (implementation):** introduce `Store`, implement `create_stream`.

### Step 4 — Append Event: ordering and StreamNotFound [DONE]

- **4a (tests) [DONE]:** `AppendEvent.01`–`.04` (events land at the end, in
  order, with the Store incrementing each Event's `vector_clock` entry for
  its own node), `AppendEvent.09`
  (`StreamNotFound` when the Stream was never created).
  Note: `AppendEvent.08` needs no dedicated test — it documents an
  application-level habit (don't re-create a Stream you already created)
  already covered by `CreateStream.02`'s idempotency.
  → **checkpoint.**
- **4b (implementation):** implement `append_event`.

**Commits:** "Add tests for AppendEvent: ordering and StreamNotFound" →
"Implement AppendEvent: ordering and StreamNotFound"

### Step 5 — Read Stream [DONE]

Moved ahead of the remaining Append rules: nothing can observe a Stream's
contents until `read_stream` exists, so both the later steps' `Then`s and the
step 7 dump depend on it.

- **5a (tests) [DONE]:** `ReadStream.01` (returns events in append order),
  `ReadStream.02` (empty Stream reads as `[]`), `ReadStream.04`
  (`StreamNotFound` for a Stream that was never created).
  These keep direct field assertions permanently — the step 7 dump is built
  *on* `read_stream`, so snapshotting its own tests would be circular.
  → **checkpoint.**
- **5b (implementation):** implement `read_stream`, returning a borrowed
  `&[Event]` rather than owned copies (DECISIONS.md 0013).

### Step 6 — Inject the clock into the Store [DONE]

Not a `FeatureRule` — a design change that approval testing needs first. A
dump carrying `FormattedDateTime::now()` output can never be stable, and
today a Stream's and an Event's `created_at` are unobservable in tests for
the same reason. Dependency injection fixes both, rather than redacting the
values out of the snapshots afterwards.

- **6a (tests) [DONE]:** a Store built with a fixed clock gives a known
  `created_at` to a Stream it creates and to an Event it appends; the
  stepping test clock advances a fixed interval per call, so Events appended
  in order carry increasing `created_at`s. Both clocks take their start (and
  the stepping one its interval) as parameters, with the shared constants as
  defaults.
  → **checkpoint.**
- **6b (implementation) [DONE]:** a `Clock` trait the Store holds —
  `SystemClock` in production, a stepping clock in `tests/common`.
  `Store::new` grows a clock argument alongside the node id, boxed rather
  than a `Store<C: Clock>` type parameter. Renaming the Event's `timestamp`
  to `created_at` came out of this step — see DECISIONS.md 0014.

### Step 7 — Approval-testing harness: one notation for Given and Then [DONE]

- **7a (the notation) [DONE]:** a Stream header line — quoted id, status,
  `created_at` — then one line per Event (`created_at`, type, vector clock)
  indented under it, with the data and metadata indented further again:

  ```
  "chores-3f2a1c" open 2026-01-01T00:00:02Z
    2026-01-01T00:00:03Z TodoCreated {node-a:1}
      data {"todo_id":1,"title":"Take out the trash"}
      metadata {}
    2026-01-01T00:00:04Z TodoFinished {node-a:2}
      data {"todo_id":1}
      metadata {"schema_version":"1.7.2"}
  ```

  The time leads each Event line because it is always the same width, so the
  Event type lines up in a column under each other. Stream ids are quoted
  because the Application chooses them and nothing stops one holding a space;
  Event types are identifiers, so they stay bare.
  Vector clocks render as the map they are, sorted by node id so `HashMap`
  iteration order can't reorder them — today one entry, per DECISIONS.md
  0005. `data` always renders (an Event without it is `InvalidEvent`), and
  so does `metadata` — `{}` when the Event carries none, so every Event is
  the same three lines. `{}` is therefore how the notation writes no
  metadata: it parses back as none, and an Event whose metadata is literally
  `{}` reads the same as one with none. Both payloads take the rest of their
  line, so nothing in them can collide with a field after it. `event_id`
  stays out: it is the Application's idempotency key, not Stream content, and
  the tests that care about it (step 8) pin and assert it directly. An empty
  Stream is its header alone. Headers sit at column 0 with their Events
  indented beneath, so blocks concatenate if a later step wants a whole-Store
  dump. A line says which of the three it is by its own shape — a quoted id,
  a `data`/`metadata` field, or a time — so the indentation is the reader's
  and not the parser's: a `Given` sits at whatever depth the code around it
  does, and a dump goes straight back in as one (settled while building 7c).
  A `Given`'s times drive the Store's clock rather than being asserted
  against it — see DECISIONS.md 0015.
  → **checkpoint.**
- **7b (Get Stream) [DONE]:** the header line needs a Stream's `status` and
  `created_at`, and no Store API hands them over — `read_stream` returns only
  Events, and `create_stream` is a write that will error on a closed Stream
  from step 10 on, which is exactly the case the header exists to show. So
  the missing feature comes first, on its own tests → checkpoint →
  implementation pair: `GetStream.01` (an open Stream's record),
  `GetStream.03` (`StreamNotFound` for a Stream that was never created).
  `GetStream.02` (a closed Stream's record) waits for step 9, the first step
  that can close one. It goes into SPECIFICATION.md as its own feature rather
  than as a test affordance: an Application that can close a Stream should be
  able to ask whether one is open instead of discovering it by failing an
  append. Kept inside step 7 rather than inserted as a step of its own, which
  would renumber every later step out from under the references in
  DECISIONS.md 0010, 0011 and 0015.
  → **checkpoint.**
  Landed as `get_stream`, mirroring `read_stream` — same lookup, same error,
  same borrow (DECISIONS.md 0013). It hands back the record whatever the
  status, which is what makes it the way to ask whether a Stream is still
  open, so step 10 leaves it out of the `StreamClosed` wiring.
- **7c (harness) [DONE]:** `dump(&store, "Chores")` renders that notation via
  `get_stream`/`read_stream`. `given("...")` parses the same notation and
  replays it through `create_stream`/`append_event`, asserting the Store
  assigns the vector clocks the text claims — so a `Given` block cannot
  quietly drift from what the Store would really produce, which is the
  weakness DECISIONS.md 0009 accepts for comment-only Givens. Three things
  came out of building it, beyond the letter of this step:
  - The harness has its own tests, in `tests/notation.rs`. 7d checks `dump`
    against the field assertions the CreateStream and AppendEvent tests carry
    today, but it never calls `given`, which would otherwise land untested.
  - `given` reads several Stream blocks, since headers sit at column 0 for
    exactly that reason, with one clock running across all of them.
  - A `closed` header parses but panics on replay: Close Stream lands at
    step 9, and replaying a closed Stream as an open one would make the test
    lie.
- **7d (conversion) [DONE]:** restate the CreateStream and AppendEvent
  `Then`s as inline `insta` snapshots, keeping the Given/When/Then comments.
  The field assertions those tests had were the check on the harness: the
  snapshots must say the same thing before the assertions come out. The
  ReadStream and GetStream tests keep their field assertions and don't
  convert — the dump is built on both, so snapshotting them would be
  circular (step 5a). Adds `insta` as a dev-dependency.
  Every snapshot was hand-written and run beside the assertions it replaces
  — the whole suite passed twice, once with both and once with the
  assertions gone. Three assertions stayed, each for something a dump does
  not render: `CreateStream.02`'s `second == first` (the record the call
  hands back, not the Store's state), `AppendEvent.01`'s check that the Event
  returned is the one stored (a dump reads the Store, and `event_id` is out
  of the notation by 7a), and `AppendEvent.09`'s `StreamNotFound` (an
  outcome, not a state — DECISIONS.md 0010).
  The `Given`s convert too, wherever there is state to describe: a test that
  starts from an existing Stream now arranges through `given()` text rather
  than `store_with`/`append` calls, so its before and after stand in the same
  notation and a reader diffs one against the other — which is what
  DECISIONS.md 0011 is for. It also writes each time down instead of leaving
  it to be counted off the clock's readings: `AppendEvent.02`'s `Given` shows
  the append to "Lists" that its `Then`'s times otherwise only hint at. A
  `Given` of "no Stream exists" stays a bare `store()` — there is nothing for
  the notation to say. Converting the arrange didn't move the snapshots: .01,
  .03 and .04 pass byte-identical to the versions the field assertions
  checked, which is what says `given()` replays to the state the helper calls
  built. Only `AppendEvent.02`'s Stream header moved, 00:00:01 → 00:00:02,
  because a `Given` replays its Streams block by block where `store_with`
  created both up front.

### Step 8 — Append Event: idempotency and validation

- **8a (tests):** `AppendEvent.05` (retry with the same `event_id` appends
  once), `AppendEvent.10` (`EventIdConflict` when `event_id` is reused with
  different data), `AppendEvent.11` (`InvalidEvent` when a required field is
  missing), `AppendEvent.07` (locks in that `data` is opaque — no app-level
  validation).
  → **checkpoint.**
- **8b (implementation):** add idempotency-by-`event_id` and shape validation
  to `append_event`.

### Step 9 — Close Stream

- **9a (tests):** `CloseStream.01` (closing an already-closed Stream is a
  no-op), `CloseStream.02` (open → closed, Events unchanged), `CloseStream.03`
  (`StreamNotFound` for a Stream that was never created), and `GetStream.02`
  (a closed Stream's record), held back from 7b because nothing could close a
  Stream until now.
  → **checkpoint.**
- **9b (implementation):** implement `close_stream`, and replace the panic a
  `closed` header hits in `given` (7c) with the `close_stream` call that
  replays it.

### Step 10 — Enforce StreamClosed across Append/Read/Create

Now that Close Stream exists, wire up the closed-Stream rules that depend on
it:

- **10a (tests):** `AppendEvent.06` (`StreamClosed` on append to a closed
  Stream), `ReadStream.03` (reading a closed Stream still returns its Events),
  `CreateStream.03` (`StreamClosed` when Create targets an id whose Stream is
  closed — Create is not a way to reopen).
  → **checkpoint.**
- **10b (implementation):** add the closed-Stream checks to
  `append_event`/`read_stream`/`create_stream` — and to nothing else:
  `get_stream` answers whatever the status (7b), which is how an Application
  asks whether a Stream is still open.

### Step 11 — Delete Stream

- **11a (tests):** `DeleteStream.01` (`StreamNotClosed` when deleting an open
  Stream), `DeleteStream.02` (deleting a closed Stream removes it and its
  Events), `DeleteStream.04` (idempotent — deleting an already-deleted Stream
  succeeds).
  → **checkpoint.**
- **11b (implementation):** implement `delete_stream`.

### Step 12 — Delete/Create interplay across Streams

- **12a (tests):** `DeleteStream.03` (deleting one Stream leaves an unrelated
  Stream, e.g. a summary Stream, untouched), `CreateStream.04` (recreating a
  previously-deleted id starts a fresh, empty Stream — delete doesn't retire
  the id).
  → **checkpoint.**
- **12b (implementation):** fix up anything `11b` didn't already cover (this
  step should mostly just confirm existing behavior — a good sign if 12b ends
  up empty).

### Step 13 — End-to-end narrative test

- **13a (tests):** one integration test walking through the full "Annabel and
  the Todo Application" narrative from SPECIFICATION.md (Lists → Chores →
  ChoresHistory), written against the public Store API only, as living
  documentation. With the step 7 harness in place this reads as a dump after
  each beat of the narrative rather than a wall of assertions.
  → **checkpoint.**
- **13b:** this test should already pass against steps 1–12's implementation
  — if it doesn't, that's a sign a rule was missed earlier, not a new
  feature to build.

---

## After this plan

Once this lands, Event and Stream management (v1, single-writer, in-memory) is
feature-complete against SPECIFICATION.md. Next slices, not covered here:

- Persistence (currently in-memory only). The step 7 dump is test-only and
  deliberately not a candidate format for it.
- Stream subscription (push/hooks) — builds on this.
- Node replication, and the still-open questions in DECISIONS.md 0003/0006
  (vector clocks, concurrent writers to the same Stream at the same Node).
- A public API protocol (JSON first; gRPC/Avro anticipated — see
  DECISIONS.md 0007). Today the only "API" is the Store's Rust function
  calls used directly by these tests.

Don't start those until this slice is reviewed and merged.
