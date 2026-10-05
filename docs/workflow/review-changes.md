# Review changes

Review a set of changes as someone who didn't write them. Report findings;
don't fix anything.

## What to review

The commits or the diff you were given. Without one, the uncommitted changes
against `HEAD`. Read the plan step behind them, the spec rules it names, and
the decisions they cite.

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

Findings, most severe first. Each gives the `file:line`, what is wrong, and
the case that shows it. "No findings" is a valid report.
