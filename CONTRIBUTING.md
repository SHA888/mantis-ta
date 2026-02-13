# Contributing

Thanks for your interest in improving mantis-ta!

## Ground rules
- Follow the indicator checklist in SPEC §4.2 for all indicators (tests, TA-Lib parity, benchmarks, docs).
- Keep the public API documented; run `cargo doc --no-deps` before submitting.
- No new `unsafe` without explicit justification.
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and all tests before opening a PR.

## Process
1. Open an issue describing the change (link math definition + reference impl for indicators).
2. Wait for acceptance/assignment; then implement in the appropriate module.
3. Add tests and benchmarks.
4. Update docs/examples if behavior or APIs change.
5. Open a PR referencing the issue; ensure CI passes.
