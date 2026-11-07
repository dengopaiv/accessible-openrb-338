---
id: TASK-216
title: >-
  system_dialog test fails in debug: a dropped TexturesDelta (epaint
  debug_assert)
status: To Do
assignee: []
created_date: '2026-11-07T13:09:00Z'
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
- [ ] #1 cargo test -p orb-ui --test system_dialog passes in debug and in release
- [ ] #2 The notes name the place that dropped the TexturesDelta and why the fix keeps the app's behavior
<!-- AC:END -->
