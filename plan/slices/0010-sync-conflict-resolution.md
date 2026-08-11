# 0010. Sync conflict resolution

* Status: planned

## Goal

When two nodes have appended independently to the same stream while disconnected, syncing
reconciles the divergent histories according to a defined policy, so neither node silently
loses or duplicates events.

## Scope

- Application/domain: conflict detection and resolution policy for sync, built on the
  transport from [0009](0009-two-node-sync-happy-path.md).

## Out of scope

- User-configurable resolution policies — a single default policy only, until proven
  insufficient.

## Depends on

[0009](0009-two-node-sync-happy-path.md)

**Before implementing:** the conflict resolution policy changes a storage/consistency
contract and is squarely ADR-worthy. Per AGENTS.md's stop-and-ask rule, don't decide the
policy unilaterally during implementation — propose the ADR first, then write examples here
once it's accepted. No examples are included yet for that reason; don't invent one from
memory of what "should" happen.

## Examples

_(pending the ADR above)_

## Observability

- Metric: count of conflicts detected and resolved, per stream, per sync cycle.
- Log: conflict detection and resolution outcome, at info level.
- Trace: conflict resolution as a labeled step within the sync cycle span from 0009.
