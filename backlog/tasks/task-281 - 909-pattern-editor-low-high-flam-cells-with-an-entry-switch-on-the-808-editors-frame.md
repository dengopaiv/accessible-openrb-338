---
id: TASK-281
title: >-
  909 pattern editor: low/high/flam cells with an entry switch, on the 808
  editor's frame
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-22T09:49:00Z'
updated_date: '2026-11-22T16:36:00Z'
labels:
  - ui
milestone: m-9
dependencies:
  - TASK-280
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/controller.rs
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
  - docs/RBS42.txt
  - docs/initial_investigation.md
priority: medium
type: feature
ordinal: 260200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** milestone m-9's third editor, the TR-909's. It is built on the drum-grid frame of the 808 editor (task-280). The user designed it on 2026-11-22.

**The data** (docs/RBS42.txt, the "909 " chunk; docs/ui_spec.md, the 909 step buttons):
- A pattern is a shuffle byte, a length 1..16, and 16 steps × 12 bytes: AC, BD, SD, LT, MT, HT, RS, CP, CH, OH, CC, RC.
- An instrument byte: 0 off, 1 low, 2 high, 3 flam (a low hit plus the flam retrigger).
- The AC row is on/off, and the stored "on" is 2, as the panel writes it.
- A byte above 3 (a song may hold any): the editor draws it as flam, as the original and the panel do, and keeps the byte unless the user edits that cell. The editor never writes values above 3.
- Colors as on the panel: low orange, high red, flam green.

**Layout:** the 808 editor's frame.
- 11 drum rows (BD SD LT MT HT RS CP CH OH CC RC), then a separator and the AC row (tooltip "Accent: every drum on this step louder").
- Bank, pattern number with arrows, Length, Shuffle, Shift, Copy/Paste/Clear, and the one-level Undo, all as in the 808 editor.
- Its own window, separate from the 808's (the user's decision).

**The entry switch: Cycle | Low | High | Flam**, in the header. Cycle is the default.
- **Cycle:** a click on a drum cell goes off → low → high → off, like the panel's plain click.
- **Low / High / Flam:** a click sets that value. Clicking a cell that already holds that value clears it, and a cell holding another value is replaced in one click. This is the original panel's modifier rule.
- **Modifiers** switch to Low / High / Flam while held, and the switch returns to its own setting on release; the switch shows the held state:
  - macOS: Cmd = Low, Option = High, Cmd+Option = Flam;
  - Windows and Linux: Ctrl = Low, Alt = High, Ctrl+Alt = Flam.
  These are consistent with the main panel, which this task changes the same way (below).
- **AC row:** a click toggles it (0 ↔ 2), whatever the switch says.
- **Drag:** paints the first cell's new value along the same row only.
- **CH/OH:** entering a value on CH clears OH on that step, and vice versa, as on the original panel (docs/ui_spec.md: `sub_447153(9 or 8, step, 0)`). A song that already holds both is shown as is.
- Every edit is one whole-pattern write through `App::write_pattern` and undoable.

**The main panel's 909 modifiers (the user's decision: consistent across the app):**
- Today the panel reads the physical Ctrl and Alt on every platform (crates/orb-ui/src/controller.rs around line 380, `Mods { ctrl, alt }` → `model::step909_new_value` → `behavior::tr909_step_value`).
- Change macOS to Cmd for Ctrl and Option for Alt. Windows and Linux keep Ctrl and Alt.
- Update the panel's docs (docs/ui_spec.md, the 909 step buttons) and its tests.

**Opening:**
- Edit menu "909 Pattern Editor...".
- Cmd/Ctrl+E with the 909 row focused opens it, as task-280 set up for the other rows.
- Works in the plugin too.

**Keyboard shortcuts help (the user's requirement):** Help > Keyboard Shortcuts must list every key and modifier the 909 editor supports, including the entry-switch modifiers with their per-platform names, and the panel's changed 909 modifiers. Its tests cover them.

**Docs:** docs/ui_spec.md, the editor section, the 909 step buttons and the Edit menu.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Edit > 909 Pattern Editor... and Cmd/Ctrl+E with the 909 row focused open a separate floating 909 editor (standalone and plugin) on task-280's drum-grid frame: 11 drum rows, the AC row, bank, pattern number, Length, Shuffle, Shift, Copy/Paste/Clear, Undo
- [x] #2 The entry switch Cycle | Low | High | Flam (default Cycle): Cycle cycles off-low-high-off; Low/High/Flam set the value, clear a cell holding the same value and replace another value in one click; modifiers (Cmd/Option on macOS, Ctrl/Alt elsewhere; both for Flam) select the switch while held and the switch shows it
- [x] #3 Cells draw low orange, high red, flam green, values above 3 as flam and kept unless edited; the AC row toggles 0/2; drag paints along the row; entering CH clears OH on that step and vice versa; every edit is one undoable pattern write
- [x] #4 The main panel's 909 step modifiers use Cmd/Option on macOS (Ctrl/Alt elsewhere), with docs and tests updated
- [x] #5 Help > Keyboard Shortcuts lists every key and modifier of the 909 editor and the panel's 909 modifiers with per-platform names, and its tests cover them; docs/ui_spec.md updated; cargo test --workspace passes, clippy -D warnings and nightly fmt clean
- [x] #6 The user reviewed the dev app and the plugin before the agent review
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. drum_editor.rs: Machine::Tr909 (row 3, labels BD SD LT MT HT RS CP CH OH CC RC = step bytes 1..=11, its title, menu item, salt, settings key pattern_editor_909_pos); an Entry enum Cycle/Low/High/Flam kept per editor, the held modifiers (egui command = Cmd on macOS / Ctrl elsewhere -> Low, alt -> High, both -> Flam) override it while held; the 909's click value reuses behavior::tr909_step_value (the panel's rule) on the shown value min(v, 3); AC toggles 0/2; a write of a non-zero CH (byte 8) or OH (byte 9) clears the other on that step (Machine::put, used by click and drag); colors from the modern panel art step909_1..3 (#ff9a30, #ff4a36, #5cff4a), values above 3 drawn as flam and kept until edited.
2. The header gets the entry switch (Cycle | Low | High | Flam) for the 909 only; the window and every edit go through the 808's code (one write_pattern each, a drag one undo).
3. Input: per-machine lists (Machine::inputs), 909-only inputs (the switch, Low/High/Flam held), keys/what by machine and platform.
4. Panel: controller reads modifiers.command (Cmd on macOS, Ctrl elsewhere) and alt for the 909 steps through one helper shared with the editor; ui_spec section 7 and the behavior tests updated.
5. App: Cmd/Ctrl+E with the 909 focused opens the 909 editor (replaces the task-280 stopgap); the Edit menu's 909 item, the shortcut text on the focused row's item; settings key.
6. Help > Keyboard Shortcuts: a 909 Pattern Editor group and a group for the panel's 909 step modifiers, with per-platform names; tests.
7. Tests: tests/drum_editor.rs harness generic over the machine, 909 tests (menu, Cmd+E, switch, modifiers, colors, >3 kept, AC, drag, CH/OH, undo, every help input); plugin test for the 909 menu item.
8. Docs: ui_spec.md section 25 (the 909 editor), section 7 (panel modifiers), section 18 (Edit menu), section 17 (settings key).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-adb2423992773026f (worktree .claude/worktrees/agent-adb2423992773026f), on main abea360.

Implementation (crates/orb-ui):
- drum_editor.rs: Machine::Tr909 (row 3, index 1, "909 Pattern Editor", salt orb_drum_editor_909, settings key pattern_editor_909_pos). Its labels are BD SD LT MT HT RS CP CH OH CC RC (step bytes 1..=11). CH is byte 8 and OH byte 9, as the panel's inst909 8/9 and sub_447153(9 or 8, step, 0) (docs/ui_spec.md section 7).
- The Entry enum: Cycle, Low, High, Flam. DrumEditor.entry is the switch and held is Entry::held of the modifiers, read each frame; entry_now() is held, else the switch.
- Machine::clicked(column, v, entry): the 909 reuses behavior::tr909_step_value (the panel's rule) on v.min(3), with the entry as its Mods; AC goes 0<->2.
- Machine::put: a non-zero CH/OH write clears the other on that step. The click and every painted step of a drag go through it.
- Colors LOW #ff9a30, HIGH #ff4a36, FLAM #5cff4a, from assets/ui/step909_1..3.svg; color() draws v >= 3 as flam.
- The switch is a second header row ("Entry  Cycle Low High Flam"): the labels are colored, the tooltips name the held key per platform. DrumAct::Entry; App::drum_editor_entry_shown for the tests.
- behavior::step_mods(egui::Modifiers) -> Mods { ctrl: command, alt }: one reading for the panel and the editor. The panel's controller.rs drum_strips read the physical ctrl before; Entry::held uses it too.
- App::open_focused_editor uses Machine::of_row: focus 3 opens the 909 editor. This replaces task-280's stopgap, which opened the 303's. The Edit menu puts the Ctrl/Cmd+E text on the focused row's item (303, 808 or 909).
- Input is per machine (Input::ALL_808, ALL_909, Machine::inputs). The 909-only inputs are EntrySwitch, HoldLow, HoldHigh and HoldFlam; keys(m, is_mac) and what(m).
- key_help.rs:
  - Row.editor is (Machine, Input).
  - The groups "808 Pattern Editor" and "909 Pattern Editor" come from editor_group.
  - A new group "The panel's TR-909 step buttons" (key_help::Step909: Click, Cmd/Ctrl+click, Option/Alt+click, Cmd+Option / Ctrl+Alt+click, AC, Drag) has per-platform names.
  - The menu row of Ctrl/Cmd+E names the 909 editor.
- settings.rs: pattern_editor_909_pos, as the 808's.

Design choices the task does not settle (for the user's review):
1. The entry switch is on its own second header row, not in the top row: the top row is already near the smallest panel's 624 points.
2. A byte above 3 counts as flam for a click (the value shown, v.min(3)): Flam clears it, Cycle takes it to low, Low/High replace it. The panel applies the original's rule to the raw byte instead (Cycle on 4 gives 2, on 5 gives 0; Flam on 4 gives 3). The task says the editor draws it as flam; I made the click agree with what is drawn.
3. The entry switch's setting is per editor and not saved (Cycle at each start).
4. The editor does not read the panel's Flam button (the switch's Flam is its own).
5. The AC row's "on" (2) is drawn red, by its byte, as the panel's LED frame of 2.
6. The help gets a separate group for the panel's 909 step buttons (no group listed panel mouse actions before).

Tests:
- crates/orb-ui/tests/drum_editor.rs: the harness takes the machine (Harness::on). New 909 tests: the_909_menu_item_and_cmd_e_open_its_own_window, the_909_header_rows_and_entry_switch, the_909_switch_and_the_held_keys_enter_their_values, a_909_byte_above_3_shows_as_flam_and_is_kept_until_its_cell_is_edited, a_909_drag_paints_its_value_along_the_row_and_clears_the_other_hat, the_909_edits_are_heard_and_shown_on_the_panel, the_909_editors_place_is_its_own_in_the_settings_file. every_input_of_the_help_does_what_it_says runs both machines' inputs through one exhaustive match.
- Unit tests: drum_editor.rs (the 909 click rule, the hats, the held modifiers, the labels); behavior.rs the_909_step_modifiers_are_cmd_and_option_on_macos; key_help.rs each_drum_editor_group_lists_every_input_of_its_editor and the_panels_909_step_group_names_the_keys_that_do_it.
- Panel tests moved to the command key: tests/interaction.rs tr909_step_click_cycles_and_modifiers_pick_the_value (plus macOS's Ctrl = no modifier) and tests/wired.rs the_909_step_buttons_send_the_values_of_the_modifiers_and_clear_the_other_hihat.
- Plugin: crates/orb-plugin/tests/editor.rs the_edit_menu_opens_the_909_pattern_editor_and_undoes_its_changes, and the 909 item in the Edit menu's item list.

Results:
- cargo test --workspace --no-fail-fast: 1064 passed, 0 failed, 5 ignored.
- cargo clippy --all-targets -- -D warnings: clean. cargo +nightly fmt --all --check: clean.
- Builds: target/OpenRB-338.app (cargo build --release -p orb-ui, then scripts/macos-dev-app.sh) and target/bundled/OpenRB-338.vst3 (cargo xtask bundle orb-plugin --release).
- No engine change (UI only), so no corpus run.

AC evidence:
- AC 1: tests/drum_editor.rs the_909_menu_item_and_cmd_e_open_its_own_window (menu order 303, 808, 909; its own window beside the 808's; Cmd/Ctrl+E with the TR-909 focused opens it, not the 303's) and the_909_header_rows_and_entry_switch (11 rows plus AC, the bank, number, Length, Shuffle, Shift, Copy/Paste/Clear hits). Undo: the_909_switch_and_the_held_keys_enter_their_values. Plugin: crates/orb-plugin/tests/editor.rs the_edit_menu_opens_the_909_pattern_editor_and_undoes_its_changes. The plugin takes no keys (task-201, app.rs), so there the Edit menu item is the way in, as for the 303 and 808 editors; Cmd/Ctrl+E is the standalone app's.
- AC 2: the_909_switch_and_the_held_keys_enter_their_values, every_input_of_the_help_does_what_it_says (EntrySwitch, HoldLow/High/Flam: shown while held, back to the switch's setting on release), drum_editor.rs unit tests a_click_on_the_909_follows_the_entry and the_modifiers_select_the_entry_while_held (Cmd via mac_cmd+command, Ctrl via ctrl+command, Alt; macOS's Ctrl alone selects nothing).
- AC 3: a_click_on_the_909_follows_the_entry (colors, above 3 as flam, AC 0/2), a_909_byte_above_3_shows_as_flam_and_is_kept_until_its_cell_is_edited, a_909_drag_paints_its_value_along_the_row_and_clears_the_other_hat (one Undo per drag), a_909_hi_hat_entered_clears_the_other_on_its_step, the_909_edits_are_heard_and_shown_on_the_panel.
- AC 4: behavior::step_mods used by controller.rs drum_strips; behavior.rs the_909_step_modifiers_are_cmd_and_option_on_macos; tests/interaction.rs and tests/wired.rs moved to the command key; docs/ui_spec.md section 7 'orb's keys'.
- AC 5: key_help.rs each_drum_editor_group_lists_every_input_of_its_editor and the_panels_909_step_group_names_the_keys_that_do_it; docs/ui_spec.md sections 7, 10, 18, 20, 23, 24 and the new 25; cargo test --workspace 1064 passed, 0 failed; clippy and nightly fmt clean.

Correction to the AC 5 evidence: the Ctrl/Cmd+E paragraph updated is in docs/ui_spec.md section 16, not section 10.

Review round 1 (the user's review of the build, 2026-11-22, through the coordinator):
- The 'Entry' label in front of the switch is removed: only the Cycle | Low | High | Flam segments remain.
- The tooltips are the user's text, in drum_editor::entry_tip (now pub). The key names follow the platform through Input::Hold*.keys:
  - Cycle: 'Click sets cell off -> low -> high -> off'
  - Low: 'Click sets cell to low. Hold Cmd to activate' (Ctrl)
  - High: 'Click sets cell to high. Hold Option to activate' (Alt)
  - Flam: 'Click sets cell to flam. Hold Cmd+Option to activate' (Ctrl+Alt)
- A new unit test, the_entry_switchs_tooltips_are_the_users_text_with_the_platforms_keys, checks both platforms. No other test checked the label or the tooltips.
- docs/ui_spec.md section 25 is updated.

Review round 1 checks: cargo test --workspace --no-fail-fast 1065 passed, 0 failed, 5 ignored (one more than before: the new tooltip test). Clippy -D warnings and nightly fmt --check are clean. Rebuilt target/OpenRB-338.app and target/bundled/OpenRB-338.vst3.

The user signed off the 909 editor on 2026-11-22 ('looks good', through the coordinator), on the build of commit 25af3fa: target/OpenRB-338.app and target/bundled/OpenRB-338.vst3. This sign-off covers the design choices as built (the second header row, a byte above 3 counting as flam for a click, the switch not saved, the panel's Flam button not read, AC drawn red, the separate help group for the panel's 909 step buttons).

Final checks after the sign-off: cargo test --workspace --no-fail-fast 1065 passed, 0 failed, 5 ignored. Clippy -D warnings and nightly fmt --check are clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The 909 pattern editor is done. Edit > 909 Pattern Editor... and Ctrl/Cmd+E with the TR-909 focused open it in the standalone app; in the plugin, which takes no keys, the menu item opens it. It is a separate floating window on task-280's drum-grid frame: 11 drum rows (BD..RC), the AC row, the bank, the pattern number, Length, Shuffle, Shift, Copy/Paste/Clear and Undo.

What the editor does:
- The entry switch Cycle | Low | High | Flam starts on Cycle and has no label in front.
  - Cmd/Option (Ctrl/Alt elsewhere) and both select Low, High and Flam while held, and the switch shows it.
  - Its tooltips are the user's text, with the key names of each platform.
- The values follow the panel's rule (behavior::tr909_step_value).
- The cells are low orange, high red and flam green. A byte above 3 is drawn as flam and kept until its cell is edited.
- The AC row goes 0/2.
- A drag paints along its row.
- Entering CH clears OH on that step and the other way round.
- Every edit is one write_pattern and one Undo.

Other changes:
- The panel's 909 step modifiers are Cmd/Option on macOS (behavior::step_mods, shared with the editor).
- Help > Keyboard Shortcuts has the "909 Pattern Editor" group and a "The panel's TR-909 step buttons" group, with the key names of each platform.
- The window's place is saved as pattern_editor_909_pos.
- docs/ui_spec.md has the new section 25, and sections 7, 16, 18, 20, 23 and 24 are updated.

The user's decisions:
- Review round 1: the label was removed and the tooltips were set to the user's text.
- The user signed off on 2026-11-22. The sign-off covers the design choices as built:
  - the switch on a second header row;
  - a byte above 3 counts as flam for a click;
  - the switch's setting is not saved;
  - the panel's Flam button is not read;
  - the AC "on" is drawn red;
  - the panel's 909 step buttons get their own help group.

Branch: worktree-agent-adb2423992773026f, one commit (amended).

Tests:
- cargo test --workspace: 1065 passed, 0 failed, 5 ignored.
- clippy -D warnings and nightly fmt are clean.
- This is a UI-only change, so there was no corpus run.
<!-- SECTION:FINAL_SUMMARY:END -->
