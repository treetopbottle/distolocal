# 0001. Use MADR for architecture decision records

* Status: accepted
* Date: 2026-07-07

## Context and Problem Statement

This project's architecture and requirements are expected to change as it's built with LLM
agent assistance. Decisions need to be recorded somewhere an agent (or the author, later) can
recover not just *what* was decided but *why*, including alternatives that were rejected —
otherwise a future revision risks silently re-litigating settled ground or missing the reason
a decision was made.

## Decision Drivers

* Needs to be cheap enough to write that it's actually kept up to date.
* Needs to capture rejected alternatives, not just the outcome, for future re-evaluation.

## Considered Options

* Nygard's original ADR format (Context / Decision / Status / Consequences)
* MADR (adds Decision Drivers and Considered Options with pros/cons)
* Y-Statement (single compressed sentence)

## Decision Outcome

Chosen option: "MADR", because the Considered Options section is what lets a later reader —
human or agent — understand why an alternative wasn't picked, which matters more here than
the extra writing cost.

### Consequences

* Good, because decisions are revisitable with full context instead of just an outcome.
* Bad, because each ADR takes longer to write than the Nygard/Y-Statement alternatives.

## Pros and Cons of the Considered Options

### Nygard's original

* Good, because it's the fastest to write.
* Bad, because it doesn't capture alternatives considered.

### Y-Statement

* Good, because it's extremely compact.
* Bad, because a single sentence isn't enough space for real trade-off discussion.
