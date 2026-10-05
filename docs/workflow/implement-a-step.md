# Implement a step

Carry out one step of an active plan in `docs/plans/`. One step at a time,
even when asked to finish the plan.

## Prepare

Read the step, the spec rules it names, and the decisions they cite. If the
step or a rule is unclear, ask before writing anything.

## (a) Tests

1. Write one test per behavior, named `<function>_<situation>`. Arrange
   through `parse_store`, and write each `Then` by hand as the inline
   snapshot of decision 0010. The spec's examples are in the same notation,
   so they paste in.
2. Run `cargo test`. The new tests should fail, or not compile, for the
   reason the step expects.
3. **Checkpoint.** Show the tests, with each of the step's rule ids matched
   to the test that covers it. Wait for an OK.
4. Commit the tests with a pathspec.

## (b) Implementation

1. Write the least code that makes the tests pass.
2. Run `cargo fmt`, then the [code checks](code-checks.md). All must pass.
3. **Checkpoint.** Show the diff and wait for an OK.
4. Commit it with a pathspec.

## Review

1. [Review the step's commits](review-changes.md). The review goes in the
   plan's folder; commit it on its own, before any fix.
2. Deal with each finding in a new commit, not by rewriting the reviewed
   ones. Then add the review's Outcome section and commit it.
3. If the fixes change much, review them again, in a new review file.

## Afterwards

Mark the step `[DONE]` in the plan, with links to its review files and a line
on anything that came out of it. A choice worth keeping goes into
`docs/decisions.md`, and a change to what Distolocal does goes into the spec.

If the design doesn't hold up at any point, stop: flag it, update the spec or
the decisions, and adjust the plan before going on.
