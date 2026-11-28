---
id: TASK-236
title: 'Screen reader: the 808 and 909 pattern editor windows'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-26T12:08:00Z'
updated_date: '2026-11-28T00:47:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-243
  - TASK-235
references:
  - crates/orb-ui/src/drum_editor.rs
  - crates/orb-ui/src/editor_frame.rs
  - crates/orb-ui/src/a11y/grid.rs
  - docs/accessibility_plan.md
priority: medium
type: feature
ordinal: 256000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream's 808 and 909 pattern editors (tasks 280 and 281; Edit > 808 / 909 Pattern Editor..., Ctrl/Cmd+E with that machine focused) are floating windows with painted drum rows, the accent row and, on the 909, low / high / flam cells with an entry switch, edited by clicks and drags. The fork already makes the panel's drum machines readable and editable as a grid in screen reader mode (a11y/grid.rs, task-191: arrows, Home, End, Space with Ctrl and Alt).

Context: the merge of upstream f83c54f (task-233; the user's decision task-232, question 2: take everything, each new window gets its own m-7 task) brought this window in as upstream built it, for the mouse. The fork's screen reader work is docs/accessibility.md (the analysis, egui's AccessKit API, how to measure: tools/uia_tree.ps1) and docs/accessibility_plan.md (the plan and its ground rules: with screen reader mode off the app behaves exactly as upstream). Existing patterns to follow: crates/orb-ui/src/a11y/ (names.rs, announce.rs with App::set_status and App::announce, window.rs with win::line for focusable text lines, nav.rs), the Win enum and App::window_focus in app.rs (a window takes the focus as it opens in screen reader mode and gives it back when it closes), and the a11y_* tests in crates/orb-ui/tests (AppTree drives the app and reads the AccessKit tree). By-ear NVDA checks are not criteria here: they go to the user's pass, TASK-197.

What to do: as for the 303 editors (the task before this one in the queue), first a Decide: task for the user (labels decision,ui, assigned @dengopaiv) with the options, for example: A. the window's grid as an AccessKit grid with the panel grid's keys (grid.rs); B. in screen reader mode Ctrl+E and the menu items go to the panel's grid of that machine instead; C. only the window's header controls accessible. Then build the choice. Reuse what the 303 task decided where it fits.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A Decide: task with the options and a recommendation exists, and the user's choice is recorded in it
- [x] #2 The chosen design is built: in screen reader mode an 808 and a 909 pattern can be read and edited from the keyboard through it, covered by a11y tests
- [x] #3 With screen reader mode off the editor windows draw and behave as upstream's
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. a11y/grid.rs: split the panel grid's logic from Panel so the window reuses it: state_word (a byte's word), row_moved/step_moved (the moves), readout_of (F2's steps), grid_node/cell_node (the AccessKit nodes). The panel grid keeps its behavior and its tests.
2. New a11y/editor_drum.rs (as editor303.rs): Part (Pattern spin, Steps grid, 909 Entry spin, Length spin, Shuffle check box, Shift left/right, Copy, Paste, Clear buttons), ids per machine, owner(), window_ui drawing the named controls and the 12x length grid (Accent first, as the panel grid) with grid.rs's keys (GridKey::of), act_said (Shift says what it did or 'Pattern A1 unchanged'; a cell key whose cell text is unchanged says the cell), readout for F2.
3. drum_editor.rs: in screen reader mode draw editor_drum instead of upstream's window_ui; new DrumAct::CellKey {column, step, entry} (one write_pattern, one Undo 'Edit Pattern'); each act judged on the bytes before and after it; the window follows Panel::edit_pattern; open_drum_editor refocuses an open window; the held modifiers do not switch the entry in this mode (Space's own modifiers choose it).
4. app.rs: Win::EditorDrum(Machine) in Win::ALL (10 windows, the u16 mask), win_open.
5. announce.rs: F2 in a drum window reads its pattern.
6. Docs: ui_spec 20.9 and notes in 24/25, Keyboard Shortcuts screen reader row, CHANGELOG line.
7. Tests: tests/a11y_drum_editor.rs (dialog, focus in/back, names and values, grid keys and words on both machines, entry, Undo and modified, Shift unchanged, F2, playback silence, mode off is upstream's) and unit tests; upstream's tests/drum_editor.rs unchanged.
8. TASK-244: one by-ear criterion. Then cargo test --release --workspace, clippy, nightly fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Supervisor, 2026-11-27: the user decided in task-243 (A, the whole window), so criterion 1's Decide task exists with the choice recorded; build option A, reusing task-235's header work.

Branch: worktree-agent-a5db36e340cbf2024 (agent-complex, 2026-11-28). Builds on task-235 (05ba65b); decision task-243, option A.

Research, 2026-11-28:
- The panel grid (a11y/grid.rs, App::drum_grid app.rs ~1966, App::grid_key ~2007) works on Panel (steps808/steps909, inst808/inst909, set_step808/set_step909); the window edits the link's 194 bytes through drum_editor_act. So grid.rs was split into Panel-free helpers the window uses: state_word (a byte's word; state_text now calls it), cell_words, readout_of (drum_readout now calls it), grid_node / cell_node (describe_grid / describe_cell now call them), row_moved / step_moved (grid::key now calls them). The panel grid's tests (a11y_grid.rs, grid unit tests) are the check that it behaves as before.
- The grid row index equals the window's step byte on both machines (grid.rs NAMES808/NAMES909 vs drum_editor LABELS_909 / Machine::labels): row 0 Accent = byte 0 (AC), rows 1..11 = bytes 1..11.
- egui 0.37 moves the focus on arrow keys (memory/mod.rs ~590) unless the focused widget's EventFilter takes them; the controller cancels arrow focus moves in windows (controller.rs ~121-147), and the window's grid also sets the panel grid's focus lock filter.
- The 909's Space on the window: Machine::clicked(column, v, entry) with the entry from Space's modifiers (Ctrl Low, Alt or Shift High, both Flam, else the Entry control's): behavior::tr909_step_value as the panel's Space, without the panel's Flam button (upstream's editor does not read it, ui_spec 25). Machine::clicked always changes the cell (every value maps to another), so act_said's 'cell unchanged' case is a guard that does not fire in practice.

Design choices (for the user's review; none large enough for a Decide task):
1. Grid row order: Accent first, then the 11 drum rows, as the panel's grid (the window's picture has AC at the bottom). Up from Bass drum goes to Accent, as on the panel.
2. The grid's cursor is the window's own (DrumEditor::sr_row/sr_step), not the panel's selected instrument: the window does not move the panel's selection frame. It starts on Bass drum, step 1, each time the window opens.
3. Tab order: Pattern, Steps, the 909's Entry, Length, Shuffle, Shift left, Shift right, Copy, Paste, Clear (Steps second, as the 303 window's Step editor).
4. The cells say the panel grid's words (on / accent / flam / accent and flam), as the brief asks; the Entry control's values say both: 'Cycle', 'Low (on)', 'High (accent)', 'Flam'.
5. In screen reader mode the held Ctrl/Alt do not switch the shown entry (upstream's Entry::held); Space's own modifiers choose what it enters, as on the panel grid.
6. Entering a hi-hat clears the other one silently, as on the panel grid.

Results, 2026-11-28:
- cargo test -p orb-ui --test a11y_drum_editor: 7 pass (first run); editor_drum unit tests 4 pass.
- cargo test --release --workspace: 1196 pass, 0 fail, 7 ignored (task-235's run was 1184; +11 new, and the grid refactor changed no existing test). Upstream's tests/drum_editor.rs is untouched and passes; a11y_grid.rs (the panel grid after the refactor) passes; frame_cost.rs passes.
- cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean.
- TASK-244 got its by-ear criterion for this task (the 909 and 808 windows with NVDA).
Reproduce: cargo test -p orb-ui --test a11y_drum_editor; cargo test -p orb-ui --lib editor_drum.

Review round 1 (2026-11-28), sent back for text fixes and one nit; fixed in the amended commit:
- The test count stays 1196: main after task-235 ran 1185 (the "1184" in the results note above is task-235's own round-1 count), and 1185 + 11 new = 1196. This round adds one unit test (below).
- key_help.rs, the row "In an 808 or 909 Pattern Editor": it now gives the 808 case on its own (Space toggles the cell, the modifiers do nothing) before the 909's (the Entry's value, Ctrl+Space on, Shift+Space accent, Ctrl+Alt+Space flam).
- Correction to design choice 1: upstream's painted window (mode off) has AC at the bottom, but the screen reader mode's own picture (editor_drum.rs steps_grid) draws the rows in the grid's order, Accent at the top. Both the grid and its picture are Accent first, as the panel's grid.
- The research note on act_said's "cell unchanged" guard: the reviewer's case (AC row, stored byte 1 or 3, word "on") was checked and does not make it fire. Space on the AC row goes through Machine::clicked -> behavior::tr909_step_value with accent_row (c != 0 -> 0; the 808's u8::from(v == 0)), so a stored 1 or 3 becomes 0, which reads "off". The new unit test editor_drum::tests::space_always_changes_the_cell_s_word checks every machine, row, stored byte 0..=255 and entry: Space always changes the cell's word. So the guard is only a guard, as noted; it is harmless anyway (it would say the cell once, which is true, and writes only what the act wrote).
- Nit: the close path when the link is gone (drum_editor.rs, drum_editor_ui's early return) now resets sr_had_focus too. The 303 window (pattern_editor.rs, pattern_editor_ui's early return when the link is gone) had the same gap and got the same reset.

Round 1 checks: cargo test --release -p orb-ui 724 pass, 0 fail, 4 ignored (editor_drum unit tests now 5); cargo clippy -p orb-ui --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Built the user's option A of task-243 (2026-11-27): in screen reader mode the 808 and 909 Pattern Editor windows are named Dialogs (Win::EditorDrum) that take the focus on Pattern when opened (Edit menu, Ctrl+E, again on an open window) and give it back on Escape or close. Controls in tab order: Pattern (spin, A1..D8), Steps (a Grid of 12 instruments, Accent first, by the pattern's length, with the panel drum grid's keys and words: arrows, Home, End, Space; on the 909 with Ctrl / Alt / Shift / Ctrl+Alt, on the 808 a toggle), the 909's Entry (spin: Cycle, Low (on), High (accent), Flam), Length, Shuffle, Shift left / right, Copy, Paste, Clear. Edits go through the new DrumAct::CellKey and the existing acts in drum_editor_act -> write_pattern (heard, modified, one Undo each); each act is judged on the bytes before and after it: Shift says 'Pattern A1 shifted right' or 'Pattern A1 unchanged' (nothing written); a cell's change is spoken by the grid's active cell itself. F2 reads the window's pattern in the panel's words. The window follows Panel::edit_pattern, so playback neither moves nor speaks it. grid.rs was split into Panel-free helpers (state_word, cell_words, readout_of, grid_node, cell_node, row_moved, step_moved) used by both the panel grid and the window. With the mode off the windows are upstream's (their tests unchanged). Criterion 1: decision task-243 exists with the user's choice recorded. Docs: ui_spec 20.9 and notes in 24/25, Keyboard Shortcuts screen reader row, CHANGELOG. Tests: cargo test --release --workspace 1196 pass, 0 fail (a11y_drum_editor 7, editor_drum unit 4); after review round 1 (help text, notes, the sr_had_focus reset on the no-link close path of the drum and 303 windows, one more unit test) cargo test --release -p orb-ui 724 pass, 0 fail (editor_drum unit 5); clippy and nightly fmt clean. Design choices for the user's review are in the notes (row order, own cursor, tab order, Entry words, held modifiers, silent hi-hat clear). Branch worktree-agent-a5db36e340cbf2024, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
