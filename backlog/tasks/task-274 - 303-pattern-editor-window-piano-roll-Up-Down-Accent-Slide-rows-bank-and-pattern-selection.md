---
id: TASK-274
title: >-
  303 pattern editor window: piano roll, Up/Down/Accent/Slide rows, bank and
  pattern selection
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T17:52:00Z'
updated_date: '2026-11-22T06:36:00Z'
labels:
  - ui
milestone: m-9
dependencies: []
references:
  - docs/ui_spec.md
  - docs/RBS42.txt
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/shortcuts.rs
  - crates/orb-ui/src/controller.rs
  - crates/orb-ui/src/app.rs
priority: medium
type: feature
ordinal: 259500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Designed with the user on 2026-11-21. It's a floating editor for one 303 pattern: a step grid with a 13-key piano roll, inspired by modern 303 pattern editors. It edits only what RBS 4.2 stores for a 303 pattern (docs/RBS42.txt, the "303 " chunk); no new song data. The 808 and 909 editors are later tasks in m-9.

**Layout** (the user's wireframe, top to bottom):
- **Top row,** all standard egui controls:
  - a 303-1 / 303-2 switch;
  - bank buttons A B C D;
  - the pattern number 1-8 as a `DragValue`: drag up or down, or click to type;
  - Copy, Paste and Clear;
  - Length 1-16, also a `DragValue`;
  - a Shuffle toggle: the pattern's own shuffle on/off byte, not the song's shuffle amount.
- **Note grid:**
  - a vertical 13-key piano roll on the left, C to the C an octave above. Pitch 0x0c is real data, so the 13th key is needed.
  - 16 step columns. Black-key rows look different from white-key rows. Colors come from the app's palette, not the skin, so the editor looks the same in Classic and Modern.
  - The step that's playing is shown.
  - Transpose up/down buttons at the grid's right.
  - Shift left/right buttons under the grid.
- **Flag grid:** four rows, Up, Down, Accent and Slide, over the same 16 columns.

**Behavior:**
- **Notes:**
  - A click in a step's column at a pitch makes the step a note at that pitch.
  - A click on the step's lit cell makes it a pause.
  - Pauses are not shown: the column is empty. A pause's stored pitch is kept, unchanged and unshown.
- **Flags:**
  - Up, Down, Accent and Slide toggle per step.
  - Up and Down are independent, as in the file; both set cancel each other (manual page 45, docs/ui_spec.md).
- **Drag painting:**
  - In the piano roll, a drag paints wherever the mouse points: each step it crosses becomes a note at the pitch under the mouse.
  - In the flag rows, a drag paints only in the row it started in, with the value set by the first cell.
- **Steps beyond the length** are dimmed but still editable.
- **Shift < >** rotates the steps, wrapping within the pattern length.
- **Transpose ^ v** moves every step's pitch by one semitone, wrapping over the 13 rows (12 up goes to 0, 0 down goes to 12). The Up and Down flags don't change.
- **Copy and Paste** use the same pattern clipboard as Edit > Copy/Paste Pattern (docs/ui_spec.md section 16), on the editor's pattern.
- **Clear** leaves the empty pattern that Edit > Cut Pattern leaves, but keeps the pattern's length and shuffle switch, and doesn't touch the clipboard.
- **Undo:** every editor change goes through the existing one-level undo (`menu::Undo`, Cmd+Z), and a drag counts as one change.
- **Live:** edits go through the same model and controller path as the panel's own pattern edits, with no second path, so they're heard at once. The song is marked modified, and the panel's step lights update.

**Pattern selection:**
- **Pattern mode:** choosing 303-1/2, a bank or a number in the editor selects that pattern on the panel too, so you hear it. Selecting a pattern on the panel changes the editor too.
- **Song mode:** the two are independent. The editor's selector only chooses what the editor shows; it never changes the panel's selection and never records. The panel's playing pattern doesn't move the editor.
- There is no Follow button: the user dropped it.

**The window:**
- It floats inside the main window (`egui::Window`): draggable and closable, not resizable. It's the same in the standalone and in the plugin, with no separate OS window.
- **Opening:**
  - from Edit > 303 Pattern Editor, and with Cmd+E (Ctrl+E) in the standalone; the plugin's editor has no keyboard shortcuts (task-201), so there it's the menu only.
  - It opens on the selected 303's active pattern. If it's already open, it comes to the front and switches to that pattern.
- Its position is saved in ui.toml in the standalone, and its open state isn't.

**Docs:** add a docs/ui_spec.md section (the window, the controls, the selection rules), and the Edit menu row and the Cmd+E shortcut in its menu and shortcut tables.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Edit > 303 Pattern Editor and Cmd+E (standalone) open the floating editor on the selected 303's active pattern; when it's open, they bring it to the front and switch it to that pattern; it works in the plugin from the menu
- [x] #2 The top row has the 303-1/2 switch, banks A-D, a 1-8 DragValue, Copy, Paste, Clear, a Length 1-16 DragValue and a Shuffle toggle; the window is not resizable; its position is kept in ui.toml in the standalone
- [x] #3 The note grid has a 13-key piano roll with distinct black and white rows in the app palette and shows the playing step; pauses are not shown, and a pause's stored pitch is kept
- [x] #4 Clicks and drags edit as specified: the piano roll paints along the mouse; flag rows paint only within their row with the first cell's value; Up and Down are independent; steps beyond the length are dimmed and editable
- [x] #5 Shift wraps within the length; Transpose wraps over the 13 rows and leaves the Up/Down flags; Copy and Paste share the Edit menu's pattern clipboard; Clear leaves the Cut Pattern result but keeps the length and shuffle
- [x] #6 Each change (a drag is one) is undoable with the existing one-level Undo; edits are heard at once, mark the song modified and update the panel, through the existing edit path with no second one
- [x] #7 Pattern mode: editor and panel selection follow each other both ways. Song mode: the editor's selection is independent and never changes the panel or records
- [x] #8 Tests cover the step edits, drag painting, shift, transpose, clear, the undo of each and both selection rules; docs/ui_spec.md describes the editor and its menu item and shortcut
- [x] #9 The user reviewed a dev app (and the plugin) before the agent review
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. New module crates/orb-ui/src/pattern_editor.rs: the editor's state (open, 303, pattern, the gesture of a drag, the window-front request) and pure functions over a 303 pattern's 98 bytes (note, pause, flag, shift within the length, transpose over 13 rows, length, shuffle), with unit tests.
2. One edit path: every editor change computes the pattern's new bytes and goes through App::write_pattern -> Link::set_pattern (the Edit menu's and the 303 Clear's whole-pattern path: msg 41 sub 44, the mirror, the panel's views, the song marked modified, the one-level Undo). A drag keeps the undo bytes of its start, so it is one change. Copy and Paste share App::clip through copy_pattern/paste_pattern refactored to take the row and pattern; Clear is the emptied pattern of Cut without touching the clipboard.
3. Selection: in pattern mode the editor shows rows[k].pattern and its 303/bank/number select on the panel (App::select_pattern), and a pattern change of a 303 row moves the editor to it; in song mode the editor keeps its own (303, pattern) and never touches the panel.
4. The window: egui::Window, fixed id, not resizable, not collapsible, closable; top row (303-1/2, A-D, 1-8 DragValue, Copy, Paste, Clear, Length DragValue, Shuffle), the 13-row piano roll with the step grid (app palette, black/white rows, playing step, dimmed beyond the length), Transpose up/down at the right, Shift left/right below, the Up/Down/Accent/Slide rows. Position kept in ui.toml (pattern_editor_pos) in the standalone only.
5. Opening: Edit > 303 Pattern Editor (also in the plugin, whose Edit menu then holds only that item) and Cmd/Ctrl+E (shortcuts::PATTERN_EDITOR, standalone); opening again brings the window to the front on the selected 303's active pattern.
6. Tests: tests/pattern_editor.rs (headless App with a Link): step edits, drag painting through pointer events, shift, transpose, clear, copy/paste, undo of each, both selection rules, the menu item and Cmd+E, the settings key; key_help line; plugin editor test for the menu.
7. Docs: docs/ui_spec.md new section (window, controls, selection rules), the Edit menu row in section 18 and section 20, key_help and the keyboard table.
8. cargo test, clippy, nightly fmt; dev app and plugin bundle for the user's review.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a7175cbca922f3106

Implementation (2026-11-21):
- crates/orb-ui/src/pattern_editor.rs: the editor state (App::editor), the pure edits over a 303 pattern's 98 bytes (with_note, with_pause, with_flag, shifted, transposed, with_length, with_shuffle, crossed), Act (what the window asks for) run by App::editor_act, and the window (App::pattern_editor_ui, drawn in App::show after the Info window).
- The one edit path: every editor change computes the pattern's new bytes and goes through App::write_pattern -> Link::set_pattern, the path of Edit > Paste Pattern and the 303's Clear (msg 41 sub 44, engine frame/edit.rs `pattern` copies it whole; the link mirror; fill_views updates the panel's 303 step views and Steps display). No second path: the Step303 per-field events stay the panel's own; the editor uses the panel's whole-pattern path because its Shift, Transpose, Clear, Paste, Length and Shuffle of any pattern (song mode: not the selected one) are whole-pattern changes, and one path for all of them keeps Undo one mechanism. Before each write the panel's pending events are noted and forwarded (a panel edit forgets the Undo first).
- Undo: write_pattern sets the one-level Undo; a drag keeps the Undo of its first write (editor_edit with keep). Names: Edit Pattern, Shift Pattern, Transpose Pattern, Clear Pattern, Paste Pattern, Pattern Length, Pattern Shuffle.
- menu.rs: Cmd::PatternEditor; Edit menu item "303 Pattern Editor" after a separator (Ctrl/Cmd+E shown); the plugin now has an Edit menu with that item only. copy_pattern/paste_pattern split into copy_pattern_at/paste_pattern_at for the editor's Copy/Paste on its own pattern; Undo row/pattern, ROW_NAMES, pattern_name, select_pattern made pub(crate).
- shortcuts::PATTERN_EDITOR (Cmd+E), consumed in App::show with Cmd+F/G (standalone only); key_help row "Edit > 303 Pattern Editor".
- settings.rs: pattern_editor_pos = [x, y] (i32 points), setter set_pattern_editor_pos, merged like theme; saved only in the standalone when the window is let go after a mouse move.
- Selection: pattern mode: editor shows rows[synth].pattern; a bank/number selects it on the panel via App::select_pattern; a change of exactly one 303 row's selection moves the editor to that 303 (a song open changing both keeps the 303); entering pattern mode takes the panel's selection. Song mode: own (synth, pattern), never touches the panel; the 303 switch keeps the pattern number.

Findings while building it (egui 0.37):
- `ui.put` advances the layout cursor below the widget: the Transpose buttons put beside the grid moved the next rows up into the grid (the flag rows overlapped the bottom of the piano roll; a click on low C hit Down). Fixed with ui.interact plus painting.
- Window::current_pos holds only for the first frame of a title-bar-dragged window (window.rs ~607: the stored pivot is put back after Area::begin), and the first (sizing) frame measures at default_area_size 600x400 and clamps x to 24 on a 624-point window. An opening now sets the AreaState's place directly for three frames (PLACE_FRAMES).
- The top row was 665 points wide (wider than the smallest 624-point window); compacted (item spacing 4, button padding 5x1, the Length label as the DragValue's prefix) to 529.

Tests: crates/orb-ui/tests/pattern_editor.rs (12 tests at this first build, 14 after the review rounds below: menu item and Cmd+E open/front/switch; top row and no resize, close; place in ui.toml and reopened there; clicks: note, move, pause keeps pitch, panel views, engine bytes, modified; flags incl. Up+Down; drags on the piano roll along the line with frame skips and one Undo; drag from a lit cell paints; flag drag stays in its row with the first cell's value and its Undo; dimmed and editable beyond the length; shift/transpose/clear/copy/paste and the undo of each, paste reaches the engine and the menu paste takes the editor's copy; shuffle and a Length drag undone as one; a panel step edit forgets the editor's Undo; pattern-mode both-way follow; song-mode independence with record on and tracks unchanged; the playing step shown); unit tests in pattern_editor.rs (7); orb-plugin tests/editor.rs the_edit_menu_opens_the_303_pattern_editor, and three plugin menu tests now find their menus along the bar (the new Edit title moved them).
Commands: cargo test -p orb-ui --test pattern_editor; cargo test -p orb-plugin --test editor; cargo test (all pass, 0 failed); cargo clippy --all-targets -- -D warnings (clean); cargo +nightly fmt --all --check (clean).

Open for the user's review: the plugin's Edit menu has no Undo (section 20 keeps Undo out of it); an editor change in the plugin cannot be undone. Adding Undo there is a one-line change if wanted.

User review, round 1 (2026-11-21), layout:
1. Row labels larger: the note names and the Up/Down/Accent/Slide names at 12.5 points (were 9.5 and 11); the label column 54 points (was 44) for Accent.
2. Shift: the label dropped; its arrows at the bottom under the flag rows, centered under the step grid. The flag rows now follow the piano roll with a 9-point gap and a divider line, nothing between them.
3. Transpose: the pair centered vertically on the piano roll.
4. One arrow size for the four Shift and Transpose buttons: 24 x 20 points (const ARROW).
5. The step columns widen to fill the top row's width up to the Transpose column (cell width = grid width / 16, at least 22); the hit tests and painting use the grid's own column width (col_w).
6. Black keys: every key now spans the key column's full width, so the black keys' names end at the same right edge as the white keys' (before, black keys were 12 points shorter and their names ended 12 points further left). If the user wants the shorter black keys back with only the text moved, that is a one-line change.
7. A horizontal line on the grid where two white rows meet: E-F and B-high C (WHITE_PAIRS).
8. Pattern number: left/right arrow buttons around the 1-8 value step it by one in the bank and wrap (8 to 1, 1 to 8); in pattern mode they select on the panel as the value does (same Act::Select).
The window is now about 569 points wide (was 529), still inside the smallest 624-point main window. The plugin Undo question stays open; no Undo added.
Tests: tests/pattern_editor.rs the_layout_of_the_users_first_review (gap and no buttons between the grids, Shift centered under the grid below the flags, Transpose centered on the roll, one size, columns filling to the Transpose column at the window's edge) and the_number_arrows_step_and_wrap; its_place_is_kept_in_the_settings_file now drags left (the wider window is held by the 624-point screen on the right). cargo test: all pass, 0 failed; clippy and nightly fmt clean. Dev app and plugin bundles rebuilt.

User review, round 2 (2026-11-21):
1. Undo in the plugin: the plugin's Edit menu now has Undo (the same one-level Undo, named after the change, "Undo Edit Pattern" and so on, no shortcut text) above the 303 Pattern Editor item; it undoes the editor's changes and the plugin's other whole-pattern writes. The plugin still takes no keys. menu.rs: the Undo item is drawn for both, with key(&shortcuts::UNDO) (None in a plugin); the pattern commands stay standalone-only.
2. The pattern number's arrows step through one sequence of 64: 303-1 A1..D8 then 303-2 A1..D8 (pattern_editor::step_pattern): 8 goes on to the next bank's 1, 303-1 D8 to 303-2 A1, left the reverse; they stop at 303-1 A1 and 303-2 D8, where the arrow is disabled (add_enabled_ui). The 303 switch and the banks follow (Act::Select with the new 303); in pattern mode the panel selects the pattern, the other 303's row included (editor_select -> select_pattern).
3. Labels bigger still: the note names and the flag names at 15.5 points (were 12.5); rows 19 points high (were 14); the label column 66 points (was 54). The window stays inside the 624 x 532 main window.
Tests: unit test the_number_arrows_run_one_sequence_of_64_and_stop_at_its_ends; tests/pattern_editor.rs the_number_arrows_cross_banks_and_303s_and_stop_at_the_ends (replaces the wrap test) and the layout test now checks the window inside the smallest main window and that the 15.5-point labels fit their column and row; orb-plugin tests/editor.rs the_edit_menu_opens_the_303_pattern_editor_and_undoes_its_changes (Undo drawn, disabled at first, no Ctrl/Cmd+Z text, then "Undo Edit Pattern" takes the edit back). cargo test: all pass, 0 failed; clippy and nightly fmt clean. docs/ui_spec.md sections 18 (Edit row), 20 and 23 updated. Dev app and plugin bundles rebuilt.

User review, round 3 (2026-11-21):
1. The item is "303 Pattern Editor..." (menu::PATTERN_EDITOR_ITEM) in the standalone and the plugin; key_help, docs and tests follow. The window's title stays "303 Pattern Editor".
2. The plugin's Edit menu now has every item of the standalone's, same order and names, no shortcut texts: Undo, Cut Pattern, Copy Pattern, Paste Pattern, Duplicate Pattern, 303 Pattern Editor... (menu.rs: the items drawn for both with key(sc), None in a plugin).
   - Copy and Cut work in the plugin and put the marker on the system clipboard: egui-baseview 0.9 (window.rs:663) writes egui's OutputCommand::CopyText to the clipboard through copypasta.
   - One difference, Paste Pattern: the standalone's item sends ViewportCommand::RequestPaste and pastes only when the clipboard brings the marker back (section 16). egui-baseview 0.9 does not handle RequestPaste (no match in its source); it reads the clipboard only on a paste key event (window.rs:876-880), and the plugin leaves keys to the host. So in the plugin, Paste Pattern pastes the app's clipboard directly (as the 303 editor's Paste does), without the marker check: text copied elsewhere since the last Copy does not block it. Reading the clipboard ourselves would need a clipboard crate in orb-plugin and a hook through Embed; not done, reported to the user.
   - Nothing else is standalone-only in the Edit menu.
Tests: orb-plugin tests/editor.rs the_edit_menu_has_the_standalones_items_without_shortcuts (all six items, in order, no Ctrl/Cmd texts) and the_edit_menus_pattern_commands_work_in_the_plugin (Copy: app clipboard and the marker CopyText; Paste into A3 directly, no RequestPaste; Undo; Cut and Undo; Duplicate into the next empty pattern, selected); the editor test renamed for the item. cargo test: 1021 passed, 0 failed; clippy and nightly fmt clean. docs/ui_spec.md sections 18 (Edit row), 20 and 23 updated. Dev app and plugin bundles rebuilt.

User sign-off (2026-11-22): the user reviewed the dev app and the plugin bundles at 2790875 (after review rounds 1-3 above) and said "looks good". Decided with the sign-off: the plugin's Paste Pattern stays as it is, pasting the app's clipboard without the marker check, because egui-baseview 0.9 reads the system clipboard only on a paste key event, which the plugin leaves to the host (it does not answer egui's RequestPaste). The amends after 2790875 changed only the task file, the tests and the docs (the agent review round below), not the editor's code.

Agent review round 1 (2026-11-22), sent back with three fixes:
1. AC 8 evidence: tests/pattern_editor.rs clicks_make_notes_and_pauses_and_toggle_the_flags now asserts Undo for a single note click (back to the empty pattern), a pause (back to the lit note; the pause comes from Act::Release) and a single flag click (Act::FlagDown; back to all four flags on), each by comparing the whole pattern with the one before.
2. docs/ui_spec.md section 16 (the key routing, near line 1075) now names Ctrl/Cmd+E: taken in the standalone by App::show with Ctrl/Cmd+F and G, also while a text field has the focus, not while the browser, prompt or error box is open.
3. These notes: the test count above (14 in tests/pattern_editor.rs now) and the sign-off line (the later amends touch no editor code).
Checks: cargo test -p orb-ui -p orb-plugin 540 passed, 0 failed; cargo +nightly fmt --all --check and cargo clippy --all-targets -- -D warnings clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: the 303 pattern editor (task-274) is done as specified and as adjusted in three rounds of the user's review (2026-11-21), signed off by the user on 2026-11-22 ("looks good").

What it is: crates/orb-ui/src/pattern_editor.rs, a floating egui window (not resizable, closable) that edits one TB-303 pattern:
- the top row: 303-1/2, banks A-D, the pattern number as a DragValue between arrows that step through 303-1 A1..D8 then 303-2 A1..D8 and stop at the ends, Copy, Paste, Clear, Length and Shuffle;
- a 13-key piano roll over 16 columns that fill the width, with the playing step, the steps beyond the length dimmed, and Transpose centered at the right;
- the Up/Down/Accent/Slide rows under a divider, and Shift at the bottom.
Clicks and drags paint notes, pauses and flags (a drag is one change). Every change is a whole-pattern write through App::write_pattern and Link::set_pattern, the Edit menu's path, so it is heard, saved, shown on the panel and undone by the one-level Undo. Pattern mode keeps the editor and the panel's selection together; song mode keeps them apart and never records. Opening: Edit > 303 Pattern Editor... and Cmd/Ctrl+E (standalone). The window's place is kept in ui.toml (pattern_editor_pos), in the standalone only.

The plugin: the plugin now has the standalone's whole Edit menu (Undo, Cut/Copy/Paste/Duplicate Pattern, 303 Pattern Editor...) without shortcut texts. Its Paste Pattern pastes the app's clipboard without the marker check (egui-baseview reads the clipboard only on a paste key); the user accepted this.

Docs: docs/ui_spec.md section 23 (new), and sections 18 and 20.

Branch and commit: worktree-agent-a7175cbca922f3106, one commit on main 5a6ec92.

Tests: cargo test 1021 passed, 0 failed. New tests: 14 in crates/orb-ui/tests/pattern_editor.rs, 8 unit tests in pattern_editor.rs, and 3 in crates/orb-plugin/tests/editor.rs; three older plugin menu tests now find their menus along the bar. cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check are clean.

User review: the dev app (target/OpenRB-338.app) and the VST3/CLAP bundles, over three rounds:
- round 1: the layout;
- round 2: Undo in the plugin, the 64-pattern sequence of the number arrows, larger labels;
- round 3: the ellipsis on the item, the plugin's full Edit menu;
then the sign-off.
<!-- SECTION:FINAL_SUMMARY:END -->
