# Implement a step

Carry out one step of an active plan in `docs/plans/`. One step at a time,
even when asked to finish the plan.

## Prepare

Read the step, the spec rules it names, and the decisions they cite. If the
step or a rule is unclear, ask before writing anything.

## (a) Tests

1. Write one test per behavior, named `<function>_<situation>`. Arrange
   through `parse_store`, and write each `Then` as an inline snapshot by
   hand. The spec's examples are in the same notation, so they paste in.
2. Run `cargo test`. The new tests should fail, or not compile, for the
   reason the step expects.
3. **Checkpoint.** Show the tests, with each of the step's rule ids matched
   to the test that covers it. Wait for an OK.
4. Commit the tests with a pathspec.

## (b) Implementation

1. Write the least code that makes the tests pass.
2. Run `cargo fmt`, `cargo test` and `cargo clippy --all-targets`. All must
   be clean.
3. **Checkpoint.** Show the diff and wait for an OK.
4. Commit it with a pathspec.

## Afterwards

Mark the step `[DONE]` in the plan, with a line on anything that came out of
it. A choice worth keeping goes into `docs/decisions.md`, and a change to what
Distolocal does goes into the spec.

If the design doesn't hold up at any point, stop: flag it, update the spec or
the decisions, and adjust the plan before going on.
