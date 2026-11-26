---
id: TASK-236
title: 'Screen reader: the 808 and 909 pattern editor windows'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-26T12:08:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-233
references:
  - crates/orb-ui/src/drum_editor.rs
  - crates/orb-ui/src/editor_frame.rs
  - crates/orb-ui/src/a11y/grid.rs
  - docs/accessibility_plan.md
priority: medium
type: feature
ordinal: 253000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream's 808 and 909 pattern editors (tasks 280 and 281; Edit > 808 / 909 Pattern Editor..., Ctrl/Cmd+E with that machine focused) are floating windows with painted drum rows, the accent row and, on the 909, low / high / flam cells with an entry switch, edited by clicks and drags. The fork already makes the panel's drum machines readable and editable as a grid in screen reader mode (a11y/grid.rs, task-191: arrows, Home, End, Space with Ctrl and Alt).

Context: the merge of upstream f83c54f (task-233; the user's decision task-232, question 2: take everything, each new window gets its own m-7 task) brought this window in as upstream built it, for the mouse. The fork's screen reader work is docs/accessibility.md (the analysis, egui's AccessKit API, how to measure: tools/uia_tree.ps1) and docs/accessibility_plan.md (the plan and its ground rules: with screen reader mode off the app behaves exactly as upstream). Existing patterns to follow: crates/orb-ui/src/a11y/ (names.rs, announce.rs with App::set_status and App::announce, window.rs with win::line for focusable text lines, nav.rs), the Win enum and App::window_focus in app.rs (a window takes the focus as it opens in screen reader mode and gives it back when it closes), and the a11y_* tests in crates/orb-ui/tests (AppTree drives the app and reads the AccessKit tree). By-ear NVDA checks are not criteria here: they go to the user's pass, TASK-197.

What to do: as for the 303 editors (the task before this one in the queue), first a Decide: task for the user (labels decision,ui, assigned @dengopaiv) with the options, for example: A. the window's grid as an AccessKit grid with the panel grid's keys (grid.rs); B. in screen reader mode Ctrl+E and the menu items go to the panel's grid of that machine instead; C. only the window's header controls accessible. Then build the choice. Reuse what the 303 task decided where it fits.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A Decide: task with the options and a recommendation exists, and the user's choice is recorded in it
- [ ] #2 The chosen design is built: in screen reader mode an 808 and a 909 pattern can be read and edited from the keyboard through it, covered by a11y tests
- [ ] #3 With screen reader mode off the editor windows draw and behave as upstream's
<!-- AC:END -->
