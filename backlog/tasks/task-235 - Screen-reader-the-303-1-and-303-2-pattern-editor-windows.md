---
id: TASK-235
title: 'Screen reader: the 303-1 and 303-2 pattern editor windows'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-26T12:08:00Z'
updated_date: '2026-11-28T00:05:00Z'
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
- [x] #1 A Decide: task with the options and a recommendation exists, and the user's choice is recorded in it
- [x] #2 The chosen design is built: in screen reader mode a 303 pattern can be read and edited from the keyboard through it, covered by a11y tests
- [x] #3 With screen reader mode off the editor windows draw and behave as upstream's
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Option A of task-242 (the user, 2026-11-27), screen reader mode only; mode off keeps upstream's window_ui untouched.
1. app.rs: Win::Editor303(k) for both windows (Win::ALL grows to 8; the window mask widened to u16 for task-236's two more); win_open reads editors[k].open; App::refocus_window so Ctrl+E / the Edit item on an already open editor moves the focus into it again.
2. New a11y/editor303.rs: the window's screen reader content. Controls in this tab order: Pattern (spin button, value A1..D8; Up/Down next/previous, PgUp/PgDn a bank, Home/End A1/D8, as the panel's pattern selector; covers the bank buttons, the number and its arrows), Step editor (a vertical Slider as the panel's, value 'Step 5 of 16: C sharp, accent'), Length (spin button, '16 steps'; Up/Down, PgUp/PgDn 10, Home/End 1/16), Shuffle (toggle button), Transpose up/down, Shift left/right, Copy, Paste, Clear (buttons). The window is a Dialog named by its title; the focus goes to Pattern on opening (focus_into_window) and back on closing; Escape closes it.
3. Step editor keys: Up/Down previous/next step (wrapping at 16), Home/End step 1 / the pattern's last, Left/Right nothing, Enter/Backspace next/previous; the programming keys through keys::map_screen_reader in Program Synth mode (pitch keys, - =, P, [, ', ;), with Options > Pitch Keys Make Notes. Edits go through a new Act::StepEdit in App::editor_act -> editor_edit -> write_pattern (heard, modified, Undo 'Edit Pattern'). Spoken: the focused node's value change; the announcer when the value cannot say it (the same note again: 'Step 1: rest, C'), as step303::press_effect. Shared words from step303.rs (step_text, compact, a new step_value).
4. The editor's pattern in screen reader mode follows Panel::edit_pattern (the pin of task-190/209) instead of the row's playing pattern, so playback never changes a focused editor (move-along silent); a pattern chosen in the header or on the panel moves it.
5. F2 with the focus in the editor reads its pattern ('303-1, pattern A3, 8 steps, shuffle off. 1 C accent, 2 rest, ...'). Transpose and Shift announce what they did; Copy/Paste/Clear already set_status.
6. Tests: crates/orb-ui/tests/a11y_pattern_editor.rs with AppTree on an App with a Link: the dialog, focus in and back, names/values, the keys and their writes (Undo, modified), speech, F2, no speech while playing, mode off unchanged (no new nodes; upstream tests pass). Unit tests in editor303.rs.
7. Docs: ui_spec 20.8 (new) and 23 pointer, Keyboard Shortcuts' screen reader group row, CHANGELOG line, TASK-244 criterion.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Supervisor, 2026-11-27: the user decided in task-242 (A, the whole window), so criterion 1's Decide task exists with the choice recorded; build option A.

Branch: worktree-agent-a41b55aa83ce450e8

Work notes (2026-11-28):
- Criterion 1: the Decide task is task-242 (Done, the user chose A, the whole window, 2026-11-27).
- New crates/orb-ui/src/a11y/editor303.rs: the window's content in screen reader mode, drawn by pattern_editor_ui in place of window_ui (mode off: window_ui untouched). Controls with fixed ids editor303::id(k, Part): Pattern (SpinButton), Step editor (vertical Slider, the panel's words), Length (SpinButton), Shuffle (CheckBox), Transpose up/down, Shift left/right, Copy, Paste, Clear (Buttons).
- Shared with step303.rs: step_value (factored out of edit_step_text), step_text, compact (now pub), EDITOR_NAME, Move. The programming keys come from keys::map_screen_reader (Program Synth mode, the window's 303), so the keys are the panel's by construction.
- Edits: new Act::StepEdit and pattern_editor::with_step_edit (pitch key keeps the rest unless Pitch Keys Make Notes), through editor_act -> editor_edit -> write_pattern (Undo 'Edit Pattern', modified).
- app.rs: Win::Editor303(k) (Win::ALL 8; window mask widened u8 -> u16 so task-236's two more fit), App::refocus_window for Ctrl+E / the menu item on an open editor.
- Move-along: in screen reader mode App::editor_follow takes Panel::edit_pattern (the pin), not rows[k].pattern; test playback_says_nothing_and_does_not_move_the_window (60x4 engine calls while playing plus adopt_pattern(0, 5): announcer serial unchanged, values unchanged).
- Failed first try: Escape closed nothing, because egui drops the focus on Escape as the frame begins (the same as editor_frame::value_field notes); fixed by Editor::sr_had_focus, the window's focus at the end of the last frame.
- Design choice taken without a Decide task: the header's bank buttons, number and arrows are one Pattern spin button (Up/Down, PgUp/PgDn a bank, Home/End A1/D8), as the panel's own pattern selector (task-187); the window has no pitch mode (the panel's stays the panel's); the programming keys work whatever the keyboard mode (Ctrl+F/G), since the list is a step editor.
- Spoken results: value/state changes of the focused control speak themselves (as the panel's controls, ui_spec 20.3); the announcer for the same note again ('Step 1: rest, C'), Transpose/Shift ('Pattern A1 transposed up', 'Pattern A1 shifted right'); Copy/Paste/Clear via set_status ('303-1: A1 copied').
- Tests: cargo test -p orb-ui --test a11y_pattern_editor: 6 pass; lib editor303 unit tests: 7 pass.

Final checks: cargo test --release --workspace 1184 passed, 0 failed, 7 ignored (orb-ui's a11y_pattern_editor 6, pattern_editor.rs upstream tests unchanged and passing, keyboard.rs, a11y_windows.rs, frame_cost.rs included); cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean. Main had not moved (d9f1d25), so no rebase was needed. TASK-244 got the by-ear criterion for this window.

Review round 1 (send back), fixed:
1. Transpose and Shift were announced even when they changed nothing. editor303::act_said now takes the pattern bytes just before and after the act and says 'Pattern A1 unchanged' when they are equal (nothing is written then; the Undo stays). Tests: unit transpose_and_shift_say_what_they_did, integration the_header_controls_work_from_the_keyboard_and_say_what_they_did (Shift on a pattern of alike steps).
2. Out::edited kept only the frame's first programming key's 'before' value. Removed: pattern_editor_ui now runs the window's acts one at a time with App::editor_pattern_bytes before and after each, and act_said judges each StepEdit on its own bytes. Unit test each_key_is_judged_on_its_own.
The pinned pattern on open is left as it was (the supervisor puts it to the user).
Checks: cargo test --release -p orb-ui 712 passed, 0 failed, 4 ignored; cargo clippy -p orb-ui --all-targets -D warnings clean; nightly fmt --check clean. Main unchanged (d9f1d25), no rebase. ui_spec 20.8 updated.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Built option A of task-242 (the user's choice of 2026-11-27, the whole window): in screen reader mode the 303-1 and 303-2 Pattern Editor windows are named Dialogs (Win::Editor303) that take the focus on Pattern as they open (Edit menu, Ctrl+E, again on an open window via App::refocus_window) and give it back on Escape or close. New crates/orb-ui/src/a11y/editor303.rs draws Pattern (spin button A1..D8; it replaces the bank buttons, number and arrows), Step editor (the 16 steps as a list, a vertical Slider with the panel step editor's words), Length, Shuffle, Transpose up/down, Shift left/right, Copy, Paste and Clear. The Step editor has the panel step editor's keys (step303.rs reused: Move, step_value, step_text, compact; the programming keys from keys::map_screen_reader), Pitch Keys Make Notes included; edits go through Act::StepEdit -> editor_act -> write_pattern (heard, modified, one Undo). Spoken: the focused control's value change; the announcer for the same note again, Transpose and Shift; Copy, Paste, Clear through set_status; F2 reads the window's pattern. In screen reader mode the window follows Panel::edit_pattern, so playback never moves or speaks it. With the mode off the window is upstream's, untouched. Choices taken without a Decide task (recorded in the notes): the single Pattern spin button, no pitch mode in the window, programming keys whatever the keyboard mode. Docs: ui_spec 20.8 (new) and a pointer in 23, Keyboard Shortcuts' screen reader group, CHANGELOG. Tests: cargo test --release --workspace 1184 pass, clippy and nightly fmt clean. Branch worktree-agent-a41b55aa83ce450e8, one commit. The by-ear check is TASK-244's new criterion.
<!-- SECTION:FINAL_SUMMARY:END -->
