# 0003. Use Rust as the implementation language

* Status: accepted
* Date: 2026-08-11

## Context and Problem Statement

No implementation language has been chosen yet — slice
[0001](../../plan/slices/0001-append-and-read-single-stream-in-memory.md) is the first to
need one. The choice affects every later slice: the storage port needs adapters with tight
control over I/O (in-memory, filesystem, S3-compatible), the system is distributed as a node
that runs locally on a client machine rather than behind a server, and the domain model
(entities, commands with optimistic-concurrency checks, later a sync/conflict layer) benefits
from a type system that can encode invariants rather than relying on runtime checks.

## Decision Drivers

* The node is distributed to and runs on a client's own machine (see
  [ADR-0002](0002-local-first-node-per-client.md)) — a single, dependency-free binary is
  preferable to requiring a language runtime on every client.
* Storage adapters (filesystem, S3-compatible, later sync/transport) are I/O- and
  concurrency-heavy; predictable performance and memory safety without a GC matter here.
* The domain model wants a type system expressive enough for DDD-style entities and
  value objects (e.g. modeling append results, version conflicts, and later sync outcomes as
  exhaustive enums) to catch invariant violations at compile time.
* This is a learning project held to production-readiness standards (see
  [README.md](../../README.md)) — favors a language with a strong, opinionated toolchain
  (formatter, linter, test runner) built in, rather than assembling one.

## Considered Options

* Rust
* Go
* TypeScript / Node.js

## Decision Outcome

Chosen option: "Rust", decided directly rather than through a bake-off — recorded here so the
reasoning and alternatives are still visible to a future reader, per
[ADR-0001](0001-use-madr-for-adrs.md). Rust best matches the decision drivers above: no
runtime to install on a client node, a type system suited to modeling domain invariants
(particularly useful once optimistic-concurrency and sync-conflict outcomes need to be
represented exhaustively), and `cargo` bundles formatting, linting, and testing rather than
requiring separate tool selection.

### Consequences

* Good, because a compiled, dependency-free binary fits the local-first, per-client node
  deployment shape directly.
* Good, because the type system (enums, `Result`, ownership) can express domain invariants
  (e.g. a version conflict as a distinct, unrepresentable-as-success outcome) at compile time.
* Good, because `cargo fmt` / `cargo clippy` / `cargo test` give a standard toolchain without
  further tool selection.
* Bad, because Rust's learning curve (ownership/borrowing) is steeper than Go or TypeScript,
  which matters for a learning project's pace.
* Bad, because compile times are slower than Go, which can slow the tight
  example-test-implement loop CONTRIBUTING.md calls for.

## Pros and Cons of the Other Considered Options

### Go

* Good, because it also compiles to a single dependency-free binary, and has a simpler
  learning curve than Rust.
* Bad, because its type system (no sum types/enums-with-data until recently, weaker
  exhaustiveness checking) is a poorer fit for modeling domain outcomes like version
  conflicts or sync results as data.

### TypeScript / Node.js

* Good, because it's fast to write in and has the largest ecosystem of the three.
* Bad, because it requires a Node runtime on every client node, working against the
  dependency-free, per-client-machine deployment shape from
  [ADR-0002](0002-local-first-node-per-client.md).
* Bad, because its type system is erased at runtime, so domain invariants (e.g. version
  conflict as an unrepresentable-as-success state) rely on discipline rather than the
  compiler.
