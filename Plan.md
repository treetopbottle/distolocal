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
  field), `event_type`, `timestamp`, and opaque `data`/`metadata: Vec<u8>`
  (per DECISIONS.md 0007 — raw bytes, not the `serde_json::Value` originally
  proposed here).
- `Stream`: `stream_id`, `status` (`Open`/`Closed`), `created_at`.
- `Error`: the five variants (`StreamNotFound`, `StreamClosed`,
  `StreamNotClosed`, `EventIdConflict`, `InvalidEvent`), each carrying the
  ids needed to describe the failure.

No Store yet.

### Step 3 — Create Stream: new and idempotent-open

- **3a (tests):** `CreateStream.01` (create a Stream that doesn't exist),
  `CreateStream.02` (create an already-open Stream again → returns it
  unchanged). This also proposes the `Store` shape (in-memory: a map from
  `stream_id` to its `Stream` + ordered `Vec<Event>`) via the test setup/API
  calls used.
  → **checkpoint.**
- **3b (implementation):** introduce `Store`, implement `create_stream`.

### Step 4 — Append Event: ordering and StreamNotFound

- **4a (tests):** `AppendEvent.01`–`.04` (events land at the end, in order,
  with the Store incrementing each Event's `vector_clock` entry for its own
  node), `AppendEvent.09`
  (`StreamNotFound` when the Stream was never created).
  Note: `AppendEvent.08` needs no dedicated test — it documents an
  application-level habit (don't re-create a Stream you already created)
  already covered by `CreateStream.02`'s idempotency.
  → **checkpoint.**
- **4b (implementation):** implement `append_event`.

**Commits:** "Add tests for AppendEvent: ordering and StreamNotFound" →
"Implement AppendEvent: ordering and StreamNotFound"

### Step 5 — Append Event: idempotency and validation

- **5a (tests):** `AppendEvent.05` (retry with the same `event_id` appends
  once), `AppendEvent.10` (`EventIdConflict` when `event_id` is reused with
  different data), `AppendEvent.11` (`InvalidEvent` when a required field is
  missing), `AppendEvent.07` (locks in that `data` is opaque — no app-level
  validation).
  → **checkpoint.**
- **5b (implementation):** add idempotency-by-`event_id` and shape validation
  to `append_event`.

### Step 6 — Read Stream

- **6a (tests):** `ReadStream.01` (returns events in append order),
  `ReadStream.02` (empty Stream reads as `[]`), `ReadStream.04`
  (`StreamNotFound` for a Stream that was never created).
  → **checkpoint.**
- **6b (implementation):** implement `read_stream`.

### Step 7 — Close Stream

- **7a (tests):** `CloseStream.01` (closing an already-closed Stream is a
  no-op), `CloseStream.02` (open → closed, Events unchanged), `CloseStream.03`
  (`StreamNotFound` for a Stream that was never created).
  → **checkpoint.**
- **7b (implementation):** implement `close_stream`.

### Step 8 — Enforce StreamClosed across Append/Read/Create

Now that Close Stream exists, wire up the closed-Stream rules that depend on
it:

- **8a (tests):** `AppendEvent.06` (`StreamClosed` on append to a closed
  Stream), `ReadStream.03` (reading a closed Stream still returns its Events),
  `CreateStream.03` (`StreamClosed` when Create targets an id whose Stream is
  closed — Create is not a way to reopen).
  → **checkpoint.**
- **8b (implementation):** add the closed-Stream checks to
  `append_event`/`read_stream`/`create_stream`.

### Step 9 — Delete Stream

- **9a (tests):** `DeleteStream.01` (`StreamNotClosed` when deleting an open
  Stream), `DeleteStream.02` (deleting a closed Stream removes it and its
  Events), `DeleteStream.04` (idempotent — deleting an already-deleted Stream
  succeeds).
  → **checkpoint.**
- **9b (implementation):** implement `delete_stream`.

### Step 10 — Delete/Create interplay across Streams

- **10a (tests):** `DeleteStream.03` (deleting one Stream leaves an unrelated
  Stream, e.g. a summary Stream, untouched), `CreateStream.04` (recreating a
  previously-deleted id starts a fresh, empty Stream — delete doesn't retire
  the id).
  → **checkpoint.**
- **10b (implementation):** fix up anything `9b` didn't already cover (this
  step should mostly just confirm existing behavior — a good sign if 10b ends
  up empty).

### Step 11 — End-to-end narrative test

- **11a (tests):** one integration test walking through the full "Annabel and
  the Todo Application" narrative from SPECIFICATION.md (Lists → Chores →
  ChoresHistory), written against the public Store API only, as living
  documentation.
  → **checkpoint.**
- **11b:** this test should already pass against steps 1–10's implementation
  — if it doesn't, that's a sign a rule was missed earlier, not a new
  feature to build.

---

## After this plan

Once this lands, Event and Stream management (v1, single-writer, in-memory) is
feature-complete against SPECIFICATION.md. Next slices, not covered here:

- Persistence (currently in-memory only).
- Stream subscription (push/hooks) — builds on this.
- Node replication, and the still-open questions in DECISIONS.md 0003/0006
  (vector clocks, concurrent writers to the same Stream at the same Node).
- A public API protocol (JSON first; gRPC/Avro anticipated — see
  DECISIONS.md 0007). Today the only "API" is the Store's Rust function
  calls used directly by these tests.

Don't start those until this slice is reviewed and merged.
