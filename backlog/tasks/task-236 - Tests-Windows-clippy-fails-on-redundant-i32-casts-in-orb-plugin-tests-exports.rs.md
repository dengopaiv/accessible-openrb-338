---
id: TASK-236
title: >-
  Tests: Windows clippy fails on redundant i32 casts in orb-plugin
  tests/exports.rs
status: To Do
assignee:
  - '@agent-chore'
created_date: '2026-11-17T15:08:00Z'
updated_date: '2026-11-17T15:09:00Z'
labels:
  - tests
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/tests/exports.rs
  - >-
    https://github.com/pingw33n/OpenRB-338/actions/runs/44110450523/job/134134275626
  - task-226
priority: high
type: bug
ordinal: 100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
CI's "clippy and test (windows-latest)" job has failed on every push since 13b61ee (task-226) with clippy::unnecessary_cast in crates/orb-plugin/tests/exports.rs (lines 641, 643, 658, 712 on main a0a12ca): `ProcessModes_::kOffline as i32`, `ProcessModes_::kRealtime as i32`, `SymbolicSampleSizes_::kSample32 as i32` (twice). The vst3 crate's enum constants are i32 on Windows but u32 elsewhere, so the cast is needed on macOS/Linux and redundant on Windows. Run: https://github.com/pingw33n/OpenRB-338/actions/runs/44110450523/job/134134275626

Fix: cast to an inferred type where the target type is known (`as _`), which clippy's unnecessary_cast does not lint; for `mode`, give the binding its type (`let mode: i32 = if ... { ProcessModes_::kOffline as _ } else { ProcessModes_::kRealtime as _ };`). Search the rest of crates/ for the same pattern with these vst3 constants (any `Kind_::kX as i32` or `as u32`) and fix them alike. Do not use #[allow].
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No `as i32` / `as u32` cast of a vst3 enum constant remains in crates/ where the same code compiles on Windows (grep listed in the notes).
- [ ] #2 cargo clippy --all-targets -- -D warnings and cargo test pass on macOS; cargo +nightly fmt --all --check passes.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The supervisor verifies CI's windows-latest clippy-and-test job after the push (not an AC: it can only be checked after merge).
<!-- SECTION:NOTES:END -->
