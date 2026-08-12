# 0001 implementation plan

Companion to
[0001-append-and-read-single-stream-in-memory.md](0001-append-and-read-single-stream-in-memory.md)
— that file is the *what/why*; this is the concrete *how*, per the per-slice loop in
[CONTRIBUTING.md](../../CONTRIBUTING.md#workflow).

## 0. Blocked on: observability stack ADR

[docs/architecture/overview.md](../../docs/architecture/overview.md) marks the observability
stack as "pending an ADR." This slice's Observability section needs metrics/logs/traces, so
**before implementation starts**, land an ADR (proposed: ADR-0004, recommending the `tracing`
crate — it covers structured logs and spans/traces in one dependency, and is the Rust
ecosystem default) on its own small branch/merge into `main`, per the ADR-blocked-slice rule
in CONTRIBUTING.md. Don't decide the metrics side unilaterally while writing code — confirm
in the ADR.

## 1. Project scaffolding (first commits on this branch, after the ADR lands)

- `cargo init --name distolocal` at repo root → single binary crate (not a workspace; no
  reason yet to split into multiple crates — see "no premature abstraction").
- `.gitignore`: `/target`. Commit `Cargo.lock` (this is an application/node binary, not a
  library — standard Rust practice commits the lockfile for binaries).
- `.github/workflows/ci.yml`: on push/PR to `main`, run `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all`. Mirrors the
  local gate from CONTRIBUTING.md's Tooling gates section.

## 2. Module layout

Single crate, module-per-layer, matching the domain/application/infrastructure boundary in
[docs/architecture/overview.md](../../docs/architecture/overview.md):

- `src/main.rs` — thin entrypoint wiring the in-memory adapter into the handlers.
- `src/domain/stream.rs` — `Stream` entity: ordered, append-only, zero-based position.
- `src/application/append.rs` — `Append` command handler.
- `src/application/read_stream.rs` — `ReadStream` query handler.
- `src/infrastructure/storage/mod.rs` — the storage port (a trait, e.g. `StreamStore`) that
  application handlers depend on.
- `src/infrastructure/storage/in_memory.rs` — the in-memory adapter implementing it.

Domain and application code depend on the `StreamStore` trait only, never on the in-memory
adapter directly — matches "Modular abstraction" in CONTRIBUTING.md, and sets up
[0004](0004-filesystem-storage-adapter.md)/[0008](0008-s3-compatible-storage-adapter.md)
(filesystem/S3 adapters) to be adapter-swaps against the same trait.

## 3. Approval tests

Per CONTRIBUTING's "write the example before the code": one integration test file,
`tests/slice_0001_append_and_read_stream.rs`, encoding the two Given/When/Then examples from
the slice file/spec.md verbatim:

- 2 events appended to a new stream with no expected version → stored at positions 0 and 1.
- `ReadStream` on a stream with 2 events already appended → returns them in append order.

No snapshot-testing crate (e.g. `insta`) for now, per prior discussion — plain `#[test]`
functions asserting directly on returned values are enough at this scale.

## 4. Observability (after ADR-0004 lands)

Implement the slice's Observability section: metric (count of events appended per stream),
log (append/read at debug level, including stream ID and resulting position), trace (one
span per handler invocation) — using whatever ADR-0004 settles on.

## 5. Docs updates (same commits as the code that makes them true)

- [docs/features.md](../../docs/features.md): flip Append/ReadStream rows to "Done" once
  tests pass.
- This slice's own status file: flip `Status` to `done`.
- CONTRIBUTING.md's "Getting started" stub: replace with real clone/build/test instructions
  now that a Cargo project exists.

## 6. Merge

`/code-review` on the branch diff, then `git merge --no-ff` into `main` per the documented
git workflow. Not part of this plan — a deliberate later step.
