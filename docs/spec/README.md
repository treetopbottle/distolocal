# Distolocal Specification

What Distolocal does, as it stands. Where possible, specification by example is used. The roles and domain entities are in the [project README](../../README.md); why things are the way they are is in [the decisions](../decisions.md).

## Components

- [Event and Stream management](event-and-stream-management.md) — Create, Append, Read, Get, Close and Delete Stream.
- Stream subscription — not specified yet.
- Node replication — not specified yet.

The Event and Stream records are in [schemas.md](schemas.md).

## Errors

Errors returned by the Store's Event and Stream management features.

| Error | Raised when |
|---|---|
| `StreamNotFound` | Append, Read, Get, Close or Delete is called on a Stream that doesn't exist — never created, or deleted |
| `StreamClosed` | Append is called on a Stream that is closed, or Create is called on an id that belongs to a closed Stream |
| `StreamNotClosed` | Delete is called on a Stream that is still open |
| `EventIdConflict` | Append is called with an `event_id` already used in that Stream, whatever the new Event holds; the error carries the Event already stored (see AppendEvent.05) |

The API adapters (HTTP, gRPC, …) add one of their own, `InvalidEvent`, for an Append request whose Event is missing a required field or holds one of the wrong type or format (see AppendEvent.11). The Store's own API is typed, so a malformed Event cannot reach it.

## Narrative: Annabel and the Todo Application

Annabel installs the Todo app. The app creates a single "Lists" Stream that will record every list [Create Stream].

Annabel starts a "Chores" list. The app first records the new list in "Lists" [Append Event], then creates a Stream with an opaque id, "chores-3f2a1c", to hold its items [Create Stream]. The display name of the list, "Chores", is taken from the Event, not the Stream.

Annabel adds "Take out the trash" to her chores list [Append Event]. Annabel adds "Wash the dishes" [Append Event].

Annabel marks "Take out the trash" as done [Append Event].

A while later, Annabel reopens the Todo app. It checks that the "Chores" list is still open [Get Stream], then reads the full Stream to reconstruct it, showing one open and one finished chore [Read Stream].

Annabel marks the whole "Chores" list as finished. The app creates a "ChoresHistory" Stream [Create Stream], appends to it a summary of how many chores were completed [Append Event], then closes the "Chores" Stream [Close Stream].

After a week, the Todo app cleans up the finished "Chores" list, deleting the Stream entirely [Delete Stream].

[Create Stream]: event-and-stream-management.md#feature-create-stream
[Append Event]: event-and-stream-management.md#feature-append-event
[Read Stream]: event-and-stream-management.md#feature-read-stream
[Get Stream]: event-and-stream-management.md#feature-get-stream
[Close Stream]: event-and-stream-management.md#feature-close-stream
[Delete Stream]: event-and-stream-management.md#feature-delete-stream

## Pattern: closing the books

"Closing the books" — computing a rollup of a Stream's history instead of keeping every individual Event, e.g. a cashier totaling a day's transactions instead of keeping every line item sold — needs no dedicated Store feature. It composes entirely from the [Event and Stream management](event-and-stream-management.md) features, with the application owning both the summarization logic and the sequencing:

1. [Create Stream] a summary Stream (if it doesn't already exist) to hold the rollup, e.g. "ChoresHistory".
2. [Append Event] the summary Event(s) to it, computed however the application sees fit.
3. [Close Stream] the original Stream.
4. Once the summary no longer needs to be paired 1:1 with the original — whenever the application decides — [Delete Stream] the original to reclaim space; the summary Stream is untouched.

None of these steps need to be atomic with each other: a Stream that's closed but not yet summarized, or summarized but not yet deleted, is still fully valid and readable throughout — see `DeleteStream.03`.
