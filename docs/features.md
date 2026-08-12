# Features

The single overview of this product's end vision and current status, organized by domain
entity. This file doesn't restate behavior — see [docs/spec.md](spec.md) — or
delivery detail — see [plan/slices/](../plan/slices/) — it only links to them. When a
concept's status changes (e.g. a slice moves from planned to done), update its row here in
the same commit.

## Status legend

| Status | Meaning |
|---|---|
| Specified | Described in the domain spec; no slice built yet |
| Planned | A slice covers it; work hasn't started |
| In progress | A slice covering it is underway |
| Done | All slice(s) covering it are done |

## Stream

Entity: [docs/spec.md#stream](spec.md#stream)

### Commands

| Command | Status | Slice | Notes |
|---|---|---|---|
| [Append](spec.md#append) | Done | [0001](../plan/slices/0001-append-and-read-single-stream-in-memory.md), [0002](../plan/slices/0002-expected-version-check-on-append.md) | 0001 covers append with no expected version. Expected-version (optimistic concurrency) check added in 0002. |

### Queries

| Query | Status | Slice | Notes |
|---|---|---|---|
| [ReadStream](spec.md#readstream) | Done | [0001](../plan/slices/0001-append-and-read-single-stream-in-memory.md), [0003](../plan/slices/0003-read-stream-from-position.md) | 0001 covers reading from the start. Reading from a given position added in 0003. |
| [ListStreams](spec.md#liststreams) | Planned | [0006](../plan/slices/0006-list-streams.md) | |
| [Subscribe](spec.md#subscribe) | Planned | [0007](../plan/slices/0007-catch-up-subscription.md) | |

### Events

_(none yet — see [docs/spec.md](spec.md#events))_

### Cross-cutting

| Concern | Status | Slice | Notes |
|---|---|---|---|
| Access control (per-stream append/read permissions) | Planned | [0011](../plan/slices/0011-access-control.md) | Assumes client identity already established; authentication mechanism is out of scope. |
| Persistent storage adapters | Specified | [0004](../plan/slices/0004-filesystem-storage-adapter.md) (filesystem), [0008](../plan/slices/0008-s3-compatible-storage-adapter.md) (S3-compatible) | No spec change — adapters implement the existing storage port; see [docs/architecture/overview.md](architecture/overview.md). |
| Node API surface | Specified | [0005](../plan/slices/0005-local-node-api-surface.md) | Exposes existing Append/ReadStream over a process boundary; protocol choice may warrant an ADR. |
| Node-to-node sync | Specified | [0009](../plan/slices/0009-two-node-sync-happy-path.md) (happy path), [0010](../plan/slices/0010-sync-conflict-resolution.md) (conflicts) | Sync/transport port design and conflict policy are undecided — need an ADR before implementation, per AGENTS.md. |
