# Workflow

How to do each kind of task in this repo, for agents and for people. The
working agreement in [AGENTS.md](../../AGENTS.md) applies throughout.

A slice of work goes through these in order:

1. [Write a plan](write-a-plan.md): turn a backlog item and its spec rules
   into steps.
2. [Implement a step](implement-a-step.md): one step of a plan, tests first.
3. [Review changes](review-changes.md): check a step's commits against the
   plan, the spec and the decisions.

The [code checks](code-checks.md) apply to the last two.

Each has a thin wrapper in `.agents/skills/`, so an agent that supports Agent
Skills can start it by name. The wrapper only points here.
