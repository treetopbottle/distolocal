# Contributing

How we work on Distolocal, for people and for coding agents. Start with the
[README](README.md) for what Distolocal is, where things are, and the
commands. Each kind of task has its steps in
[docs/workflow/](docs/workflow/README.md).

## Working agreement

- **Tests first, with a checkpoint.** Write a step's tests, stop and show
  them, and wait for an OK before implementing. Tests and implementation are
  separate commits.
- **Snapshots are written, not recorded.** A test's `Then` is an inline
  `insta` snapshot, written by hand from the spec. It fails until the
  implementation matches (decision 0010).
- **Tests carry no spec comments.** A test is named `<function>_<situation>`
  (decision 0017).
- **If the design doesn't hold up, stop.** Flag it, update the spec or the
  decisions, and adjust the plan before going on. Don't route around it.
- **Fix forward.** Once a review of some commits has started, don't rewrite
  them: a fix is a new commit, so the review's commit range stays what was
  reviewed. Before that, amending is fine (decision 0029).

## Docs

Following decision 0028:

- The spec says what Distolocal does now: no history, no placeholders, no
  reasons.
- Decisions say why. Append-only; follow the guidelines at the top of
  `docs/decisions.md`.
- A plan has a status line. When it is done, move what should outlive it into
  the spec or the decisions ([finish a plan](docs/workflow/finish-a-plan.md)).
- References point only toward the longer-lived: plans cite the spec and the
  decisions, the spec cites decisions, and neither cites a plan. A plan and
  its reviews may cite each other.

## Style

- Domain entities are capitalized: Event, Stream, Application, Node.
- Comments go on their own line above what they describe, not after it, in
  code and in the spec's code blocks.
- Match the wrapping of the file you're in: the spec and the README have one
  line per paragraph; plans, decisions and the other docs wrap at 78 columns.
