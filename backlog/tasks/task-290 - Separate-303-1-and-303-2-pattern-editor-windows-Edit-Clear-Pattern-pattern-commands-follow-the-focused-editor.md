---
id: TASK-290
title: >-
  Separate 303-1 and 303-2 pattern editor windows; Edit > Clear Pattern; pattern
  commands follow the focused editor
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-24T12:27:00Z'
updated_date: '2026-11-24T16:00:00Z'
labels:
  - ui
  - player
milestone: m-9
dependencies: []
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/src/editor_frame.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/shortcuts.rs
  - crates/orb-plugin/src/clip.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 265700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-24 for three changes in one task.

**1. Two 303 editor windows.** Today one window, "303 Pattern Editor" (crates/orb-ui/src/pattern_editor.rs around 624, task-274), edits either 303 and switches between them with its own 303-1 / 303-2 buttons (pattern_editor.rs around 377).
- Make it two independent windows, titled "303-1 Pattern Editor" and "303-2 Pattern Editor", each fixed to its 303. Either or both can be open, as the 808 and 909 editors are (crates/orb-ui/src/drum_editor.rs, editor_frame.rs).
- The in-window 303 selector goes.
- The Edit menu gets "303-1 Pattern Editor..." and "303-2 Pattern Editor..." in place of "303 Pattern Editor...".
- Cmd/Ctrl+E opens the focused row's editor, as for the 808 and 909.
- Each window keeps its own position in ui.toml (today pattern_editor_pos; e.g. pattern_editor_303_1_pos and _2_pos). The old key seeds 303-1's position once, so a saved position is not lost.
- Bringing an already open window to the front (task-283) works for both.
- The plugin gets the same windows and menu items; it takes no keys (task-201).

**2. Edit > Clear Pattern**, with no keyboard shortcut. It empties the target pattern's steps the way the editors' Clear button does (emptied_pattern: the shuffle and length bytes stay), as one write_pattern with Undo.

**3. Which pattern the Edit menu's pattern commands act on.** Copy, Cut, Paste, Duplicate and Clear Pattern act on:
- the pattern selected in the pattern editor window that has the focus, when one does: the editor's own selected pattern and its device;
- otherwise, the selected pattern of the focused row in the main panel, as today (`App::pattern_target`, crates/orb-ui/src/menu.rs around 1054).
Define "has the focus" in the notes: egui's focused or topmost window, a click into it, or similar, whatever is reliable in both the app and the plugin. Paste Pattern follows the same rule (the user, 2026-11-24): it pastes into the focused editor's selected pattern, and the device check is against that editor's device.

**4. Survey the names of the two 303s** in every user-visible place: window titles, menus, status messages, tooltips, the Keyboard Shortcuts help, the plugin's MIDI clip sections and drag-out file names, the docs (docs/ui_spec.md, docs/plugin.md, the site and README), and the CLI. The supervisor's quick grep on 2026-11-24 found three forms:
- "303 1" / "303 2": the status line row names (menu.rs ROW_NAMES);
- "303-1" / "303-2": the 303 editor's selector buttons and the plugin's drag-out clip file names, e.g. "303-1 A1.mid" (crates/orb-plugin/src/clip.rs);
- "303 #1" / "303 #2": the plugin's MIDI clip sections (clip.rs SECTIONS) and the Keyboard Shortcuts help's Select Patterns rows (key_help.rs around 497).
List every occurrence with file:line in the notes, as a table by form. Do not unify them in this task. The new window titles use "303-1" / "303-2" as the user named them. Report the table to the user, who decides on one form; that becomes a follow-up task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 "303-1 Pattern Editor" and "303-2 Pattern Editor" are separate windows, each fixed to its 303, both can be open, with Edit menu items, Cmd/Ctrl+E for the focused row, positions kept per window, in the app and the plugin
- [x] #2 Edit > Clear Pattern (no shortcut) empties the target pattern's steps with Undo, keeping shuffle and length
- [x] #3 The notes hold a table of every user-visible 303-1 / 303 1 / 303 #1 form (and any other) with file:line, and the report gives it to the user; nothing is unified yet
- [x] #4 docs/ui_spec.md describes the two windows, Clear Pattern and the target rule; the Keyboard Shortcuts help stays consistent with the user's cuts (task-284)
- [x] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [x] #6 The user has checked it in the app and the plugin
- [x] #7 Copy, Cut, Paste, Duplicate and Clear Pattern act on the focused pattern editor's selected pattern, else on the main panel's focused row's selected pattern; tests cover both cases for a 303, the 808 and the 909
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. pattern_editor.rs: App.editors [Editor; 2], each fixed to its 303 (synth 0/1); the in-window 303 switch (Act::Synth) goes; Act::Select(pattern); titles '303-1 Pattern Editor' / '303-2 Pattern Editor', ids per 303; follow rule as the drum editors (pattern mode: the 303's selected pattern; song mode: its own); the number arrows run A1..D8 of its own 303 and stop (the shared step_pattern moves to editor_frame). API takes the 303 index (pattern_editor(k), open_pattern_editor(k), editor_act(k, a), ..._hit(k, ..)).
2. settings.rs: pattern_editor_303_1_pos and pattern_editor_303_2_pos with setters; the old pattern_editor_pos is read into 303_1 when 303_1 is absent and is no longer written (dropped at the next save, as theme_viewer_filter).
3. menu.rs: Cmd::PatternEditor(k); Edit menu items '303-1 Pattern Editor...' and '303-2 Pattern Editor...', Cmd/Ctrl+E text on the focused row's item; Cmd::ClearPattern 'Clear Pattern' after Duplicate, no shortcut (EDIT_ITEMS take an optional shortcut); clear_pattern / clear_pattern_at (emptied_pattern, one write_pattern, Undo 'Clear Pattern', status 'cleared'), also used by the editors' Clear button.
4. Focus: App.editor_focus: Option<row>. Set when an editor opens (menu, Cmd/Ctrl+E) and on a primary press inside its window (frame::show_window reports a press whose layer is the window's); cleared when that window closes and on a primary press on the panel itself (focus_click, the panel's layer inside the skin's rect; the menu bar and other windows do not clear it). pattern_target returns the focused open editor's row and its pattern (after its follow rule runs), else the panel's focused row as today; Copy/Cut/Paste (menu and keys)/Duplicate/Clear all go through it; Duplicate from an editor moves that editor to the copy.
5. Tests: update tests/pattern_editor.rs, drum_editor.rs, raise_windows.rs, the plugin's tests/editor.rs; new tests: two 303 windows open at once with their own places and the old key seeding 303-1; Clear Pattern with Undo keeping shuffle and length; the target rule for 303-1/303-2, the 808 and the 909 (editor focused vs panel, click on panel takes it back), Paste device check against the editor's device.
6. docs/ui_spec.md: section 23 and the Edit menu table; settings module text.
7. Survey of the 303 name forms -> notes table (no unification).
8. cargo test --workspace, clippy (macOS and x86_64-pc-windows-msvc), nightly fmt; build the dev app and the VST3; one commit, task still To Do, report to the supervisor.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-abfab1cbfa1e731a9

Work (2026-11-24), rebased on e46e92b.

1. Two 303 windows. crates/orb-ui/src/pattern_editor.rs: App.editors [Editor; 2] (app.rs), each fixed to its 303 (Editor::new(k)); titles "303-1 Pattern Editor" / "303-2 Pattern Editor" (pattern_editor::title), menu items "303-1 Pattern Editor..." / "303-2 Pattern Editor..." (menu_item), window ids egui::Id::unique(salt(k)) = "orb_pattern_editor_303_1" / "_2". The in-window 303-1/303-2 switch (Act::Synth) is gone; Act::Select(pattern). API takes the 303: pattern_editor(k), open_pattern_editor(k), editor_act(k, a), pattern_editor_hit/note_cell/flag_cell/window/shown(k, ..); Cmd::PatternEditor(k).
   - Follow rule now the drum editors' (editor_follow): pattern mode shows its 303's selected pattern; song mode keeps its own. The old rule (a selection of the other 303 moved the one window to that 303, the seen/was_song_mode fields) went with the switch.
   - Consequence the task did not spell out: the number arrows no longer run 303-1 A1..D8 then 303-2 A1..D8 (task-274 round 2); with a window fixed to its 303 they run A1..D8 of that 303 and stop at both ends, as the drum editors'. step_pattern moved to editor_frame (shared). Flagged for the user's check.
   - Cmd/Ctrl+E: open_focused_editor opens open_pattern_editor(focus) for rows 0/1. Menu shortcut text on the focused row's item (303-1, 303-2, 808 or 909).
   - Bring to front (task-283): tests/raise_windows.rs lists both 303 windows; passes.
   - Positions: settings.rs pattern_editor_303_1_pos / pattern_editor_303_2_pos with setters set_pattern_editor_303_1_pos / _2_pos, merged like the others. The old pattern_editor_pos is read into 303_1 when 303_1 is absent and is no longer written, so the next save of any key moves it to the new key and drops the old (as theme_viewer_filter). Test the_old_place_seeds_303_1s_window.
   - Plugin: same windows and items (shared orb-ui App); no keys; positions not saved (unchanged rule).
2. Edit > Clear Pattern (menu.rs Cmd::ClearPattern, EDIT_ITEMS now carry Option<shortcut>, Clear has None): clear_pattern -> pattern_target -> clear_pattern_at: emptied_pattern (shuffle and length bytes kept), one write_pattern "Clear Pattern" with Undo, status "<row>: <pattern> cleared". The editors' Clear buttons (303 and drum) now call clear_pattern_at too (same bytes, same Undo name as before).
3. Target rule (menu.rs App::pattern_target): the focused open editor's row and pattern (App::focused_editor, after its follow rule runs so a selection made since its last frame counts), else panel.focus and its selected pattern. Copy, Cut, Paste (menu item, Ctrl/Cmd+V Paste event, plugin's direct read), Duplicate and Clear all go through it; Paste's device check is ROW_KINDS[row] of that target, so the editor's device. Duplicate from a focused editor also moves that editor to the copy (song mode too).
   **"Has the focus" defined** (App.editor_focus: Option<row>, 0/1 the 303 editors, 2 the 808's, 3 the 909's): the editor opened last (Edit menu item or Ctrl/Cmd+E) or the one a primary mouse press went down on last (anywhere on its window, title bar included; editor_frame::show_window compares egui's layer_id_at(press origin), last frame's layers, with the window's LayerId). It loses the focus when a primary press goes down on the panel itself (controller.rs focus_click: the panel's own layer and inside the skin's rect, so the menu bar, the status line, the plugin's desk margin and windows over the panel do not count) or when its window closes (editor_unfocus). Key-driven selections and pattern buttons pressed by code do not change it. Not used: egui's keyboard focus (the plugin takes no keys) and the window order (a click on the panel does not reorder windows, so the topmost editor would keep the target after the user went back to the panel). Pointer presses are the same in the app and the plugin (baseview delivers them to egui alike). The focused window is not marked visually.
4. Tests: tests/pattern_editor.rs rewritten for two windows (menu items in order, both open at once, Cmd+E per 303, bring to front, no 303 switch, per-window places, the old key seeding 303-1, arrows A1..D8 per 303, selection rules) plus the_edit_menus_pattern_commands_follow_the_focused_editor (for rows 0, 1, 2, 3: Copy, Clear with Undo keeping shuffle and length, Cut, Duplicate, Paste with the device refused and accepted, then a real click on the panel row gives the panel's pattern, a click into the editor window gives the editor's again) and of_two_303_windows_the_last_clicked_is_the_target (Clear Pattern through the menu bar; the menu bar keeps the focus). tests/drum_editor.rs, tests/raise_windows.rs, crates/orb-plugin/tests/editor.rs (both 303 items opened in the plugin, Clear Pattern in the plugin's Edit menu order) updated.
5. Keyboard Shortcuts help (key_help.rs) unchanged: Clear Pattern has no shortcut, the Menu shortcuts group keeps Undo only, no editor groups (task-284's cuts hold; the_users_cuts_and_renames_hold passes).
6. docs/ui_spec.md: sections 4, 16, 17 (the place keys), 18 (Edit table: Clear Pattern, the two 303 items, the target rule and the focus definition), 20 (plugin's Edit menu), 23 (two windows, places, top row without the switch, arrows, follow rule, Clear).

Checks: cargo test --workspace 1114 passed, 0 failed, 5 ignored. cargo clippy --all-targets -D warnings clean (macOS) and cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets -D warnings clean. cargo +nightly fmt --all --check clean. Dev app target/OpenRB-338.app (scripts/macos-dev-app.sh after cargo build --release -p orb-ui), VST3 target/bundled/OpenRB-338.vst3 (cargo xtask bundle orb-plugin --release).

## The 303 name survey (AC 3; nothing unified)

User-visible places only (UI strings, plugin names and files, docs for users, site, README; the CLI has no numbered 303 name). Internal ids such as tb303_1 and test assertions are left out. Lines as of this branch.

**"303-1" / "303-2" (hyphen)**
| Where | file:line |
|---|---|
| new window titles "303-1 Pattern Editor" / "303-2 ..." | crates/orb-ui/src/pattern_editor.rs:307 |
| new Edit menu items "303-1 Pattern Editor..." / "303-2 ..." | crates/orb-ui/src/pattern_editor.rs:312 |
| plugin's VST3 aux output bus names | crates/orb-plugin/src/lib.rs:57 (OUTPUT_NAMES) |
| plugin's drag-out clip file names "303-1 A1.mid" (derived from SECTIONS by replacing " #" with "-") | crates/orb-plugin/src/clip.rs:94 |
| docs: ui_spec.md (new text of this task) | docs/ui_spec.md:1098, 1263, 1313, 1314, 1988, 2359, 2360, 2364, 2373, 2388 |
| docs: plugin.md (clip file names, output buses, per-device outputs) | docs/plugin.md:1700, 1791, 1859, 1862 |
| README (outputs) | README.md:46 |
| site (outputs) | site/index.html:100 |
| removed by this task: the 303 editor's selector buttons | (was crates/orb-ui/src/pattern_editor.rs:663) |

**"303 1" / "303 2" (space)**
| Where | file:line |
|---|---|
| status line row names: Copy/Cut/Paste/Duplicate/Clear/Undo messages ("303 1: A1 copied") | crates/orb-ui/src/menu.rs:143 (ROW_NAMES; used at menu.rs:1171, 1190, 1242, 1261, 1314) |
| docs: ui_spec.md status examples | docs/ui_spec.md:1312, 2444 |

**"ORB-303 1" / "ORB-303 2"** (a fourth form)
| Where | file:line |
|---|---|
| plugin's automation parameter group/name prefix shown in hosts ("ORB-303 1 Cutoff") | crates/orb-plugin/src/params.rs:273, 274 |
| docs: plugin.md (parameter names) | docs/plugin.md:129, 158 |

**"303 #1" / "303 #2"**
| Where | file:line |
|---|---|
| plugin's MIDI clip sections and MIDI track names ("303 #1 A1") | crates/orb-plugin/src/clip.rs:81 (SECTIONS; track name at clip.rs:370) |
| Keyboard Shortcuts help, Select Patterns rows | crates/orb-ui/src/key_help.rs:497, 498 |
| docs: plugin.md (MIDI channels, patterns state, clip sections and tracks) | docs/plugin.md:311, 683, 684, 727, 1565, 1679, 1682 |
| docs: midi.md (MIDI channels) | docs/midi.md:84, 176, 177, 185 |
| docs: standard_midi_mapping.md (RE heading) | docs/standard_midi_mapping.md:68 |
| docs: ui_spec.md (mixer sections) | docs/ui_spec.md:681 |

**"TB-303 #1" / "TB-303 #2"** (docs only; also a TB- name, task-286's area)
| Where | file:line |
|---|---|
| docs: ui_spec.md (panel geometry table, sections, keyboard keys) | docs/ui_spec.md:68, 72, 367, 846 |

Survey script: a regex over git ls-files (crates/*/src string literals, docs/ui_spec.md, plugin.md, midi.md, standard_midi_mapping.md, building.md, README.md, site/*), checked by hand; panel art (assets, orb-ui-art), plugin_usage.rs and orb-cli have no numbered 303 name.

Review round 1 (the user, 2026-11-24):
1. No compatibility code between unreleased builds: pattern_editor_pos was added by task-274 (2026-11-22), after the last release v2026.1118.0919, so no release wrote it. Removed: the seeding of pattern_editor_303_1_pos from pattern_editor_pos (settings.rs Settings::read), its mentions in the settings.rs header and the field doc, the test the_old_place_seeds_303_1s_window (tests/pattern_editor.rs), and the docs/ui_spec.md text in sections 17 and 23 (section 23 now says the old key is not read). An old key in a dev file is ignored. Nothing else in the diff is compatibility code (the 'until task-290' sentences in ui_spec are history, not code).
2. Clear Pattern always sets the status '<row>: <pattern> cleared', also on an already empty pattern (then nothing is written and the Undo stays): menu.rs clear_pattern_at, which the editors' Clear buttons share. Test of_two_303_windows_the_last_clicked_is_the_target clears an empty pattern from the menu and from 303-1's Clear button and checks the status. The status keeps ROW_NAMES's form ('303 1: A2 cleared'); the user's naming choice ('303-1' everywhere, no 'ORB-') is the supervisor's follow-up task, not done here. ui_spec section 18 table and section 23 Clear updated.
3. Arrows A1..D8 per window: kept (the user did not object).
Survey table line corrections after the round: ROW_NAMES is menu.rs:148 (used at 1171, 1190, 1242, 1261, 1316); ui_spec.md '303-1' lines 1098, 1311, 1312, 1986, 2357, 2358, 2362, 2385 (the section 17 line is gone), '303 1' lines 1310, 2442.

Sign-off: the user checked commit 8d30113 in the app and the plugin on 2026-11-24 ('looks good'), covering the per-window arrows (A1..D8 of its own 303), the focus definition and Clear's always-shown status. Follow-up for the name survey: task-293 (303-1 / 303-2 everywhere, no ORB- prefix). Rebased onto main e1e3838 (backlog commits only since the base).

AC 1's words 'the old key seeds 303-1' were superseded by the user's review round 1: pattern_editor_pos was never in a release, so it is not read; positions are kept per window. The other criteria: 2 and 7 are tested in tests/pattern_editor.rs (the_edit_menus_pattern_commands_follow_the_focused_editor, of_two_303_windows_the_last_clicked_is_the_target); 3 is the survey table above, reported to the user; 4 is the ui_spec edits plus key_help being unchanged (the_users_cuts_and_renames_hold passes); 5 is the checks below; 6 is the sign-off above.

Criterion 1 reworded after the user's review round 1 (the 'old key seeds 303-1' clause dropped: no release wrote pattern_editor_pos); still checked.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The single 303 Pattern Editor is now two windows, '303-1 Pattern Editor' and '303-2 Pattern Editor'. Each is fixed to its 303, and either or both can be open in the app and the plugin. The window no longer has its own 303 switch. The number arrows run A1..D8 of the window's own 303 and stop at both ends. Each window follows its 303 row the way the 808 and 909 editors do. The Edit menu has '303-1 Pattern Editor...' and '303-2 Pattern Editor...', and Cmd/Ctrl+E opens the editor of the focused row. Each window keeps its own place in the settings file (pattern_editor_303_1_pos, pattern_editor_303_2_pos). The unreleased pattern_editor_pos key is not read. Edit > Clear Pattern (no shortcut) empties the target pattern's steps and keeps its shuffle and length, as one write with Undo. Its status line message is always shown, also on an already empty pattern; the editors' Clear buttons share this code. Copy, Cut, Paste, Duplicate and Clear Pattern act on the pattern shown by the focused pattern editor, else on the selected pattern of the panel's focused row (App::pattern_target). Paste checks the clipboard's device against that editor's device. Focus rule: an editor has the focus when it was opened last or a primary press landed on its window last. It loses the focus to a press on the panel itself or when its window closes; the menu bar does not take it. The survey of the 303 name forms is in the notes. Nothing was unified; that is task-293. docs/ui_spec.md sections 4, 16, 17, 18, 20 and 23 are updated, and the Keyboard Shortcuts help is unchanged (task-284's cuts hold). Tests: cargo test --workspace passes 1113 tests (0 failed, 5 ignored). New tests cover the target rule for 303-1, 303-2, the 808 and the 909, two windows open at once, and their places. Clippy -D warnings is clean on macOS and x86_64-pc-windows-msvc, and nightly fmt --check is clean. The user signed it off on 2026-11-24 after one review round, which removed the code that seeded 303-1's place from the old key and made Clear always report. Branch worktree-agent-abfab1cbfa1e731a9, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
