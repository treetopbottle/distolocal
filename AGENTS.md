# Agent Guide

Read [CONTRIBUTING.md](CONTRIBUTING.md) first — it is the shared source of truth for both
human and agent contributors. This file only adds rules specific to working as an agent.

## Before implementing anything

1. Read [docs/spec.md](docs/spec.md) for the relevant `what` (entities/commands/queries/
   events) and any existing examples. If the concept you're about to build isn't in there,
   add it before writing code — spec first, then example, then implementation.
2. Check [docs/architecture/decisions/](docs/architecture/decisions/) for a decision that
   already covers the area you're touching. Don't silently contradict an accepted ADR — if
   you think it's wrong, say so and propose a new one instead of quietly diverging.
3. Check [plan/slices/](plan/slices/) for the slice this work belongs to. Work inside the
   scope defined there; if scope is unclear, ask rather than expanding it.
4. Check [docs/features.md](docs/features.md) for the current status of the concept you're
   touching, so you don't duplicate or contradict work already planned elsewhere.

## While implementing

- Every new behavior gets an example in the spec (or a slice's example section) before the
  approval test that encodes it. Don't write the approval test from memory of what the
  behavior "should" be — derive it from the written example.
- Respect the module/port boundaries described in
  [docs/architecture/overview.md](docs/architecture/overview.md). A change to a storage
  adapter must not leak into domain code, and vice versa.
- When a slice's status changes, or a new command/query/event is added to the spec, update
  [docs/features.md](docs/features.md) in the same commit.

## When to stop and ask

- Before proposing a new ADR-worthy decision (anything that changes an architectural
  boundary, storage contract, or cross-slice concern), surface it and wait — don't decide
  unilaterally and document it after the fact.
- Before reordering, splitting, or dropping a slice in [plan/slices/](plan/slices/).
