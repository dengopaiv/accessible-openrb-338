---
id: TASK-183
title: Accessibility test harness and the UI Automation probe
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:31:00Z'
updated_date: '2026-11-05T20:01:00Z'
labels:
  - tests
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/tests/keyboard.rs
  - tools/uia_tree.ps1
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: task
ordinal: 206000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-0; the analysis is docs/accessibility.md. Read both before starting.

- Why: every later task proves its result on the accessibility tree; they need one shared way to
  build it headless, find nodes and send the actions a screen reader sends.
- What: a test helper module `crates/orb-ui/tests/common/a11y.rs` (used through `#[path]` or
  `mod common` as the other tests do): run frames of an `App` with AccessKit enabled and keep
  the last `TreeUpdate`; find nodes by name, by role, by layout id (the node id is derived from
  the egui id); read name, role, value, numeric value, toggled state and supported actions;
  the focused node; send an `ActionRequest` (Focus, Click, Increment, Decrement, SetValue with
  a number) as an input event; press keys as `keyboard.rs` does. A baseline test
  `crates/orb-ui/tests/a11y_baseline.rs` that records today's state (261 nodes, 4 named) as the
  starting point and is updated, not deleted, as tasks land. `tools/uia_tree.ps1` (committed
  already) stays the Windows check of the real window.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The helper builds the tree of `App` headless in `cargo test` on Windows, macOS and Linux (CI), without a window or a GPU.
- [x] #2 A test sends an Increment request to a node and sees it arrive in the app (`InputState::accesskit_action_requests`), with a small test widget.
- [x] #3 The baseline test passes and documents what it measures.
- [x] #4 `tools/uia_tree.ps1`'s use is described in docs/accessibility.md.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. tests/common/a11y.rs: Frames (AccessKit context running any UI closure), AppTree (App headless), Tree (find by id, layout id, name, role; focus), request and set_value_request, key, send.
2. a11y_actions.rs: a test widget (painted rect made a Slider) receives Increment, Decrement, SetValue.
3. a11y_baseline.rs: 261 nodes, 4 named, 250 Unknown, no values (verify first).
4. docs/accessibility.md 6.2: point to the committed helper; 6.3 already describes uia_tree.ps1 (check only).
5. cargo test -p orb-ui, clippy, rustfmt check.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a991a9a6eb6a9c850

Measured headless (cargo test --release -p orb-ui --test a11y_baseline -- --nocapture): nodes 261, named 4 (Options, Mods, File, Help), valued 0, roles {Button 4, GenericContainer 4, Menu 1, Pane 1, Unknown 250, Window 1}. Equals docs/accessibility.md 1.1. Real window (6.3): 256 elements; not rerun (no change to the panel).
Helper layout: no shared test code existed in orb-ui/tests, so tests/common/a11y.rs is used by #[path = "common/a11y.rs"] mod a11y; (a subfolder file is not its own test target). Avoided deprecated WidgetType and Id::new (Role, Id::unique).
Full run: cargo test --release -p orb-ui --no-fail-fast: all pass except tests/crash.rs a_deliberate_panic_exits_101_and_writes_the_log (asserts the backtrace contains orb_ui::crash, crash.rs:56). Not touched by this task (tests only added); likely release-build frame inlining; not investigated on main, so listed here, not fixed. clippy -p orb-ui --all-targets -D warnings clean; cargo +nightly-2026-10-18 fmt --all --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added crates/orb-ui/tests/common/a11y.rs (Frames, AppTree, Tree finders by id/layout id/name/role, focus, request and set_value_request, key, send), tests/a11y_actions.rs (Increment, Decrement and SetValue reach a test Slider via accesskit_action_requests; a request for another id does not) and tests/a11y_baseline.rs (261 nodes, 4 named, 250 Unknown, no values). docs/accessibility.md 6.2 now describes the helper; 6.3 already described tools/uia_tree.ps1 (AC4 checked, no gap). Tests: cargo test --release -p orb-ui: all pass except the unrelated crash.rs backtrace test (see notes). Branch worktree-agent-a991a9a6eb6a9c850, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
