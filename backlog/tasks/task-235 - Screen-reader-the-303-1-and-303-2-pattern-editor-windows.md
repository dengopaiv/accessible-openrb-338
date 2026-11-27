---
id: TASK-235
title: 'Screen reader: the 303-1 and 303-2 pattern editor windows'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-26T12:08:00Z'
updated_date: '2026-11-27T21:43:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-242
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/src/editor_frame.rs
  - crates/orb-ui/src/a11y/step303.rs
  - docs/accessibility_plan.md
priority: medium
type: feature
ordinal: 255000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream's 303 pattern editors (tasks 274 and 290; Edit > 303-1 / 303-2 Pattern Editor..., Ctrl/Cmd+E with that 303 focused) are floating windows with a painted 13-key piano roll over 16 steps and Up, Down, Accent and Slide rows, edited by clicks and drags. A screen reader sees painted cells as nothing. The fork already has a keyboard step editor for the 303 on the main panel in screen reader mode (a11y/step303.rs, task-190, with the pitch keys, Options > Pitch Keys Make Notes).

Context: the merge of upstream f83c54f (task-233; the user's decision task-232, question 2: take everything, each new window gets its own m-7 task) brought this window in as upstream built it, for the mouse. The fork's screen reader work is docs/accessibility.md (the analysis, egui's AccessKit API, how to measure: tools/uia_tree.ps1) and docs/accessibility_plan.md (the plan and its ground rules: with screen reader mode off the app behaves exactly as upstream). Existing patterns to follow: crates/orb-ui/src/a11y/ (names.rs, announce.rs with App::set_status and App::announce, window.rs with win::line for focusable text lines, nav.rs), the Win enum and App::window_focus in app.rs (a window takes the focus as it opens in screen reader mode and gives it back when it closes), and the a11y_* tests in crates/orb-ui/tests (AppTree drives the app and reads the AccessKit tree). By-ear NVDA checks are not criteria here: they go to the user's pass, TASK-197.

What to do: first decide with the user how the editor should work for a screen reader; that is the user's choice, so make it a Decide: task (labels decision,ui, assigned @dengopaiv) with the options and a recommendation, for example: A. the window as a grid of steps with the same keys as the panel's step editor (step303.rs), every cell a node; B. in screen reader mode Ctrl+E and the menu items move the focus to the panel's existing step editor of that 303 instead of opening the window; C. the window's header controls (303-1/2, bank, pattern, Copy, Paste, Clear, Length, Shuffle, Transpose, Shift) accessible, the roll left to the panel's step editor. This task depends on that decision; then build what was chosen.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A Decide: task with the options and a recommendation exists, and the user's choice is recorded in it
- [ ] #2 The chosen design is built: in screen reader mode a 303 pattern can be read and edited from the keyboard through it, covered by a11y tests
- [ ] #3 With screen reader mode off the editor windows draw and behave as upstream's
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Supervisor, 2026-11-27: the user decided in task-242 (A, the whole window), so criterion 1's Decide task exists with the choice recorded; build option A.
<!-- SECTION:NOTES:END -->
