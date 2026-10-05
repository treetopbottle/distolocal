# Finish a plan

Close a plan once its last step is done: keep what should outlive it, and
leave the decisions tidier than the plan found them.

## Keep what should outlive it

1. Read the plan, its reviews, and the lines its `[DONE]` steps added.
2. A change to what Distolocal does goes into the spec, and a choice worth
   keeping into `docs/decisions.md`, if they aren't there already.
3. Move anything left for later to the backlog in `docs/plans/README.md`.
4. Commit it with a pathspec.

## Review the decisions

1. [Review the decisions](review.md) against the code and the spec.
   The review goes in the plan's folder; commit it on its own, before any
   fix.
2. Deal with each finding in a new commit. Cleaning up may tighten wording,
   fix the index and the `relates to` links, mark a supersession on both
   sides, and move a wholly superseded decision to Superseded decisions.
   Changing what was decided takes a new decision.
3. Add the review's Outcome section and commit it.

## Checkpoint

Show what moved, and the review. Wait for an OK, then set the plan's status
to `Done`, with the date, in the plan and in `docs/plans/README.md`, and
commit it with a pathspec.
