# Contributing

Conventions in this file apply equally to human and agent contributors. `AGENTS.md` adds
operational rules on top of this — it never restates what's already here.

## Getting started

- Install Rust via [rustup](https://rustup.rs).
- Clone the repo, then from the repo root:
  - `cargo build` — build the `distolocal` binary.
  - `cargo test` — run the approval test suite.
  - `cargo run` — run the binary (currently a thin demo wiring the in-memory
    adapter into the Append/ReadStream handlers; see
    [plan/slices/0005-local-node-api-surface.md](plan/slices/0005-local-node-api-surface.md)
    for the real API surface).
- Before opening a PR, run the same gate CI runs: `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.

## Domain modeling

- Use Domain-Driven Design. Keep the ubiquitous language in
  [docs/spec.md](docs/spec.md) in sync with the code — a renamed concept in code
  means an updated spec, same commit.
- Bounded contexts are not yet decided (see [docs/spec.md](docs/spec.md)); until
  they are, the domain model lives in one place. Don't pre-split it.

## Specification by example

- Write the example before the code. Examples live in [docs/spec.md](docs/spec.md)
  (inline with the concept they clarify) or in a slice's own example section under
  [plan/slices/](plan/slices/).
- Approval tests encode these examples directly — a test's approved output should be
  traceable back to a written example, not invented ad hoc during implementation.

## Sliced architecture

- Features are built as vertical slices: a slice cuts through domain, application, and
  infrastructure to deliver one thin, end-to-end working capability — not a horizontal layer.
- Each slice is tracked as a file in [plan/slices/](plan/slices/). A slice is done when its example(s) pass
  as approval tests end-to-end, not when one layer is "finished."
- Slices can be reordered, split, or dropped — the plan is expected to change. Update the
  slice's status rather than treating the sequence as fixed.

## Workflow

### Git

- One branch per slice, named after the slice file, e.g.
  `slice/0002-expected-version-check-on-append`.
- Commit freely within the branch — the natural granularity is one commit per step of the
  per-slice loop below (failing test, implementation, observability, docs update). This is
  the history worth keeping.
- Merge to `main` with a real merge commit (`git merge --no-ff`), never squash, never
  rebase-to-linear. The merge commit marks "slice N landed" as a clean point in history; the
  branch's internal commits stay intact underneath it.
- Parallel slices: parallel branches off `main`. Nothing merges until its approval tests pass
  end-to-end, so `main` is always a set of complete slices — no feature flags needed.
- Dependency chains (e.g. [0007](../plan/slices/0007-catch-up-subscription.md) depends on
  [0003](../plan/slices/0003-read-stream-from-position.md)): don't branch off an unmerged
  dependency. Wait until the dependency slice is merged to `main`, then branch the dependent
  slice from `main` as normal.
- ADR-blocked slices (e.g.
  [0009](../plan/slices/0009-two-node-sync-happy-path.md),
  [0010](../plan/slices/0010-sync-conflict-resolution.md)): the ADR lands first, as its own
  small branch/merge (docs only), before the slice's implementation branch is opened.

### Per-slice loop

1. Branch from `main`. Mark the slice file's `Status` as `in-progress`, commit that alone.
2. Re-read the slice file and its `docs/spec.md` example(s) — confirm scope before writing
   anything.
3. Write the approval test(s) from the Examples section, red. Commit.
4. Implement the thinnest code to go green, respecting the domain/application/infrastructure
   boundaries in [docs/architecture/overview.md](docs/architecture/overview.md). Commit (can
   be several commits if it's naturally staged).
5. Add the slice's Observability section (metric/log/trace) in the same slice, not a
   follow-up. Commit.
6. Run the full gate locally: `cargo fmt --check`, `cargo clippy --all-targets -- -D
   warnings`, `cargo test`. Fix anything red before moving on.
7. Update `docs/features.md` status for every concept the slice covers, and flip the slice
   file's `Status` to `done` — same commit as whatever code change makes it true.
8. Run `/code-review` on the branch's diff against `main` as the merge gate. Apply or
   consciously dismiss findings.
9. Merge to `main` with `--no-ff`. Delete the branch.

### Tooling gates

- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are the two
  non-negotiable local gates before `/code-review`.
- `cargo test` runs the approval test suite (see Specification by example above);
  adapter-parity slices (filesystem, S3-compatible) share one test module parameterized over
  adapters rather than duplicating the suite.
- If a GitHub remote is added, the same three commands become the CI job gating merge.

## Feature overview

- [docs/features.md](docs/features.md) is the single overview of end vision and current
  status, organized by domain entity. It links into [docs/spec.md](docs/spec.md)
  for behavior and [plan/slices/](plan/slices/) for delivery detail — it never restates
  either.
- A command, query, or event gets a row in `docs/features.md` when it's added to the spec,
  even before a slice exists for it (status: Specified).
- When a slice's status changes, update the status of every concept it covers in
  `docs/features.md`, same commit.

## Modular abstraction

- Storage is a port with swappable adapters (in-memory, filesystem, S3-compatible). Domain
  and application code depend on the port, never on a concrete adapter.
- The in-memory adapter is the reference implementation for behavior; other adapters must
  pass the same approval test suite against it.

## Observability

- Metrics, logs, and traces are part of the slice, not a follow-up task. A slice that adds a
  command/query handler adds its instrumentation in the same slice.

## Architecture decisions

- Significant, hard-to-reverse decisions get an ADR in
  [docs/architecture/decisions/](docs/architecture/decisions/), using the
  [MADR format](docs/architecture/decisions/template.md). Superseded decisions get a new ADR,
  not an edit to the old one.
