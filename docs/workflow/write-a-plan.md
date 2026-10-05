# Write a plan

Turn one slice of work into a plan: a folder in `docs/plans/` whose
`README.md` scopes the slice, designs it, and splits it into steps, each
small enough to review. The plan is reviewed before any step starts.

## Before writing

1. Pick the slice from the backlog in `docs/plans/README.md`, or take the one
   you were given. If it is too big for one plan, split it in the backlog
   first, and plan one part.
2. Read the spec rules it covers, the decisions they cite, and the code it
   touches.
3. If a rule is missing, ambiguous or contradicts another, stop. Fix the spec
   first, or record the question as an `Undecided` decision. A plan built on
   a gap routes around it.

## The plan

Write `docs/plans/NN-<slug>/README.md`, numbered after the last plan, with:

- A title, and a status line: `Status: Draft`.
- **Scope:** the backlog item, the rule ids it covers, and what it leaves
  out.
- **Design:** how the code will do it: the types and functions it adds or
  changes, the public API, and the files it touches. A choice that isn't
  obvious goes into `docs/decisions.md`, as `Undecided` if it is still open.
- **Working agreement:** only what differs from CONTRIBUTING.md for this
  slice, if anything.
- **Steps**, ordered by real dependency. Each step names the rule ids it
  covers and splits into **(a) tests** and **(b) implementation**. A step
  that only refactors or only changes docs has no split.

Add the plan to the table in `docs/plans/README.md`, and take its slice out
of the backlog.

## Review

1. Commit the plan, and the docs it changed, with a pathspec.
2. [Review the plan](review-changes.md). The review goes in the plan's
   folder; commit it on its own, before any fix.
3. Deal with each finding in a new commit. Then add the review's Outcome
   section and commit it.

## Checkpoint

Show the plan and its review, and wait for an OK. Once approved, set its
status to `Active` and commit it with a pathspec.
