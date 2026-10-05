# Review

Review a plan, one step of it, or the decisions, as someone who didn't write
them. Report findings; don't fix anything.

## What to review

The commits you were given: a review is kept, so it is of commits, never of
uncommitted changes. They hold a new plan, or one step of an active plan.
Read the plan, the spec rules it names, and the decisions they cite.

A review of the decisions is of `docs/decisions.md` at one commit, against
the code and the spec there. Given a range of commits instead, it also
covers how the decisions changed across them.

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

**Tests first.** At the step's tests commit, the new tests fail, or don't
compile, for the reason the step expects. Check it out in a separate
worktree (`git worktree add`), never in the one you were started in.

**Spec and decisions.**
- The code does what the spec says, and nothing the spec doesn't.
- The code follows the plan's Design, or the plan says why not.
- A choice the code makes that isn't obvious has a decision.
- The docs follow the rules in CONTRIBUTING.md.

**Scope.**
- The change is the least that does the job. No unused code, no comments
  that repeat the code.
- A change to the public API is one the step asked for.

## Checks for the decisions

**Each entry.**
- It keeps the why, the cost and the rejected alternatives.
- It says why, not what: what Distolocal does belongs in the spec.

**Still true.**
- The code and the spec still match each `Accepted` decision. One they have
  drifted from needs a new decision, or a fix.
- Each `Undecided` decision is still open. One the code or the spec has
  settled needs a decision that says so.

**Missing.** A choice in the code or the spec that isn't obvious has a
decision.

**Order.**
- The index lists every entry, under the right topic, with its status.
- A supersession is marked on both sides, and a wholly superseded decision
  is under Superseded decisions.
- `relates to` links point at the right entries.

## Report

The review is a file in the plan's folder: `review-plan.md` for the plan,
`review-step-<N>.md` for a step, and `review-decisions.md` for the decisions
when the plan finishes, with `-2`, `-3` and so on for later reviews of the
same thing. A review that belongs to no plan gets a folder of its own in
`docs/plans/`, numbered like a plan, `NN-review-<slug>/`, with the review as
its `README.md`.

A review has:

- A title: `Review: plan NN`, `Review: plan NN, step N`,
  `Review: decisions, plan NN`, or `Review: <what>` for one of its own.
- A line with the commit, or the range of commits, reviewed, and the date.
- **Findings**, most severe first. Each is tagged **must fix**, **should
  fix** or **note**, and gives the `file:line`, what is wrong, and the case
  that shows it. "No findings" is a valid report.

A reviewer that can't write files returns the report, and whoever ran the
review writes the file. Once written, the findings aren't edited.

## Outcome

Added by whoever deals with the findings, not the reviewer: one line per
finding, saying it was fixed (with the commit), kept (and why), or led to a
change in the spec or the decisions. A must-fix is never just kept.

If dealing with the findings changed any code, review again, in a new
review file.
