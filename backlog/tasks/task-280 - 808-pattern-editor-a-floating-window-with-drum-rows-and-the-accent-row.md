---
id: TASK-280
title: '808 pattern editor: a floating window with drum rows and the accent row'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-22T09:48:00Z'
updated_date: '2026-11-22T15:16:00Z'
labels:
  - ui
milestone: m-9
dependencies: []
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/tests/pattern_editor.rs
  - crates/orb-plugin/tests/editor.rs
  - crates/orb-ui/src/key_help.rs
  - crates/orb-ui/src/menu.rs
  - docs/ui_spec.md
  - docs/RBS42.txt
priority: medium
type: feature
ordinal: 260100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** milestone m-9 adds pattern editors for the 303, 808 and 909. The 303 editor is done (task-274, crates/orb-ui/src/pattern_editor.rs). This task adds the TR-808's, in the same frame and with the same conventions. The user designed it on 2026-11-22.

**The data** (docs/RBS42.txt, the "808 " chunk; docs/ui_spec.md, the 808 section):
- A pattern is a shuffle byte, a length 1..16, and 16 steps × 12 bytes.
- The 12 bytes, in RBS column order: AC, BD, SD, LT/LC, MT/MC, HT/HC, RS/CL, CP/MA, CB, CY, OH, CH.
- Each byte is 0 or 1: there is no per-drum level on the 808. AC is the per-step accent.
- The LT/LC, MT/MC, HT/HC, RS/CL and CP/MA switches are song controllers (kit settings for every pattern), not pattern data. **The editor has no such switches** (the user's decision). The row labels show the names the panel's switches currently select, and update when the panel changes.
- Only what RBS stores is edited. No extra data.

**Layout:** like the 303 editor, a floating in-app egui window, non-resizable.
- Header: bank buttons, the pattern number field with left/right arrows (across banks; the 808 has no 1/2 switch), Length, Shuffle, and Copy / Paste / Clear.
- 11 drum rows in place of the piano roll, labeled on the left in the 303 editor's label column style and size.
- A separator, then the AC row (labeled "AC", tooltip "Accent: every drum on this step louder"), like the 303's flag rows.
- The shift arrows under the grid, centered. No transpose.
- The grid fills the width, and the step and current-step indication works as in the 303 editor.

**Editing:**
- A click toggles the cell (0 ↔ 1).
- A drag paints the first cell's new value along the same row only, as on the panel.
- Shift wraps within the length.
- Clear: like Cut but keeps the length and shuffle, as in the 303 editor.
- Copy and Paste go through the existing pattern clipboard.
- Every edit is one whole-pattern write through `App::write_pattern` (→ `Link::set_pattern`) and is undoable with the one-level Undo, as in the 303 editor.

**Selection:**
- In pattern mode the editor and the panel select each other's pattern.
- In song mode they are independent.
- There is no Follow button.

**Opening:**
- Edit menu "808 Pattern Editor..." (with "303 Pattern Editor..." and "909 Pattern Editor...", task-281).
- Cmd/Ctrl+E opens the editor for the panel's focused row: 303, 808, or 909 once task-281 is done. It focuses the window if it is already open and switches it to the active pattern.
- Works in the plugin too, as the 303 editor does.

**Keyboard shortcuts help (the user's requirement):** Help > Keyboard Shortcuts (crates/orb-ui/src/key_help.rs) must list every key and modifier the 808 editor supports, under its own group, including any keys and mouse modifiers it handles. Its bind tests must cover them, the way they cover the panel. A full audit of every context follows in task-282.

**Shared code:** the 909 editor (task-281) has the same frame. Factor the drum-grid editor so the 909 adds only its cell states and entry switch, rather than copying.

**Docs:** docs/ui_spec.md, the pattern editor section and the Edit menu.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Edit > 808 Pattern Editor... and Cmd/Ctrl+E with the 808 row focused open a floating, non-resizable 808 editor in the standalone app and the plugin; it focuses and switches to the active pattern when already open
- [x] #2 11 drum rows (labels follow the panel's current LT/LC, MT/MC, HT/HC, RS/CL, CP/MA switch positions; no switches in the editor) plus the AC row below a separator; bank, pattern number with arrows, Length, Shuffle, Shift (wraps), Copy/Paste/Clear as in the 303 editor
- [x] #3 Click toggles a cell; drag paints the first cell's new value along its row only; every edit is one pattern write and undoable; pattern-mode selection is linked with the panel, song mode independent
- [x] #4 Help > Keyboard Shortcuts lists every key and mouse modifier the 808 editor supports, and its tests cover them
- [x] #5 The drum-grid editor is shared code ready for the 909 (task-281); tests cover the layout, the edits, undo, the selection link and the plugin's Edit menu; docs/ui_spec.md updated; cargo test --workspace passes, clippy -D warnings and nightly fmt clean
- [x] #6 The user reviewed the dev app and the plugin before the agent review
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. New module crates/orb-ui/src/drum_editor.rs: the shared drum-grid editor. A Machine enum (Tr808 now; task-281 adds Tr909) gives the row (2), the title, the 11 drum row labels (from the panel's slot switches tr808.slot3..7_switch: 0 = LC MC HC CL MA, 1 = LT MT HT RS CP), the byte column of each row, the AC row, the value a click writes and the cell colors. The frame (header: banks, number with arrows across banks, Copy/Paste/Clear, Length, Shuffle; step numbers; 11 drum rows; divider; AC row; Shift arrows centered) and the gestures (click toggles, drag paints the first cell's new value along its row) are Machine-independent. One DrumEditor state per machine in App.
2. Reuse the 303 editor's helpers (arrow buttons, paint_columns, cell geometry, pointer, placing frames) by making them pub(crate) in pattern_editor.rs; no copies.
3. Edits: pure functions over the 194 bytes (cell, with_cell, shifted within the length, length, shuffle); every edit one App::write_pattern; a drag keeps the Undo of its first write (as the 303 editor). Clear = emptied_pattern(2) (length and shuffle kept). Copy/Paste via copy_pattern_at / paste_pattern_at.
4. Selection: pattern mode follows panel.rows[2].pattern both ways (select_pattern); song mode independent.
5. Opening: Edit > 808 Pattern Editor... (Cmd::DrumEditor(Machine)), and Cmd/Ctrl+E opens the editor of the focused row (303 rows the 303 editor, the 808 row the 808 editor; the 909 row keeps the 303 editor until task-281). The menu shows the Cmd+E text on the item the key opens now. The plugin's Edit menu gets the item too. Window place saved in ui.toml (pattern_editor_808_pos) in the standalone, as the 303's.
6. Help > Keyboard Shortcuts: a group for the 808 editor (Cmd+E with the 808 focused, click, drag, modifier-free clicks, the number/Length fields' drag/Shift+drag/type/Enter/Escape, Undo), with tests: a unit test pins the group's rows; integration tests run each listed gesture (with each modifier held: same result) and the field keys.
7. Tests: crates/orb-ui/tests/drum_editor.rs (opening, layout, labels follow the switches, click/drag/undo, shift/clear/copy/paste, length/shuffle, selection link in pattern and song mode, playing step), unit tests in drum_editor.rs, plugin tests/editor.rs (the Edit menu item and an edit with Undo).
8. Docs: docs/ui_spec.md section 24 (the 808 editor), the Edit menu row and Cmd+E in sections 16/18/20/23.
9. cargo test --workspace, clippy -D warnings, nightly fmt; build the dev app and the VST3 bundle for the user's review (AC 6).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aaf3cd1f515b85ddb (worktree .claude/worktrees/agent-aaf3cd1f515b85ddb), rebased on main 2166d01.

Work (2026-11-22):
- Shared frame: new crates/orb-ui/src/editor_frame.rs, moved out of pattern_editor.rs (no copies): Placing and show_window (the 3-frame placing, to the front, the saved place on release), the header controls (bank_buttons, number_field with its arrows, edit_buttons, length_shuffle), the grid geometry (cell_at, column_at, cell_rect), grid_press (Down / Held / Release on a grid), paint_columns, step_numbers, divider, the arrow buttons (ids salted per editor), shifted(bytes, step_len). The 303 editor now uses it; its 14 tests pass unchanged.
- New crates/orb-ui/src/drum_editor.rs: Machine (Tr808 now; task-281 adds Tr909: row, title, labels, the clicked value, the color and the saved place are per-Machine matches), a DrumEditor per machine in App::drum_editors, DrumAct, App::open_drum_editor / open_focused_editor / drum_editor_act / drum_editors_ui and the test accessors (drum_editor_cell, _hit, _labels, _window, _shown). Rows: BD SD slot3..7 CB CY OH CH (step bytes 1..11), a divider, AC (byte 0). The slot labels come from panel.switch_position(tr808.slotK_switch): non-zero LT MT HT RS CP, 0 LC MC HC CL MA (docs/ui_spec.md section 6). A new song's engine values are 0 (LC MC HC CL MA shown), seen in the test; my first assumption (LT..) was wrong.
- Edits: a click toggles 0/1 (a byte above 1 counts as on), a drag paints the first cell's new value along its row (CellMove uses only the column), Shift wraps in the length, Clear is emptied_pattern(2, ..), Copy/Paste through copy_pattern_at / paste_pattern_at (Paste enabled with an 808 clip), Length and Shuffle; each one App::write_pattern; a drag keeps the Undo of its first write.
- Opening: Cmd::DrumEditor(Machine); the Edit menu item "808 Pattern Editor..." under the 303's; Cmd/Ctrl+E calls App::open_focused_editor (TR-808 focused: the 808 editor; else the 303 editor, the TR-909 row too until task-281). The menu writes the Cmd+E text on the item the key opens now. Plugin: the item, no shortcut text.
- Settings: pattern_editor_808_pos (as the 303's), saved in the standalone only.
- Typed values: the shared frame's DragValues use update_while_editing(false) (Enter sets, one change), so the 303's number and Length now also set on Enter instead of per keystroke. Escape did not cancel in egui 0.37: the DragValue sets the text in the frame after the Escape (lost_focus covers two frames, and Escape is no longer pressed then). editor_frame::value_field drops a change in the 3 frames after an Escape on the focused field. First try (memory.has_focus at the Escape) failed: egui drops the focus on Escape as the frame begins; the fix keeps r.has_focus() of the last frame. A nested ui.data_mut with r.has_focus() inside deadlocked egui's RwLock (every test hung 10 s); fixed by reading it first.
- Help: key_help's Row has editor: Option<drum_editor::Input>; the group "808 Pattern Editor" is Input::ALL (OpenKey Cmd/Ctrl+E, Click, Drag, Modified (Shift/Cmd/Option/Ctrl on macOS, Shift/Ctrl/Alt elsewhere: the same as without), ValueDrag, ValueFineDrag (Shift+drag), ValueType (click, type, Enter), ValueCancel (Escape), Undo Cmd/Ctrl+Z), with the plugin differences in the text. The menu group's Cmd+E row names both editors. Unit test the_808_editor_group_lists_every_input_of_the_editor; tests/drum_editor.rs every_input_of_the_help_does_what_it_says matches on every Input.
- Tests: crates/orb-ui/tests/drum_editor.rs, 12 tests (cargo test -p orb-ui --test drum_editor); 6 unit tests in drum_editor.rs; plugin tests/editor.rs the_edit_menu_opens_the_808_pattern_editor_and_undoes_its_changes, and the Edit menu order list with the 808 item.
- Docs: docs/ui_spec.md section 24 (new), section 23 (typed values, the shared frame), 16 (Cmd+E), 18 (the Edit row, the Help text), 20 (the plugin's Edit menu).

Checks (2026-11-22): cargo test --workspace 1050 passed, 0 failed; cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean. Built for the user's review: target/OpenRB-338.app (scripts/macos-dev-app.sh, after cargo build --release -p orb-ui) and target/bundled/OpenRB-338.vst3 (cargo xtask bundle orb-plugin --release).
AC evidence: AC 1 tests/drum_editor.rs the_menu_item_and_cmd_e_open_it_on_the_808s_pattern_and_bring_it_to_the_front, the_header_the_rows_and_the_window_that_does_not_resize, plugin the_edit_menu_opens_the_808_pattern_editor_and_undoes_its_changes. AC 2 the_header_the_rows..., the_row_labels_follow_the_panels_switches, shift_clear_copy_paste_length_shuffle_and_their_undo. AC 3 clicks_toggle_cells_and_each_is_one_undo, a_drag_paints_the_first_cells_new_value_along_its_row_only, pattern_mode_selections_follow_each_other_both_ways, song_mode_selections_are_independent_and_never_recorded. AC 4 key_help the_808_editor_group_lists_every_input_of_the_editor and every_input_of_the_help_does_what_it_says. AC 5 drum_editor.rs + editor_frame.rs (Machine is the 909's extension point), the tests above, docs/ui_spec.md section 24, the checks above.
AC 6 (the user's review) is open: the task stays To Do until the user has reviewed the dev app and the plugin.

User sign-off (2026-11-22): the user reviewed the dev app (target/OpenRB-338.app) and the VST3 bundle (target/bundled/OpenRB-338.vst3) built from b5bb06b and said "looks good" (relayed by the supervisor). AC 6 checked on that evidence. The amend after it changes only the task file.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: the 808 pattern editor (task-280) is done and signed off by the user on 2026-11-22 ("looks good").

What it is: crates/orb-ui/src/drum_editor.rs, a floating, non-resizable "808 Pattern Editor" window, separate from the 303's:
- the header: banks A-D, the pattern number with arrows (A1..D8, stopping at the ends), Copy/Paste/Clear, Length and Shuffle;
- 11 drum rows whose labels follow the panel's slot switches (LT/LC, MT/MC, HT/HC, RS/CL, CP/MA), each with a full-name tooltip; the editor has no such switches;
- a divider, the AC row, and Shift centered at the bottom.
A click toggles a cell and a drag paints along its own row. Every edit is one App::write_pattern, so it is heard, shown on the panel and undoable. Pattern mode links the editor and the panel's selection; song mode keeps them apart.

Opening: Edit > 808 Pattern Editor... in the standalone and the plugin, and Ctrl/Cmd+E with the TR-808 focused.

Shared code: the frame both editors use is crates/orb-ui/src/editor_frame.rs (the window placing, the header controls, the grid geometry and painting, the mouse on a grid). The 303 editor moved onto it. The 909 (task-281) adds a Machine variant.

Help: Help > Keyboard Shortcuts has a group "808 Pattern Editor" built from drum_editor::Input::ALL, with each platform's key names and the plugin differences. A unit test requires each input once; tests/drum_editor.rs runs every input.

Decisions taken where the task left them open:
- A typed pattern number or Length is set on Enter, as one change, and Escape cancels it (the shared frame, so the 303 editor too). value_field works around egui 0.37, whose DragValue applies the text in the frame after an Escape.
- Ctrl/Cmd+E with the TR-909 focused opens the 303 editor until task-281.
- The Edit menu writes the Ctrl/Cmd+E text on the item the key opens now: the 808's with the TR-808 focused, else the 303's.
- The window's place is saved in ui.toml as pattern_editor_808_pos (standalone only), as the 303's.

Docs: docs/ui_spec.md section 24 (new), and sections 16, 18, 20 and 23.

Branch and commit: worktree-agent-aaf3cd1f515b85ddb, one commit (b5bb06b, amended with this task file).

Tests:
- cargo test --workspace: 1050 passed, 0 failed.
- cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check: clean.
- New tests: 12 in crates/orb-ui/tests/drum_editor.rs, 6 unit tests in drum_editor.rs, 1 in key_help.rs, and 1 in crates/orb-plugin/tests/editor.rs, plus the 808 item in that file's Edit-menu order test.

User review: the dev app and the VST3 bundle, signed off in one round.
<!-- SECTION:FINAL_SUMMARY:END -->
