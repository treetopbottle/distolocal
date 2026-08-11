# Architecture Overview

See [ADR-0002](decisions/0002-local-first-node-per-client.md) for the local-first premise this
is built on.

## Context

Every client runs its own node. A node is fully functional offline; when connectivity is
available, nodes sync with peers. There is no mandatory central server.

(Context diagram goes here once there's more than one node type to draw.)

## Modules / ports

Domain and application code depend on ports, never on concrete adapters. Each port has at
least one adapter; the in-memory adapter is the reference implementation that other adapters
are tested against (see [CONTRIBUTING.md](../../CONTRIBUTING.md)).

| Port | Adapters (planned) |
|---|---|
| Storage | in-memory, filesystem, S3-compatible |
| Sync/transport | _(not yet designed — deferred past v1)_ |

## Observability

Metrics, logs, and traces are cross-cutting and instrumented per-slice, not bolted on
afterward. Concrete stack choice is pending an ADR.

## Slices vs. modules

Modules (above) describe the standing structure of the codebase. Slices
([plan/slices/](../../plan/slices/)) describe the order in which capability is built through
that structure — a slice touches multiple modules/layers at once rather than completing one
module before starting the next.
