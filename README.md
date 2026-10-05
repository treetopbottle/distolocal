# Distolocal

Distributed event storage: a local-first event store, written in Rust.

This is a specification of the software using a why? who? what? approach. Where possible, specification by example is used.

_To do: decide on styling of domain entities (capitalize? fixed width font?) and examples (italics? marked with "Example:"?)._

## Why?

Mission: Enable applications to store events in a local database to work without an internet connection, that can be replicated for collaboration.

Vision: Developers building local-first apps adopt this store instead of building sync logic themselves, the way they'd adopt SQLite instead of writing a file format.


## Who? (roles)

- Application user; Annabel, user of Todo app
- Synchronization manager; Annabel, make the Todo app events available to Claire
- Developer; Debby, works on Todo app


## What? (domain entities)

These are the domain entities. The convention is to capitalize them.

- Event - A single record of state change; TodoFinished Event
- Stream - A collection of Events in order of occurrence; Chores Stream
- Application - A client application that uses multiple Streams to store its data; Todo app
- Node - A server providing the event store; Local node for Todo app
  + One Node per Application, no multi tenancy (yet)
  + Data can be replicated to other nodes

## Core use cases (components)

- Event and Stream management
    + Append Event; create Stream; read Events; close Stream; delete Stream
    + Event data is opaque to Distolocal
- Stream subscription
    + Push Event; Event hooks
    + Builds on Event and Stream management
- Node replication
    + Add Node; Remove Node; View replication lag
    + Builds on Stream subscription
    + Version conflicts and data merges are handled by the Application, not Distolocal

## Where things are

- [distolocal/](distolocal/): the crate. `src/lib.rs` is the Store; `tests/` holds one file per feature, and `tests/common/` the test helpers and the plain-text notation.
- [docs/spec/](docs/spec/README.md): what Distolocal does now, in detail.
- [docs/decisions.md](docs/decisions.md): why it is the way it is.
- [docs/plans/](docs/plans/README.md): how we get there, one folder per slice with its plan and the reviews of its steps, and the backlog.
- [docs/workflow/](docs/workflow/README.md): how to do each kind of task.
- [CONTRIBUTING.md](CONTRIBUTING.md): how we work in this repo.
- [AGENTS.md](AGENTS.md): what coding agents need on top of that.

## Development

From `distolocal/`:

- `cargo fmt`, and `cargo fmt --check` to check without changing files
- `cargo test`
- `cargo clippy --all-targets`, which must be clean
- `cargo insta review`, only to update a snapshot that was already approved
