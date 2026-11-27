---
id: TASK-237
title: 'Screen reader: the Initialize Song from Pattern Mode question'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:08:00Z'
updated_date: '2026-11-27T22:02:00Z'
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
- [x] #1 In screen reader mode the question takes the focus on Initialize when it opens and gives it back after either answer (an a11y test)
- [x] #2 The question's text is in the AccessKit tree where a screen reader reads it with the focused button (an a11y test)
- [x] #3 With screen reader mode off the question behaves as upstream's
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. init_song_ui: in screen reader mode name the modal a Dialog (title as name, INIT_SONG_TEXT as description) with a11y::window::dialog; Initialize takes the focus once (init_focused, reset when closed), as prompt_ui does for Save.
2. window_focus: count confirm_init as a modal so the focus is given back two frames after it closes.
3. Test in a11y_windows.rs through AppTree (App::ask_init_for_test opens the question without an engine).
4. CHANGELOG line; cargo test/clippy/fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a8d8b4407932700fb

Evidence: new test a11y_windows::the_init_song_question_takes_the_focus_on_initialize_and_gives_it_back (AppTree): Dialog named 'Initialize Song from Pattern Mode' with INIT_SONG_TEXT as description; focus on Initialize (Button); Tab reaches Cancel; Escape and Enter each give the focus back to the panel control; mode off: focus not moved, no description set. Failed first try: the mode-off check assumed no Dialog node at all, but egui's own modal node is one; narrowed to 'no description'. The Edit command needs the engine (can_init), so the test opens the question through App::ask_init_for_test. Also window_focus counts confirm_init as a modal (panel keys wait, focus returns after 2 frames). Mode-off code path is unchanged (all new code is gated on sr). cargo test --release --workspace: 1158 passed 0 failed; clippy -D warnings clean; nightly fmt --check clean. Deferred to TASK-244: by-ear NVDA check of the question (an acceptance criterion there).

Review round 1: (1) subject shortened to 72 or fewer characters; (2) the first edit to task-197 reverted (it is Done) and the by-ear check added as an acceptance criterion of task-244 instead; (3) the a11y test now walks the AccessKit parents of the focused Initialize button and requires one whose description is INIT_SONG_TEXT: it passes without a code change, so the tree nests them as needed; (4) App::ask_init_for_test is #[doc(hidden)]. Checks: cargo test --release -p orb-ui 685 pass, clippy -p orb-ui -D warnings clean, nightly fmt --check clean.

Review round 2: text fixes only: the notes and the final summary named task-197 for the by-ear check; they now name task-244. No code change.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
In screen reader mode Initialize Song's question is a named Dialog (title as name, the original's text as description) with Initialize inside it, takes the focus on Initialize once, Tab reaches Cancel, and the focus returns after either answer; the panel's keys already waited (asking()). Mode off: unchanged. Files: crates/orb-ui/src/menu.rs, app.rs, tests/a11y_windows.rs, CHANGELOG.md. Tests: workspace 1158 pass (orb-ui 685 after round 1), clippy and fmt clean. The by-ear NVDA check is an acceptance criterion of TASK-244.
<!-- SECTION:FINAL_SUMMARY:END -->
