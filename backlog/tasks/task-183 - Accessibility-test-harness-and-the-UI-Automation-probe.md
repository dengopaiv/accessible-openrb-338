---
id: TASK-183
title: Accessibility test harness and the UI Automation probe
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:31:00Z'
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
- [ ] #1 The helper builds the tree of `App` headless in `cargo test` on Windows, macOS and Linux (CI), without a window or a GPU.
- [ ] #2 A test sends an Increment request to a node and sees it arrive in the app (`InputState::accesskit_action_requests`), with a small test widget.
- [ ] #3 The baseline test passes and documents what it measures.
- [ ] #4 `tools/uia_tree.ps1`'s use is described in docs/accessibility.md.
<!-- AC:END -->
