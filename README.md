# distolocal

A distributed, local-first event store. Every client runs its own node, so it keeps working
fully offline; nodes synchronize with each other (and any remote peers) when connectivity is
available. Built as a learning project, held to production-readiness standards.

## Status

Pre-code scaffolding — no implementation yet. See [plan/slices/](plan/slices/) for the current
slice and what's next.

## Documentation map

- [AGENTS.md](AGENTS.md) — rules for agents working in this repo
- [CONTRIBUTING.md](CONTRIBUTING.md) — conventions for all contributors (domain-driven design,
  specification by example, sliced architecture)
- [docs/features.md](docs/features.md) — current status of every feature, organized by
  domain entity — start here for what exists and what's next
- [docs/spec.md](docs/spec.md) — domain spec: entities, commands, queries, events
- [docs/architecture/overview.md](docs/architecture/overview.md) — modules, ports, and adapters
- [docs/architecture/decisions/](docs/architecture/decisions/) — architecture decision records
- [plan/slices/](plan/slices/) — vertical slices tracking what's built and what's next

## Getting started

See [Getting started in CONTRIBUTING.md](CONTRIBUTING.md#getting-started).

## License

Not yet decided.
