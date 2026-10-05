# Code checks

What must hold for a step's code and tests, while implementing and when
reviewing. Run them before the implementation checkpoint in
[implement-a-step](implement-a-step.md); at the tests checkpoint the new
tests are still meant to fail. [review-changes](review-changes.md) runs them
again on someone else's work.

**Build.** From `distolocal/`, `cargo fmt --check`, `cargo test` and
`cargo clippy --all-targets` run clean. See Commands in
[AGENTS.md](../../AGENTS.md).

**Tests.**
- Each rule id the step names has a test, matched by name.
- Each snapshot matches the spec's example for its rule. A snapshot that
  doesn't, or has no example, may have been recorded rather than written.
- Names follow `<function>_<situation>`, with no spec comments.
