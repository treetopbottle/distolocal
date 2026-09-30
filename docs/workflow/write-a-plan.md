# Write a plan

Turn one slice of work into a plan: a file in `docs/plans/` that splits it
into steps, each small enough to review.

## Before writing

1. Pick the slice from the backlog in `docs/plans/README.md`, or take the one
   you were given.
2. Read the spec rules it covers, and the decisions they cite.
3. If a rule is missing, ambiguous or contradicts another, stop. Fix the spec
   first, or record the question as an `Undecided` decision. A plan built on
   a gap routes around it.

## The plan

Write `docs/plans/NN-<slug>.md`, numbered after the last plan, with:

- A title, and a status line: `Status: Draft`.
- **Scope:** what the slice covers, and what it leaves out.
- **Working agreement:** only what differs from AGENTS.md for this slice, if
  anything.
- **Steps**, ordered by real dependency. Each step names the rule ids it
  covers and splits into **(a) tests** and **(b) implementation**. A step
  that only refactors or only changes docs has no split.

Add the plan to the table in `docs/plans/README.md`, and take its slice out
of the backlog.

## Checkpoint

Show the plan and wait for an OK. Once approved, set its status to `Active`
and commit it with a pathspec.
