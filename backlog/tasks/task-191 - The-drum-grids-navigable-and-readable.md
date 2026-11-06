---
id: TASK-191
title: The drum grids, navigable and readable
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T21:55:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-188
  - TASK-189
  - TASK-209
references:
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/behavior.rs
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 214000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-7; the analysis is docs/accessibility.md. Read both before starting.

- What: in each drum section, a Grid node with 12 rows (the instruments, AC first, named as
  the panel's legend: "Accent", "Bass drum", ...; the 808's switchable slots by the voice they
  play now) and 16 columns (the steps, up to the pattern's length). Up and Down choose the
  instrument (`select_instrument`, so the panel's selection follows), Left and Right the step,
  Space sets the step as a click does (D2; 808: on/off; 909: the click cycle, with Ctrl, Alt and
  Ctrl+Alt as the mouse's modifiers, and the flam button's effect). Each cell reads "Bass drum,
  step 5, on" ("accent", "flam" on the 909). F2 on a drum section reads each instrument that
  has steps, with its steps ("Bass drum: 1, 5, 9, 13. Closed hi-hat: 3, 7, 11, 15").
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: setting steps through the grid gives the same events as clicks on the same steps, on both machines and with each 909 modifier.
- [x] #2 Tests: F2 reads a known pattern correctly; steps past the pattern length are not offered.
- [x] #3 The selection frame on the panel follows the grid's row (visible for sighted helpers).
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. New module crates/orb-ui/src/a11y/grid.rs: the grid's ids (one Grid node per drum machine, a cell node per instrument and step), the instrument names (the panel's legend; the 808's slots 3..7 by the voice their switch picks now, `names::switch_words`), the cell text ("Bass drum, step 5, on"; 909: "accent", "flam", "accent and flam"), the keys (Up/Down: `Panel::select_instrument`; Left/Right: the step, clamped to the pattern's length; Home/End: first/last step; Space: 808 `set_step808(!current)`, 909 `step909_new_value(step, Mods{ctrl, alt})` + `set_step909`, exactly the click's calls) and F2's drum part (`drum_readout`).
2. App (screen reader mode only): in each drum section's group a child Ui with role Grid, focusable (UiBuilder::sense(FOCUSABLE), explicit id = the nav stop), row_count 12 / column_count = pattern length, active_descendant = the cell of (selected instrument, cursor step); the cells are hover-only widgets in that Ui (so their parent is the grid), 12 x length, with bounds on the step's button. The selection frame on the panel follows because the row IS the selected instrument (AC3). With the mode off nothing is built and no key changes.
3. nav::order: the grid's stop at the place of the machine's step buttons in layout.tsv (after the knobs, before the mixer channel). keyboard(): a grid dispatch (`App::grid_key`) before the value keys and the key map, only when the focus is a grid; the 303 sections untouched.
4. announce.rs: `focused_row` knows the grid ids; F2 appends `grid::drum_readout` for rows 2 and 3 through a separate function (`read_drum_pattern`), minimal change.
5. Help > Keyboard Shortcuts rows (Left/Right, Space, Up/Down, Home/End), docs/ui_spec.md 20.4 (new), docs/accessibility.md 3.3/3.4 done-notes, plan NVDA checklist item for what is by ear (TASK-197).
6. Tests: crates/orb-ui/tests/a11y_grid.rs: grid vs clicks events on both machines and each 909 modifier (none, Ctrl, Alt, Ctrl+Alt, flam button), a hi-hat clearing the other; F2 on a known pattern; steps past the length not offered (cells, Right clamp, F2); selection frame follows (inst808/inst909 and the painted frame's place); mode off unchanged. Update a11y_baseline (SR tree), nav unit test, tools/uia_tree.ps1 numbers if they name counts.
7. Alt+Space: winit 0.30.13 sends WM_SYSKEYDOWN to DefWindowProc (event_loop.rs:1767), so Windows opens the system menu on Alt+Space: a Decide task for the user, not settled here.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ae78deeae4d42fe78 (worktree .claude/worktrees/agent-ae78deeae4d42fe78), from main b8ee68a.

Research (2026-11-06):
- egui 0.37: a child Ui can take the focus (UiBuilder::sense(Sense::FOCUSABLE); create_widget registers interest in focus, context.rs:1517..1565) and gets a fixed id with UiBuilder::scope_id (IdSource::Explicit, ui_builder.rs:72). A widget's node parent is the Ui it was made in (Ui::interact_opt -> register_accesskit_parent, ui.rs:956); accesskit_node_builder on an id with no parent entry hangs the node on the root (context.rs:651..689), so the cells are hover-only widgets in the grid's Ui.
- accesskit_consumer 0.39.2 Tree::focus follows the focused node's active_descendant (tree.rs:538..541), so the cell under the cursor is what UI Automation reports as focused; moving the cursor moves that focus (focus_moved).
- accesskit_windows 0.35.2 maps Grid to DataGrid and GridCell/Row to DataItem (node.rs:64..146) but implements no Grid/Table/GridItem pattern; a Row node would be a named DataItem spoken with every cell, so the cells hang directly under the grid and carry the instrument in their text. Name changes are queued as property changes (node.rs:743, UIA_NamePropertyId in the property table 1310), so a Space that changes the focused cell's name raises NameChanged; whether NVDA speaks it is for TASK-197 (plan checklist item 9).
- Alt+Space: winit 0.30.13 passes WM_SYSKEYDOWN to DefWindowProc (src/platform_impl/windows/event_loop.rs:1767), so Windows opens the system menu on Alt+Space after the app got the key. Not settled here: TASK-208 (Decide, @dengopaiv, m-7), recommendation Shift+Space as an extra accent key.
- Alt alone and the menu bar (menu.rs:1390 menu_bar_key): a key press while Alt is down marks it used, so Alt+Space does not open the menu bar.
Implementation: crates/orb-ui/src/a11y/grid.rs (ids, names, cell texts, GridKey, key(), set(), drum_readout, describe_grid/describe_cell), App::drum_grid (in the 808/909 section groups, screen reader mode only), App::grid_key (before the value keys in App::keyboard), App::grid_step (cursor), nav::order (grid stop at layout.tsv's step1 of each machine), announce::read_drum_pattern + focused_row knows the grids. key_help rows: Space, Up/Down, Home/End text, new Left/Right row.
First test run: a11y_baseline SR tree 598 nodes (212 + 2 grids + 384 cells), named 591, focusable 196; a11y_grid 7/8, the one failure was the test's own expectation (it set a slot switch, which is an event); fixed.

Resumed by the supervisor on 2026-11-06 at the user's request ("Initiate and complete Task 191"), in the foreground (no background agent, the user's rule), from the WIP commit e2c1ac2.
Rebase on main 7c09ffa (TASK-193, TASK-190): conflicts resolved keeping both sides in App::keyboard (step_editor_key, then grid_key), announce.rs readout_key (read_drum_pattern for rows 2/3, else the 303 path), key_help.rs (Up/Down and Left/Right rows name both), a11y_baseline (598 nodes, 591 named, 198 focusable), a11y_sections (194 stops, the step editors' and the grids' names), accessibility.md, ui_spec.md (section 10 rows; the drum grids are section 20.6 now, after TASK-193's 20.4 and TASK-190's 20.5; every reference moved). TASK-197's file taken from main and the grid's by-ear check re-added as its AC #8 with the CLI; TASK-208's description now says 20.6.
Decision TASK-209 applied to the grids ("move-along doesn't speak"; a setting): the problem was that playback moving a drum row to another pattern refilled the strip (Link::fill_views) and changed the focused grid's active cell name, which UIA raises as NameChanged. Now:
- model.rs: Panel::drum_pin [Option<u8>; 2] (808, 909), Panel::pin / pin_mut for rows 0..3, Panel::edit_pattern(r) for every row (the 303s' as before); press_pattern moves any pin; Event::Step808 / Step909 carry `pattern` (edit_pattern of the row; a tap's is the row's); tap_drum with the strip on another pattern sends the tap to the playing pattern (and the 909's other hi-hat cleared) without changing the strip.
- link.rs: drum_step writes the event's pattern (it used row_pattern; with the mode off the two are the same pattern: the panel's row pattern and row_pattern are set together by load, Event::Pattern and follow_values); fill_views shows edit_pattern for rows 2/3 and keeps row_pattern = the row's pattern; Link::show_patterns (pub) fills at once.
- app.rs: App::pin_303 renamed pin_patterns, keeps all four pins with step303::pin_of (now any row, Panel::pin); when a drum pin changed it calls Link::show_patterns, so the frame's input (Space) reads the steps of the pattern it edits, not the strip of the frame before (found while designing: fill_views runs only at the end of Link::sync).
- grid.rs: length is the edit pattern's; announce::read_drum_pattern says "Grid on pattern A1: ..." when the grid is not on the row's pattern.
- The setting stays Options > Step Editor Follows Playback (Settings::step_editor_follows), now documented as covering the grids.
Test: a11y_grid::playback_moving_the_pattern_is_silent_on_a_grid on a real Link and engine (the new song, the built-in kit, link.log): A2 gets a bass drum on step 3 directly, the grid sets step 1 of A1, the engine is moved to A2 by an Event::Pattern pushed past the panel (as song mode moves it); with the setting off and on: rows[2].pattern 1, pin still A1, active cell unchanged, strip shows A1, announcer serial unchanged; F2 (input) then: on "TR-808, pattern A2, 16 steps, shuffle off. Bass drum: 3" and the cell of A2; off "... Grid on pattern A1: Bass drum: 1"; Space sends [pattern, step, inst, value] = [1,0,1,1] on / [0,0,1,0] off; off: a tap goes to A2 ([1,5,2,1]) with the strip unchanged, and choosing A2 on the panel moves the pin (cell "Bass drum, step 3, on"). with_the_mode_off_there_is_no_grid also checks drum_pin stays None and edit_pattern is the row's.
Runs (shared build folder CARGO_TARGET_DIR=C:/GIT/music/accessible-openrb-338/target): cargo +nightly-2026-10-18 fmt --all --check clean; cargo clippy --release --workspace --all-targets -- -D warnings clean; cargo test --release -p orb-ui --no-fail-fast: 401 passed, 1 failed (crash.rs a_deliberate_panic_exits_101_and_writes_the_log, the known TASK-200 failure, also on main); a11y_grid 9/9.

Review round 1 (agent-reviewer, 2026-11-06): verdict merge. Checked: a11y_grid 9/9; the mode-off claim (the event's pattern equals what row_pattern was: press_pattern, follow_values, load, load_new and Panel::new set both together; no pin with the mode off, so tap_drum takes the original branch); grid vs mouse calls match App::drum_strips; length clamped 1..16. Non-blocking notes: (1) a pattern chosen on the panel and a grid Space in the same egui frame: the Space reads the previous strip, re-synced at the frame's end; the same holds for the mouse upstream (a pattern button and a step click in one frame), so not changed; the pin_patterns comment claimed too much and now says so (amended). (2) every cell of a step has the step button's bounds (the rows have no place of their own on the panel); harmless for the focus, left as it is.

Merged by the supervisor on 2026-11-06 after review round 1 (merge): fast-forward of main to the branch's one commit, this note amended in. Tests before the merge: cargo test --release -p orb-ui --no-fail-fast 401 pass, 1 known failure (crash.rs, TASK-200); after the review's comment fix, fmt --check and clippy -D warnings clean. Pushed to origin (the fork).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The drum grids of screen reader mode (A11Y-7, docs/ui_spec.md 20.6), done.
- Each drum section has one focusable Grid node, "Steps" (12 instrument rows, AC first, the 808's slots named by their switch; as many columns as the pattern has steps), whose active descendant is the cell under the cursor, so NVDA speaks "Bass drum, step 5, on" ("accent", "flam", "accent and flam" on the 909) as the focus. Up/Down choose the instrument through Panel::select_instrument (the panel's selection frame follows, AC #3), Left/Right/Home/End the step within the pattern's length, Space sets the step with the click's own calls (808 on/off; 909 the click cycle, Ctrl/Alt/Ctrl+Alt as the mouse's modifiers, the flam button's effect). F2 reads each instrument with its steps. Help > Keyboard Shortcuts and ui_spec section 10 list the keys.
- Decision TASK-209 applied: a grid edits its own pattern (Panel::drum_pin, kept by App::pin_patterns with step303::pin_of); playback moving the row never changes the grid or speaks; Options > Step Editor Follows Playback (off by default) makes it move to the playing pattern at the next input, unannounced; F2 says "Grid on pattern A1" when it is not on the playing one. The drum step events carry their pattern; with the mode off it is the row's, as before.
- Tests: crates/orb-ui/tests/a11y_grid.rs, 9 tests: grid vs mouse events on both machines and each 909 modifier and the flam button (AC #1); F2 on known patterns and nothing past the pattern length (AC #2); the selection frame (AC #3); the playback rule on a real link and engine; mode off unchanged. Unit tests in a11y/grid.rs.
- Runs: fmt check clean, clippy -D warnings clean, cargo test --release -p orb-ui --no-fail-fast 401 passed, 1 failed (crash.rs, the known TASK-200 failure).
- Left to the user: TASK-208 (Alt+Space also opens the system menu; Decide), the by-ear check in TASK-197 AC #8 (checklist item 9).
Branch worktree-agent-ae78deeae4d42fe78, one commit on main 7c09ffa.
<!-- SECTION:FINAL_SUMMARY:END -->
