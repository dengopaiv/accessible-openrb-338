---
id: TASK-188
title: Sections, the focus order and the key conflicts
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T08:07:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-187
  - TASK-178
  - TASK-179
  - TASK-182
  - TASK-207
references:
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/keys.rs
  - docs/ui_spec.md
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: feature
ordinal: 211000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-4; the analysis is docs/accessibility.md. Read both before starting.

- What, in screen reader mode only:
  - The sections of docs/accessibility.md 3.2 as Group nodes with names, in that order; F6 and
    Shift+F6 move the focus to the first control of the next or previous section; Tab and
    Shift+Tab move within the panel in section order (wrapping at the ends of the panel).
  - The row focus follows the section the focus is in (the two 303s, the 808, the 909), and the
    change is announced through A11Y-5's announcer when that exists, else by the section name.
  - The key map of D6: Tab no longer taps the 303 (Ctrl+Tab and Ctrl+Shift+Tab do), Space
    presses or toggles the focused control and is no longer Play/Stop (D2), F5 is Play/Stop,
    F9 Record, F11 Loop, F12 song/pattern mode (each the same `Panel::press` as the panel's
    button), Enter keeps the original's meaning, Up/Down
    change the focused value instead of the row focus, Ctrl+Up/Down move the row focus.
  - egui's own arrow-key focus movement is suppressed on the panel (the arrows change values).
  - The original's play and program keys (letters, digits, minus, equals, Enter in Program
    Synth mode) keep working, whatever control has the focus.
  - Auto-repeat of Up/Down/Page Up/Page Down on a focused value control is honored (the
    original ignores repeats for its own keys; that stays).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: F6 visits the sections in order and wraps; Tab visits the controls of the panel in section order; the focused node after each step has the expected name.
- [x] #2 Tests: while a 303 plays, Tab moves the focus and does not change the pattern; Ctrl+Tab taps the playing step as Tab did (Ctrl+Shift+Tab a rest); F5, F9, F11 and F12 give the same engine events as the panel's Play/Stop, Record, Loop and mode buttons; Space on a focused toggle toggles it and does not play; Enter with a 303 focused in Program Synth mode is the 303's Step.
- [x] #3 With the mode off, `crates/orb-ui/tests/keyboard.rs` passes unchanged, and a test shows Tab and Up/Down behaving as on main.
- [x] #4 docs/ui_spec.md section 10 has a "Screen reader mode" part with the key map, and Help > Keyboard Shortcuts lists the mode's keys when it is on.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. a11y/nav.rs (new): the 9 sections of docs/accessibility.md 3.2 (names::section_name), the panel's focus order in screen reader mode (sections in order; within a section the panel's order, except Transport, which follows 3.2's list, and a machine's pattern selector, which comes first), next/previous control (wraps at the ends of the panel), next/previous section (its first control), the row of a section.
2. keys.rs: `map_screen_reader(k, shift, ctrl, mode, focus)` -> `SrAction` (Panel(Action) | Control(prev) | Section(prev) | Activate | Help): D6's map (F1 help, F5 Play/Stop, F6/Shift+F6, F9 Record, F11 Loop, F12 mode, Tab/Shift+Tab controls, Space activate, Ctrl+Up/Down row focus, Ctrl+Tab/Ctrl+Shift+Tab tap, the value keys and Left/Right nothing, every other key `map`'s). F2-F4 left free (TASK-189); F10 stays the menu's.
3. app.rs, screen reader mode only: `App::keyboard` routes through `map_screen_reader` (Tab and F6 move egui's focus with `request_focus` in the nav order; Space runs `mouse_down` of the focused control, the AccessKit Click's path; auto-repeat honored for Tab/F6 and the value keys only); egui's own Tab and arrow focus moves cancelled (`move_focus(FocusDirection::None)`) while the focus is on the panel or nowhere; the row focus follows the section the navigation focus enters (tracked per frame, so Ctrl+Up/Down is not undone); the controls drawn into one child Ui per section with role Group and the section's name (UiBuilder accessibility_role/label), so a screen reader says the section when the focus enters it (the announcement proper is TASK-189's). With the mode off nothing changes.
4. key_help.rs: a "Screen reader mode" group, shown when the mode is on, its binds checked against `map_screen_reader` both ways (as the other groups against `map`).
5. Tests: tests/a11y_sections.rs (F6 order and wrap, Tab order with names, Shift+Tab, groups in the tree, the row focus follows; Tab during 303 playback moves the focus only, Ctrl+Tab/Ctrl+Shift+Tab tap, F5/F9/F11/F12 events equal the button clicks, Space on a toggle, Enter is Step; mode off: Tab taps and Up/Down move the row as on main). a11y_baseline: a screen reader mode row. keyboard.rs unchanged.
6. Docs: docs/ui_spec.md 10 "Screen reader mode" part with the key map and 20.2 for the sections; docs/accessibility.md 3.2/3.4 pointers; tools/uia_tree.ps1 output if the probe's tree changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ae287232a2daf446e

Research (egui 0.37.0 sources, ~/.cargo/registry/src/*/egui-0.37.0):
- Tab order = registration order (memory/mod.rs:659 `Focus::interested_in_focus`); Tab/arrows set `focus_direction` in `Focus::begin_pass` (:588-600), acted on as widgets register (Tab) and in `end_pass` (:635, arrows). So `Memory::move_focus(FocusDirection::None)` called in App::keyboard, before any widget of the frame registers, cancels both for the frame. Done only in screen reader mode with the focus on the panel or nowhere; a window's focus keeps egui's Tab.
- A node's parent is the nearest Ui with a node (context.rs:665 `find_accesskit_parent`; `Ui::interact_opt` registers the Ui as parent, ui.rs:956). `UiBuilder::accessibility_role` / `accessibility_label` (ui_builder.rs:214-223) give a child Ui a named node (ui.rs:323-331). So the section Groups are 9 child Uis (`nav::group`), the controls drawn into them; `register_accesskit_parent` is pub(crate), so the 303's edit step label (no widget) gets a hover-only `interact` in the mode to sit in its group.
- `request_focus` from App::keyboard (before the widgets) moves egui's focus; the AccessKit tree's focus follows at the end of the frame.

Design choices (within the task, not decisions): within a section the layout.tsv order, with a machine's pattern selector first (3.2 lists it first) and the transport in 3.2's listed order (Play first), so F6 lands on Play / the pattern selectors. Tab wraps within the panel (the menu bar by F10). The row focus moves only when the navigation focus enters another section (App::nav_section), so Ctrl+Up/Down is not undone each frame. Auto-repeat honored for Tab/F6 and the value keys; F-keys, Space, Ctrl+Tab ignore it as the original's keys. The section's name is spoken by the reader when the focus enters its Group; the announcement proper (A11Y-5's announcer) comes with TASK-189. F2-F4 left free and not listed in Keyboard Shortcuts (not bound on main at 3a82419).

Failed first tries in the tests: the groups' order read from `by_role` is hash order (now read from the parent's children); mode-off Up/Down after Tab moved egui's focus up into the menu bar, which then takes the keys (TASK-202, as on main), so the mode-off test presses the arrows first; Space/Enter vs mouse on tb303_2.step sends no engine event in a stopped pattern (the edit step display is compared instead).

Numbers (headless, a11y_baseline): mode off unchanged 203 nodes / 196 named / 194 focusable; screen reader mode 212 / 205 / 9 Group / 194. Focus order: 190 stops (186 controls + 4 selectors). tools/uia_tree.ps1 (new -ScreenReader switch) on the release build: 207 elements, 9 Group, 194 focusable all named; the same without the switch, because NVDA runs on this machine (SPI_GETSCREENREADER = true, so "auto" is on).

Verification (2026-11-06, main at 3a82419, no rebase needed):
- cargo test --release -p orb-ui --no-fail-fast: 339 passed, 1 failed (crash.rs a_deliberate_panic_exits_101_and_writes_the_log, the known TASK-200 failure), 3 ignored. keyboard.rs 15/15 unchanged. New: a11y_sections.rs 11, a11y_baseline +1, lib +6 (nav 3, keys 1, key_help 3 new, existing updated for groups(is_mac, screen_reader)).
- cargo clippy --workspace --all-targets -- -D warnings: clean. cargo +nightly-2026-10-18 fmt --all --check: clean.
- AC1: a11y_sections f6_visits_the_sections_in_order_and_wraps (10 F6 presses: each focused node = the section's first stop, its name = names::name, its parent Group = the section, wraps to Play; row focus [0,0,1,2,3,3,3,3,3,3]; Shift+F6 back); tab_visits_the_controls_of_the_panel_in_section_order (190 Tabs, each focused id/name/group as nav::order, wrap, Shift+Tab, auto-repeat).
- AC2: tab_moves_the_focus_while_a_303_plays_and_ctrl_tab_taps; f5_f9_f11_and_f12_are_the_transport_buttons (events equal to mouse clicks of mode, record, loop, play, stop); space_presses_the_focused_control_and_does_not_play (5 controls vs mouse, a knob takes none); enter_keeps_the_original_meaning_whatever_has_the_focus (tb303_2 focused: the Step as the mouse's, the edit step moves, nothing plays; V and the patterns key 3 as mode off; an 808 control: Play).
- AC3: keyboard.rs unchanged and passing; with_the_mode_off_the_keys_are_as_before (Up/Down row focus, Tab taps and egui's own Tab focus on transport.shuffle, Ctrl+Tab and F1-F12 nothing, Space plays); f1_opens... mode off nothing.
- AC4: docs/ui_spec.md section 10 "Screen reader mode" table; key_help.rs screen_reader_group shown by groups(is_mac, App::screen_reader_mode()), tested both ways against keys::map_screen_reader. F2-F4 not listed: not bound on main (TASK-189).

Review round 1 (send back): 1. Tab cannot reach the Keyboard Shortcuts, Info, About and Mods > Info windows (egui's Tab cancelled while the focus is on the panel; the windows take no focus): move the focus into a window as it opens in screen reader mode, treat focus in an open window as not the panel's, give the focus back on close. 2. The user's decision (2026-11-06): Enter follows the section, not the row focus, in screen reader mode (a 303's section: Step in Program Synth mode; elsewhere Play); recorded as a Done Decide task. 3. key_help.rs doubled doc comment on groups(). 4. More tests: the row focus kept across F10 and a dialog; auto-repeat ignored by F5/F9/F11/F12/Space/Ctrl+Tab and honored by F6/Tab; literal names of the first three and last stops of Transport and Bass Line 1. 5. Prove or mark unproven uia_tree.ps1 -ScreenReader.

Review round 1, fixes (amending 646be07):
1. Windows (app.rs `App::window_focus`, `app::focus_into_window`, called at the start of the contents of Keyboard Shortcuts, File > Info, Help > About, Mods > Info): in screen reader mode, the focus moves into a window at its second frame (the first is egui's sizing pass: a move at the first, tried first, landed on an unnamed Window-role node), with FocusDirection::Next at the start of its contents, so the first content widget takes it: Info's first text field, About's and Mods > Info's first link, Keyboard Shortcuts' first line (made a focusable label in screen reader mode, as the window has no control but the title bar). Focus in a window is not the panel's, so egui's Tab works there; Tab from the window's last widget would go to the panel's first control (windows draw after the panel), so it is sent back to the window's first (window_tab). The title bar is registered before the contents in egui 0.37 (close button, then the unnamed __title_click area): Shift+Tab from the first reaches them; a Previous move to wrap was tried and landed on the unnamed title click node, so Tab cycles the contents only. On close the panel control that had the focus last gets it back. Tests: f1_opens_the_keyboard_shortcuts_in_screen_reader_mode (focus on the line, Tab cycles in the window, Shift+Tab x2 the close button, Space closes, focus back; Escape: focus back), tab_works_in_a_window_opened_from_the_panel (Info: text field, Tab to the next field, Escape, back to Play), every_window_takes_the_focus_and_gives_it_back (all four).
2. Enter follows the section (decision TASK-207, created Done in this commit, @dengopaiv, m-7): App::keyboard maps Enter by the focused control's section (a 303's: its row; other sections: Press transport.play); with no panel focus, the row focus. Test enter_follows_the_section (Shift+F6 from Bass Line 1 to Transport: Enter = the Play button's events; Delay on: Play; Bass Line 1 after Ctrl+Down moved the row: the first 303 steps). docs/ui_spec.md 10 and the key help row updated; docs/accessibility.md 3.4 notes the refinement.
3. key_help.rs groups() doc comment merged.
4. Tests added: the_menu_and_a_window_keep_the_row_focus (Ctrl+Down twice, F10 to File, Escape, focus back to the same section, F1/Escape: row 2 kept); auto_repeat_moves_the_focus_only (F12, F9, F11, F5 x2, Space, Ctrl+Tab held with repeats = one click each; F6 and Tab held with 2 repeats = 3 steps); the_order_of_two_sections_by_name (Transport: Play, Stop, Record ... Fast forward; Bass Line 1: Pattern, Pattern section on, Shuffle ... Channel 1 compressor).
5. tools/uia_tree.ps1: the switch is now -ScreenReader auto|on|off (the setting written into the throwaway file). Proven on the release build with NVDA running: -ScreenReader off 198 elements, no Group, 194 focusable; -ScreenReader on 207 elements, 9 Group, 194 focusable.
Deferred by-ear check added to TASK-197 as AC #4 (NVDA says the section name on entering a section, reads Keyboard Shortcuts' first line on F1).
Verification: cargo test --release -p orb-ui --no-fail-fast 345 passed, 1 failed (crash.rs, TASK-200), 3 ignored; a11y_sections 17; keyboard.rs unchanged; clippy --workspace --all-targets -D warnings and nightly-2026-10-18 fmt --check clean. main still at 3a82419.

Review round 2 (send back): 1. Keys leak from a focused window (Keyboard Shortcuts, Info, About, Mods > Info, non-modal) into the panel: Enter on a link or the close button plays or steps. In screen reader mode, while the focus is inside an open window, the panel's own keys wait (the original's keys, Enter, Space, the arrows, Tab routing); F1, F5, F6, F9, F11, F12 keep working; Escape closes the window. 2. The second-frame focus move needs a request_repaint on open; window_frames is shared, so a second window opened while one is open got no focus move; define the case of a window opening under a modal. 3. Reword TASK-197 AC #4 to "NVDA reads the section's name when Tab or F6 enters a section (the group name, or TASK-189's announcement)".

Review round 2, fixes (amending cfc232e):
1. App::keyboard, screen reader mode: with the focus in a window (not a panel control, not nowhere) only F1, F5, F6, F9, F11 and F12 pass; the original's keys, Enter, Space, the value keys, Ctrl+Up/Down and Ctrl+Tab wait (Escape stays the window's). Found while testing: egui's own arrow focus move took the focus from About's link to a Bass Line 2 button (by position), which moved the row focus; so in a window the arrows' focus moves are cancelled too (move_focus(None) when an arrow and no Tab was pressed), Tab stays egui's. Test the_panel_keys_wait_while_a_window_has_the_focus (About's link: Enter, Space, V, Up, Insert, Ctrl+Tab, Ctrl+Down: no event, row, edit step, cutoff and focus unchanged; F5 plays).
2. Per-window tracking (app::Win, App::windows_open bit mask, window_new, focus_window): the newest window to open gets the focus move at its second frame, with ctx.request_repaint() so that frame comes; window_focus_due(Win) is taken only by that window. Choice for a modal (browser, prompt, Replace, error box): the move waits until it closes. Test a_second_window_and_a_window_under_a_modal_get_the_focus (Keyboard Shortcuts then About: About's link has it; Keyboard Shortcuts under an error box: no move, after dismiss_error the line has it, Escape gives it back to the cutoff knob).
3. TASK-197 AC #4 reworded: "NVDA reads the section's name when Tab or F6 enters a section (the group name, or TASK-189's announcement) ...".
Docs: docs/ui_spec.md 10 (windows paragraph), the key help group note.
Verification: cargo test --release -p orb-ui --no-fail-fast 347 passed, 1 failed (crash.rs, TASK-200), 3 ignored; a11y_sections 19; clippy and nightly fmt clean; main still 3a82419.

Review round 3 (send back, small): 1. The window gate keyed on `on_panel`, true with no focus, so with a window open and nothing focused the panel keys still acted; gate on "a window is open and the focus is not a panel control". 2. Test a focused Info text field: typed characters and Left/Right reach the field, the panel does not change. 3. docs/ui_spec.md 10 and key_help.rs still said every other key keeps its meaning whatever has the focus: reword for the window (and menu, TASK-202) rule.

Review round 3, fixes (amending 43adec4):
1. App::keyboard: the window gate is now `window_has_keys` = a window open (Win::ALL) and the focus not a panel control (none counts), so the panel's keys also wait in the frames before a window takes the focus and after egui cleared it. Test the_panel_keys_wait_while_a_window_is_open_and_nothing_has_the_focus (About open, nothing focused: Enter, V give no event; focus surrendered from the link: Enter, V, Space no event, nothing plays; F5 plays).
2. Test typing_in_an_info_field_does_not_reach_the_panel (Info's Title field focused: c, v, Left, [, Right, b give "c[vb", no engine event, edit step, row focus and transport unchanged).
3. docs/ui_spec.md 10 (the intro of "Screen reader mode" and the windows paragraph) and the key help group note now say the panel's keys wait while a menu (TASK-202) or a window has the focus, or a window is open and nothing has it.
Verification: cargo test --release -p orb-ui --no-fail-fast 349 passed, 1 failed (crash.rs, TASK-200), 3 ignored; a11y_sections 21; clippy and nightly fmt clean; main still 3a82419.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Screen reader mode now has the sections, the focus order and the key map of decisions A11Y-D2/D3/D6 and TASK-207 (branch worktree-agent-ae287232a2daf446e, one commit on main 3a82419, amended after review rounds 1 to 3).

What works, in screen reader mode only: the nine sections of docs/accessibility.md 3.2 are named Group nodes (child Uis with egui's accessibility role/label) holding their controls; Tab/Shift+Tab go through the panel's 190 stops in section order and wrap (pattern selector first in a machine's section, the transport in 3.2's order); F6/Shift+F6 go to a section's first stop; the row focus follows the section the focus enters and survives the menu bar and windows; F1 Keyboard Shortcuts, F5 Play/Stop, F9 Record, F11 Loop, F12 mode (the buttons' Panel::press), Space = the focused control's mouse down (the AccessKit Click path), Enter by the focused control's section (a 303's: its Step in Program Synth mode; elsewhere Play; TASK-207), Ctrl+Up/Down row focus, Ctrl+Tab/Ctrl+Shift+Tab the 303 tap; egui's own Tab/arrow focus moves cancelled on the panel; the original's keys keep working whatever has the focus; auto-repeat honored by Tab/F6 and the value keys only. The windows Keyboard Shortcuts, Info, About and Mods > Info take the focus as they open (the newest; after a modal closes), Tab cycles in them, the panel keys wait while one has the focus or one is open and nothing has it (but F1, F5, F6, F9, F11, F12), and the panel control gets the focus back as they close. Help > Keyboard Shortcuts lists a "Screen reader mode" group when the mode is on. With the mode off nothing changes (keyboard.rs unchanged, a mode-off test).

Left for others: F2-F4 (TASK-189, not bound or listed here), Left/Right (TASK-190, TASK-191), the spoken confirmation of a row focus change (TASK-189's announcer; the reader says the Group's name as the focus enters it, by-ear check in TASK-197 AC #4), dialog roles and the unnamed title-bar node and Info text fields (TASK-193).

Numbers: tree in the mode 212 nodes / 205 named / 9 Group / 194 focusable (off: 203/196/194, unchanged). tools/uia_tree.ps1 -ScreenReader on: 207 elements, 9 Group, 194 focusable all named; off: 198, no Group.

Tests: cargo test --release -p orb-ui --no-fail-fast 349 pass, 1 known failure (crash.rs, TASK-200), 3 ignored; clippy and nightly fmt clean.

Files: crates/orb-ui/src/a11y/nav.rs (new), crates/orb-ui/src/a11y/mod.rs, crates/orb-ui/src/app.rs, crates/orb-ui/src/keys.rs, crates/orb-ui/src/key_help.rs, crates/orb-ui/src/info.rs, crates/orb-ui/src/about.rs, crates/orb-ui/src/mod_info.rs, crates/orb-ui/tests/a11y_sections.rs (new), crates/orb-ui/tests/a11y_baseline.rs, crates/orb-ui/tests/common/a11y.rs (Tree::parent), docs/ui_spec.md (10 "Screen reader mode", 20.1, 20.2), docs/accessibility.md (3.2, 3.4, 6.3), tools/uia_tree.ps1 (-ScreenReader auto|on|off), backlog: this task, TASK-207 (new, Done), TASK-197 (AC #4).
<!-- SECTION:FINAL_SUMMARY:END -->
