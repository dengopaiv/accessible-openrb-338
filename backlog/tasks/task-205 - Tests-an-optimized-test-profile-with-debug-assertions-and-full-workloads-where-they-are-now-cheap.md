---
id: TASK-205
title: >-
  Tests: an optimized test profile with debug assertions, and full workloads
  where they are now cheap
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T09:10:00Z'
labels:
  - tests
  - tools
milestone: m-7
dependencies: []
references:
  - Cargo.toml
  - .github/workflows/ci.yml
  - AGENTS.md
  - crates/orb-engine/tests/corpus_golden.rs
  - crates/orb-cli/tests/corpus_mods.rs
  - crates/orb-player/tests/export_equal.rs
  - crates/orb-engine/src/seq/merge_tests.rs
  - crates/orb-ui/tests/frame_cost.rs
  - crates/orb-plugin/tests/exports.rs
priority: medium
type: chore
ordinal: 211500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`cargo test` in the default (unoptimized) profile is too slow (the user, 2026-11-16). Make the test build optimized while keeping `debug_assertions` and overflow checks, so plain `cargo test` (AGENTS.md, CI, the agents) stays the command.

- Add to the workspace Cargo.toml a `[profile.test]` with `opt-level = 3` (it inherits debug-assertions and overflow-checks from dev) and a light debug info (e.g. `debug = "line-tables-only"`). Check whether every unit of `cargo test` (dependencies, the libraries the integration tests link) is built with it; if dependencies stay unoptimized, add `[profile.dev.package."*"] opt-level = 3` (or another setting) and say why. Plain `cargo build`/`cargo run` stay as they are unless the change above affects them on purpose.
- Several tests use `cfg!(debug_assertions)` as "slow build, run a reduced workload": corpus_golden and corpus_mods skip the shipped songs, export_equal runs 10 of 164 cases, merge_tests 40 of 400 cases, frame_cost 2 of 200 runs (grep for more). With an optimized test build they keep the reduced path. Measure each full workload under the new profile and switch to the full set where it is cheap enough for every run (state the time); keep a reduced path only with a better key than debug_assertions (e.g. an `ORB_FULL_TESTS`-style variable documented in docs/building.md) and say which.
- `crates/orb-plugin/tests/exports.rs:38` uses `cfg!(debug_assertions)` to find the built library under target/debug or target/release: keep it correct (the test profile writes to target/debug).
- Bit-exactness: the engine's results must not change with opt-level (Rust does not contract or reorder floats); the corpus tests passing under the new profile is the check.
- Update AGENTS.md "Build and test" (the golden test's debug/release note, timings) and CI if needed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Cargo.toml has the optimized test profile with debug assertions and overflow checks on; a test or a note shows debug_assertions is true under cargo test
- [ ] #2 The wall time of a clean and of an incremental `cargo test` (whole workspace) before and after is recorded in the notes
- [ ] #3 Each cfg!(debug_assertions) workload switch is either removed (full set always, with its time) or keyed on a documented alternative
- [ ] #4 The corpus tests (corpus_golden, corpus_mods where inputs exist) pass under the new profile
- [ ] #5 AGENTS.md and docs/building.md describe the new behavior; CI passes
- [ ] #6 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
