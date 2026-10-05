# Review changes

Review a set of changes as someone who didn't write them. Report findings;
don't fix anything.

## What to review

The commits you were given: a review is kept, so it is of commits, never of
uncommitted changes. Read the plan step behind them, the spec rules it names,
and the decisions they cite.

## Checks

The [code checks](code-checks.md), then:

**Spec and decisions.**
- The code does what the spec says, and nothing the spec doesn't.
- A choice the code makes that isn't obvious has a decision.
- The plan step is marked done, and the docs follow the rules in AGENTS.md.

**Scope.**
- The change is the least that does the job. No unused code, no comments
  that repeat the code.
- A change to the public API is one the step asked for.

## Report

The review is a file in the plan's folder, `review-step-<N>.md`, with `-2`,
`-3` and so on for later reviews of the same step. It has:

- A title, `Review: plan NN, step N`.
- A line with the commit range reviewed and the date.
- **Findings**, most severe first. Each gives the `file:line`, what is wrong,
  and the case that shows it. "No findings" is a valid report.

A reviewer that can't write files returns the report, and whoever ran the
review writes the file. Once written, the findings aren't edited.

## Outcome

Added by whoever deals with the findings, not the reviewer: one line per
finding, saying it was fixed (with the commit), kept (and why), or led to a
change in the spec or the decisions.
