# Plan: simplify the remaining code files

Do to the rest of the code what 61a4600 did to `tests/append_event.rs` and
17e5a65 did to `tests/notation.rs`. This is a cleanup only: no behavior
changes and no changes to the public API.

## What "simplify" means here

These rules come from those two commits:

1. **Delete comments that repeat the code, tell its history, or point at
   Plan.md steps, DECISIONS.md numbers or `FeatureRule.NN` ids.** That
   includes every Given/When/Then comment and the `///` doc comment above a
   test.
2. **If a file needs context, give it one short `//!` module doc** that says
   what the file is for, as `tests/notation.rs` now has.
3. **Name a test `<function>_<situation>`, after the input rather than the
   outcome**, e.g. `append_event_to_stream_that_does_not_exist` or
   `pprint_store_empty_stream`.
4. **Order tests with the error case first, then from simplest to fullest.**
5. **Merge tests that overlap.** One general test beats one test per corner
   case, especially for test helpers.
6. **Remove imports and helpers that nothing uses any more.**

Keep the `.expect("...")` messages. Also keep any assertion that checks
something a snapshot can't show, such as a return value compared with what the
Store holds.

## Workflow

Take one step at a time. Each step is one commit, titled
`Simplify <path>` like 17e5a65. For each step:

1. Make the change, then run `cargo fmt`.
2. Check that `cargo test` and `cargo clippy --all-targets` both pass cleanly.
   Both are clean at the start (18 tests).
3. **Show the diff and wait for an OK before committing.**
4. Commit with a pathspec (`git commit -- <path>`), because files sometimes
   arrive already staged.

Steps 4 → 5 → 6 depend on each other in that order. The others can go in any
order.

---

### Step 1: `tests/create_stream.rs`

- Remove both `///` doc comments and all the Given/When/Then comments,
  including the three-line comment above `existing`.
- Rename `create_stream_that_already_exists_is_idempotent` to
  `create_stream_that_already_exists`.
- Change `.expect("given should have created the Stream")` to
  `.expect("the Stream exists")`. `given` no longer exists, and this matches
  `append_event.rs`.
- Keep `assert_eq!(second, existing)`. The snapshot can't show the value the
  call returns.

### Step 2: `tests/read_stream.rs`

- Remove the doc comments and the Given/When/Then comments.
- Rename `read_stream_that_was_never_created` to
  `read_stream_that_does_not_exist` and move it to the top.
- Keep the field assertions rather than snapshots. The dump is built on
  `read_stream`, so a snapshot would test the function against itself (Plan.md
  7d).

### Step 3: `tests/get_stream.rs`

- Remove the doc comments and the Given/When/Then comments.
- Rename `get_stream_that_was_never_created` to
  `get_stream_that_does_not_exist` and move it to the top.
- In `get_stream_that_is_open`, drop the second `append`. One Event is enough
  to show that `created_at` is the Stream's own time, not its latest Event's.

### Step 4: `tests/clock.rs`

- Remove the Plan.md step 6 header, the doc comments and the Given/When/Then
  comments.
- Keep only `every_call_reads_the_clock_again`, renamed to
  `store_reads_clock_on_every_call`. It already shows that both a Stream's
  and an Event's `created_at` come from the Store's clock, which makes the two
  `FixedClock` tests redundant (3 tests become 1).
- Build its Store with `Store::new(NODE_ID, SteppingClock::default())`. The
  clock under test stays visible, and `START` and `STEP` no longer need
  importing.
- Add a one-line `//!` doc, such as: "The Store takes every `created_at` from
  its clock."

### Step 5: `tests/common/mod.rs`

- Delete `FixedClock`, which nothing uses after step 4. Also delete
  `clock(count)`, which nothing has used since 17e5a65.
- Fold `SteppingClock::new` into its `Default` impl, which is now its only
  caller.
- Move `ScriptedClock` into `notation.rs`, since `parse_store` is its only
  user.
- Delete the stale comment above the re-export, which still says
  `dump`/`given`. Keep the `#[allow(unused_imports)]`.
- Trim the doc comments that point elsewhere:
  - Shorten the comment above the narrative Events to one line saying they are
    the Events from SPECIFICATION.md's Todo narrative. Drop the lines about
    `event_id` and Plan.md step 8.
  - Remove "unless the test says otherwise" from `START` and `STEP`.
  - Trim `with_metadata` to a single line.
- Keep the comment on `#![allow(dead_code)]`, because it explains an attribute
  that would otherwise look odd.

### Step 6: `tests/common/notation.rs`

This is the largest file (341 lines). It should shrink by about a third.

- **Header:** replace the 23-line `//` block with a `//!` doc. It should have
  one line saying what the notation is: a test-only text form of a Store that
  `pprint_store` renders and `parse_store` reads back. Then include the
  example block, then one line saying that `metadata {}` means no metadata.
  Drop the Plan.md/DECISIONS.md references and the old names `dump` and
  `given`.
- **Constants:**
  - Inline `EVENT_INDENT` and `PAYLOAD_INDENT` by writing the spaces into the
    format strings, so each format string shows the shape of its line.
  - Inline `HELD` into its two `.expect(...)` calls.
  - Keep `NO_METADATA`, because rendering and parsing must agree on it.
- **Panic messages:** there are about ten bespoke messages, all with a stale
  `"given: "` prefix. Replace them with one helper,
  `fn invalid(text: &str) -> ! { panic!("parse_store: can't read {text:?}") }`,
  called as `.unwrap_or_else(|| invalid(line))` and from the `let ... else`.
  No test depends on the panic text any more.
- **Guards:**
  - Drop the duplicate-Stream check in `parse`. It guards a corner case in a
    test helper, and the vector-clock assert in `replay` already catches a
    repeated block that has Events.
  - Keep the vector-clock assert and the closed-Stream panic in `replay`, as
    one-line messages without the comments around them.
- **Helpers:**
  - Inline `last_stream` and `last_event` into `parse`, since each has only
    one caller.
  - Rewrite `vector_clock` as a single pass: format each `node:count`, sort,
    then join. Sorting the formatted strings is just as deterministic.
- **Comments:**
  - Cut the six-line sorting comment in `pprint_store` down to one line, for
    example `// The Store lists ids in HashMap order; sort by creation, then id.`
  - Remove the other explanatory comments: quoted ids, why the time leads,
    payloads taking the rest of the line, the made-up `event_id`, and `{}`
    parsing back as nothing. The example in the header already shows these.
  - Cut `parse_store`'s doc to two lines: it replays the text through
    `create_stream`/`append_event`, and the text's times drive the clock,
    which keeps stepping after the text ends.
- Add `ScriptedClock`, moved here from step 5, with a one-line doc.

### Step 7: `src/lib.rs`

This is production code, so each `pub` item keeps a one-line `///` doc. The
cross-references and design rationale go.

- **Public method docs:** drop the FeatureRule ids and DECISIONS.md
  references. For example, `create_stream` becomes
  `/// Creates the Stream, or returns it unchanged if it already exists.`
  Handle `get_stream`, `read_stream` and `append_event` the same way.
- **`Clock`:** turn the `//` comment into
  `/// Where the Store gets the times it assigns.`
- **`FormattedDateTime`:** use
  `/// An RFC 3339 time, only built from an `OffsetDateTime`.`
- **`Event` fields:**
  - Give `vector_clock` one line: a count per Node, where a missing Node
    counts as 0.
  - Give `data`/`metadata` one line: opaque bytes, stored as given.
  - Drop the note about a future `received_at` on `created_at`.
- **`Store`:**
  - Drop the comment explaining why `clock` is a `Box<dyn Clock>`.
  - Give `stream` and `stream_mut` one line each. Drop the paragraph on where
    a `StreamClosed` check belongs. If that note is worth keeping, it belongs
    in Plan.md step 10, not in the code.
- **`append_event`:** drop the `let Store { node_id, clock, streams } = self;`
  destructuring. `Self::stream_mut(&mut self.streams, stream_id)?` borrows
  only the `streams` field, so `self.node_id` and `self.clock.now()` stay
  usable. Still read the clock *after* the lookup, so a failed append doesn't
  use up a clock tick.
- **`stream_ids`:** cut the six-line doc to
  `/// Test-only: every Stream id, for pprint_store.` Keep `#[doc(hidden)]`.

### Step 8: `Cargo.toml`

- Shorten the two dev-dependency comments to one line each. Drop the
  DECISIONS.md 0010 reference and the stale `given()`.

### Step 9: docs follow-up

This step isn't code, but the docs will contradict the code without it.
DECISIONS.md 0009 says every test carries its FeatureRule id and
Given/When/Then comments, and Plan.md 7d says "keeping the Given/When/Then
comments". After steps 1–4, no test does either. A future session following
Plan.md step 8 onward would put those comments back.

- Add DECISIONS.md 0017, superseding 0009. Tests are named after the
  situation, the spec wording lives only in SPECIFICATION.md, and test helpers
  get general tests rather than one per corner case.
- Add one bullet to Plan.md's working agreement pointing at 0017, so that
  steps 8 onward follow the new style.

## Out of scope

- The unused `Error` variants (`StreamClosed`, `EventIdConflict`, …). Steps
  8–10 use them.
- Converting the ReadStream and GetStream tests to snapshots. This would be
  circular, as explained in step 2.
- Changing how `pprint_store` orders Streams, or how the Store holds them.
