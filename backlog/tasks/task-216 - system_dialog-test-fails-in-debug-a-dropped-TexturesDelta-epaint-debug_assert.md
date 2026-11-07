---
id: TASK-216
title: >-
  system_dialog test fails in debug: a dropped TexturesDelta (epaint
  debug_assert)
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-07T13:09:00Z'
updated_date: '2026-11-07T13:36:00Z'
labels:
  - tests
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/tests/system_dialog.rs
  - crates/orb-ui/src/system_dialog.rs
priority: medium
type: bug
ordinal: 234000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: `cargo test -p orb-ui` (debug) fails crates/orb-ui/tests/system_dialog.rs `a_name_the_dialog_did_not_confirm_asks_before_it_replaces_a_file` on main, found while finishing task-214 (2026-11-07; it fails the same way with task-214's changes stashed, at 22a05c3). The panic:

thread 'a_name_the_dialog_did_not_confirm_asks_before_it_replaces_a_file' panicked at epaint-0.37.0/src/textures.rs:337:9:
Dropped TexturesDelta with 1 unapplied deltas. Deltas need to be handled. If you want to drop this intentionally call `clear` before dropping.

That is a debug_assert in epaint's `Drop for TexturesDelta`: a frame's output (FullOutput.textures_delta) is dropped without being applied or cleared. Release builds skip debug_assert, so `cargo test --release` passes; CI runs debug `cargo test`, so CI fails on this test too.

What: find where the test (or the code it drives, the system file dialog path of task-194, crates/orb-ui/src/system_dialog.rs) drops a FullOutput / TexturesDelta, and clear or apply it, as the other tests' harness does. The fix must not change the app's behavior; if the cause is in the app rather than the test, say so in the notes.

Reproduce: cargo test -p orb-ui --test system_dialog
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 cargo test -p orb-ui --test system_dialog passes in debug and in release
- [x] #2 The notes name the place that dropped the TexturesDelta and why the fix keeps the app's behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reproduce with cargo test -p orb-ui --test system_dialog (debug).
2. The test helper replace_frame (crates/orb-ui/tests/system_dialog.rs:264) drops ctx.run_ui's FullOutput with let _ = ...; every other harness clears out.textures_delta first. Clear it the same way.
3. Run the test in debug and release, clippy and nightly fmt check.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-216-dropped-textures-delta (in the main checkout, done by the supervisor directly: a one-line test fix).

Cause: the test helper replace_frame (crates/orb-ui/tests/system_dialog.rs:264, added in 771ce7c 'Export dialog keeps the format the user picks') dropped ctx.run_ui's FullOutput with let _ = ..., so its textures_delta (the frame's texture uploads, which no renderer applies in a test) was dropped unapplied and epaint 0.37's Drop for TexturesDelta debug_assert fired in debug builds. Every other harness (browser, error_box, frame_cost, interaction, keyboard, menu, mod_name, part_textures, repaint, song_mode_save, switch_values) clears out.textures_delta first. Fix: the same clear in replace_frame. The app's code is not changed (only the test helper), so its behavior is the same; the only run_ui in the file is this one.
Before: system_dialog 10 passed, 1 failed in debug (a_name_the_dialog_did_not_confirm_asks_before_it_replaces_a_file). After: 11 passed in debug and 11 in release; clippy -p orb-ui --all-targets -D warnings clean; nightly fmt --check clean.

Merged by the supervisor 2026-11-07: review merge on the first round, no defects (the reviewer's note: the assignee is @supervisor, who made the change directly).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
crates/orb-ui/tests/system_dialog.rs: the helper replace_frame now clears the frame's textures_delta before dropping it, as every other test harness does; dropping it unapplied tripped epaint 0.37's debug_assert in debug builds. Test-only change; the app is unchanged. system_dialog: 11 pass in debug (was 10 pass, 1 fail) and in release; clippy and nightly fmt clean. Branch task-216-dropped-textures-delta.
<!-- SECTION:FINAL_SUMMARY:END -->
