---
id: TASK-237
title: 'Screen reader: the Initialize Song from Pattern Mode question'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:08:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-233
references:
  - crates/orb-ui/src/menu.rs
  - docs/accessibility_plan.md
priority: low
type: feature
ordinal: 251000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream's Edit > Initialize Song from Pattern Mode (task-306) asks first, in a modal drawn by App::init_song_ui (crates/orb-ui/src/menu.rs): a title, the original's alert text (INIT_SONG_TEXT) and the buttons Initialize (Enter) and Cancel (Escape). The fork's other prompts put the focus on their default button and speak their text in screen reader mode (the unsaved-changes prompt and the error box, task-193: App::prompt_focused, App::error_focused, App::window_focus and its modal_was).

Context: the merge of upstream f83c54f (task-233; the user's decision task-232, question 2: take everything, each new window gets its own m-7 task) brought this window in as upstream built it, for the mouse. The fork's screen reader work is docs/accessibility.md (the analysis, egui's AccessKit API, how to measure: tools/uia_tree.ps1) and docs/accessibility_plan.md (the plan and its ground rules: with screen reader mode off the app behaves exactly as upstream). Existing patterns to follow: crates/orb-ui/src/a11y/ (names.rs, announce.rs with App::set_status and App::announce, window.rs with win::line for focusable text lines, nav.rs), the Win enum and App::window_focus in app.rs (a window takes the focus as it opens in screen reader mode and gives it back when it closes), and the a11y_* tests in crates/orb-ui/tests (AppTree drives the app and reads the AccessKit tree). By-ear NVDA checks are not criteria here: they go to the user's pass, TASK-197.

What to do: treat this question as the fork's other prompts: in screen reader mode the focus goes to Initialize when it opens, the title and text are read (a node with the text, or the button's description), Tab reaches both buttons, the focus returns to where it was after the answer; the panel's keys wait while it is open. The two Edit commands' status lines are announced already (task-233 routes them through set_status).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In screen reader mode the question takes the focus on Initialize when it opens and gives it back after either answer (an a11y test)
- [ ] #2 The question's text is in the AccessKit tree where a screen reader reads it with the focused button (an a11y test)
- [ ] #3 With screen reader mode off the question behaves as upstream's
<!-- AC:END -->
