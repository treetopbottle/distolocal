# Review changes

Review a plan, or one step of it, as someone who didn't write it. Report
findings; don't fix anything.

## What to review

The commits you were given: a review is kept, so it is of commits, never of
uncommitted changes. They hold either a new plan, or one step of an active
plan. Read the plan, the spec rules it names, and the decisions they cite.

## Checks for a plan

**Slice.**
- The slice is a backlog item, or part of one, and the backlog no longer
  lists it.
- Each rule id it names is in the spec, unambiguous, and doesn't contradict
  another.
- Scope says what the slice leaves out.

**Design.**
- It fits the existing code and the decisions, and reopens a decision only
  by saying so.
- A choice that isn't obvious has a decision.
- A change to the public API is named.

**Steps.**
- Every rule id in Scope is covered by a step.
- Steps are ordered by real dependency, and each is small enough to review.
- Each step splits into tests and implementation, unless it only refactors
  or changes docs.

## Checks for a step

The [code checks](code-checks.md), then:

**Spec and decisions.**
- The code does what the spec says, and nothing the spec doesn't.
- The code follows the plan's Design, or the plan says why not.
- A choice the code makes that isn't obvious has a decision.
- The docs follow the rules in CONTRIBUTING.md.

**Scope.**
- The change is the least that does the job. No unused code, no comments
  that repeat the code.
- A change to the public API is one the step asked for.

## Report

The review is a file in the plan's folder: `review-plan.md` for the plan,
`review-step-<N>.md` for a step, with `-2`, `-3` and so on for later reviews
of the same thing. It has:

- A title, `Review: plan NN` or `Review: plan NN, step N`.
- A line with the commit range reviewed and the date.
- **Findings**, most severe first. Each gives the `file:line`, what is wrong,
  and the case that shows it. "No findings" is a valid report.

A reviewer that can't write files returns the report, and whoever ran the
review writes the file. Once written, the findings aren't edited.

## Outcome

Added by whoever deals with the findings, not the reviewer: one line per
finding, saying it was fixed (with the commit), kept (and why), or led to a
change in the spec or the decisions.
