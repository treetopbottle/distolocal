# Workflow

How to do each kind of task in this repo, for agents and for people. The
working agreement in [CONTRIBUTING.md](../../CONTRIBUTING.md) applies
throughout.

A slice of work goes through these in order:

1. [Write a plan](write-a-plan.md): scope a backlog item, design it, and
   split it into steps. The plan is reviewed before any step starts.
2. [Implement a step](implement-a-step.md): one step of a plan, tests first,
   then reviewed.
3. [Review changes](review-changes.md): check a plan, or a step's commits,
   against the spec and the decisions.

The [code checks](code-checks.md) apply to implementing and reviewing a
step.

Each has a thin wrapper in `.agents/skills/`, so an agent that supports Agent
Skills can start it by name. The wrapper only points here.
