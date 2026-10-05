# AGENTS.md

Distolocal is a local-first event store, written in Rust. This file is for
any coding agent, and for people: how the repo is laid out and how we work in
it.

## Where things are

- `distolocal/`: the crate. `src/lib.rs` is the Store; `tests/` holds one
  file per feature, and `tests/common/` the test helpers and the plain-text
  notation.
- `docs/spec/`: what Distolocal does now.
- `docs/decisions.md`: why it is the way it is.
- `docs/plans/`: how we get there, one plan per slice, and the backlog.
- `docs/workflow/`: how to do each kind of task.

## Commands

From `distolocal/`:

- `cargo fmt`, and `cargo fmt --check` to check without changing files
- `cargo test`
- `cargo clippy --all-targets`, which must be clean
- `cargo insta review`, only to update a snapshot that was already approved

## Working agreement

- **Tests first, with a checkpoint.** Write a step's tests, stop and show
  them, and wait for an OK before implementing. Tests and implementation are
  separate commits. A test file already on disk is not an approved one.
- **Snapshots are written, not recorded.** A test's `Then` is an inline
  `insta` snapshot, written by hand from the spec. It fails until the
  implementation matches (decision 0010).
- **Tests carry no spec comments.** A test is named `<function>_<situation>`
  (decision 0017).
- **If the design doesn't hold up, stop.** Flag it, update the spec or the
  decisions, and adjust the plan before going on. Don't route around it.
- **Commit only when asked, and with a pathspec** (`git commit -- <paths>`).
  Files sometimes arrive in the index already staged.

## Docs

Following decision 0028:

- The spec says what Distolocal does now: no history, no placeholders, no
  reasons.
- Decisions say why. Append-only; follow the guidelines at the top of
  `docs/decisions.md`.
- A plan has a status line. When it is done, move what should outlive it into
  the spec or the decisions.
- References point only toward the longer-lived: plans cite the spec and the
  decisions, the spec cites decisions, and neither cites a plan.

## Style

- Domain entities are capitalized: Event, Stream, Application, Node.
- Comments go on their own line above what they describe, not after it, in
  code and in the spec's code blocks.
- Match the wrapping of the file you're in: the spec has one line per
  paragraph; plans, decisions and this file wrap at 78 columns.
