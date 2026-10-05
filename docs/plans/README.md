# Plans

How we get to what the [spec](../spec/README.md) describes. A plan is one
slice of work, split into steps with checkpoints. Each plan has a folder:
the plan is its `README.md`, next to the reviews of its steps
([decision 0029](../decisions.md#0029)). When it is done, anything that
should outlive it moves into the spec or the [decisions](../decisions.md),
and the folder stays here as history ([decision 0028](../decisions.md#0028)).

Every plan follows the working agreement in
[CONTRIBUTING.md](../../CONTRIBUTING.md); plan 01 has the original.

| Plan | Status |
|---|---|
| [01 — Event and Stream management (v1)](01-event-and-stream-v1/README.md) | Done |
| [02 — Simplify the remaining code](02-simplify-code/README.md) | Done |

## Backlog

Next slices, not planned yet:

- Persistence (currently in-memory only). The test notation is test-only and
  deliberately not a candidate format for it (decision 0011).
- Stream subscription (push/hooks) — builds on Event and Stream management.
- Node replication, and the still-open questions in decisions 0003 and 0006
  (vector clocks, concurrent writers to the same Stream at the same Node).
  Stream deletion needs rethinking here: tombstones, so Nodes remember a
  deleted Stream and a recreated id doesn't restart its vector clock
  (decision 0023), decided together with retention (0024).
- A public API protocol (JSON first; gRPC/Avro anticipated — decision 0007).
  Today the only "API" is the Store's Rust function calls used directly by
  the tests. `AppendEvent.11`'s shape validation lands here, in the adapters
  (decision 0018).

Ideas:

- A snapshot test for the public API of the Store, both as documentation and
  as a safeguard against backwards incompatible changes.
- The Event schema could take inspiration from CloudEvents.

Spec sections still to write, each when its slice starts:

- Interaction design: UI designs and mockups, commands and view models — the
  visual representation of the software. Take inspiration from event
  modeling.
- Non-functional requirements.
- Technical design: a component diagram, data storage, and the APIs,
  including protocols (HTTP, gRPC, etc.) and perhaps internal ones.
- Metrics.
