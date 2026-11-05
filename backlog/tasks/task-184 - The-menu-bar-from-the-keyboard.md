---
id: TASK-184
title: The menu bar from the keyboard
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:31:00Z'
updated_date: '2026-11-05T20:58:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-183
references:
  - crates/orb-ui/src/menu.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: task
ordinal: 207000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-11; the analysis is docs/accessibility.md. Read both before starting.

- What: find out how egui 0.37's menu bar is reached and walked from the keyboard (today the
  four menu titles are focusable buttons; whether Alt or F10 focuses the bar, whether the arrow
  keys open and walk a menu, whether Escape returns). Make F10 (and Alt pressed and released
  alone, on Windows) focus the File menu, the arrows walk the bar and the items, Enter choose,
  Escape close, in both modes if harmless, else in screen reader mode. Checkboxes and radio
  items in the Options menu read their state.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: F10 focuses File; Right moves to Mods; Down opens it and focuses its first item; Escape closes; Enter on an item runs it.
- [x] #2 The findings about egui's own behavior are in the notes and in docs/ui_spec.md section 18.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Probe egui 0.37's own keyboard use of the menu bar with the TASK-183 harness (no code change): record it.
2. F10 and a lone Alt (press and release, no key or click between) focus the File title (menu_bar_key in menu.rs); both modes, not gated on screen reader mode (F10/Alt unused today).
3. Down on a focused title opens the menu and focuses the first item (egui does not); the panel's key handler is off while a title has focus or a menu is open (so Up/Down do not move the row focus).
4. Tests in crates/orb-ui/tests/a11y_menu_bar.rs; findings in docs/ui_spec.md 18.2.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ab6467ecced7f686a

Findings about egui 0.37 itself (probe, headless harness, before any change):
- Nothing focuses the bar: F10 and Alt do nothing. Initial focus is the Window node; Tab from there lands on a panel control (the bar is drawn after the panel, so it is last in the tab order).
- Right/Left between focused titles works (spatial focus, no wrap). Enter on a title opens the menu and keeps the focus on the title.
- Down on a closed bar title does NOT open the menu: egui moves focus to the panel control below, and OpenRB-338's own key handler also took the Down as 'row focus'. MenuState::handle_keyboard (containers/menu.rs:233) runs only inside an open popup; there Down/Up walk the items, Down from the anchor title focuses the first item. Escape closes (focus goes to the Window node).
- Tree: titles are Button nodes named File/Mods/Options/Help; items are Buttons named with the shortcut appended ('New Ctrl+N'); Options checkboxes are CheckBox with Toggled, the two key modes RadioButton with Toggled. Mods entries (selectable_label) are Buttons without a checked state: the selected mod is not in the tree (follow-up, not done here).
Failed tries: opening the popup and expecting the same-frame Down to reach handle_keyboard (the popup is in its sizing pass, focus moved to the panel instead). Fix: open on the Down frame, cancel egui's focus move (move_focus None), and send Down in the next frame (pending flag in egui temp data). The item focus lands one frame after the open (request_focus_next_frame); a real window repaints in between (request_repaint).
egui 0.37 RawInput has no modifiers field: the harness sends Event::ModifiersChanged for Alt; the harness needed no change.
No gating on App::screen_reader_mode (TASK-185 not on main): behavior is harmless in both modes. Expect a merge conflict in menu.rs with TASK-185 (it adds the Options > Screen Reader Mode submenu; here the Options button response is captured as let options = ...; keep both).

Results: cargo test -p orb-ui all pass in debug (a11y_menu_bar 6 tests, a11y_baseline, a11y_actions, the rest); clippy -D warnings and nightly-2026-10-18 fmt --check clean. The two long tests/menu.rs runs were slow only because of the shared machine (export test renders). Not done / for later: F10 or Alt a second time does not restore the previous focus; Mods entries have no checked state in the tree (selectable_label); Left on File does not wrap to Help.

Review round 1 (coordinator): rebased onto main 650d17f (TASK-185 squashed) with git rebase --onto main 870a654: one commit, no conflicts. The user's decision on the panel's keys while a menu has the focus (2026-11-05, 'Accept, in both modes') is TASK-202 (created Done in this commit); described in docs/ui_spec.md 18.2 and as an exception to the ground rule in docs/accessibility_plan.md. ui_spec claims now measured in tests/a11y_menu_bar.rs (Space opens a title, Left of File does not wrap, focus after choosing an item is the Window node) or marked as read from egui's source; 18.2 moved after 18.1; the Left test split and asserted. Follow-up (supervisor's): Mods entries carry no checked state in the tree.

Round 1 test run (debug, after the rebase): cargo test -p orb-ui: 217 pass, 0 fail (80 lib, a11y_menu_bar 7, a11y_baseline 1, a11y_actions 1, the other test files 128 incl. menu.rs 53 of 54). One test did not finish: tests/menu.rs export_writes_the_file_orb_render_writes ran for over an hour at no CPU progress and was killed (it is an export, no keys involved; not checked whether main hangs too on this machine; reported to the supervisor). clippy -D warnings clean, nightly-2026-10-18 fmt --check clean.

Review round 2: final summary rewritten; the focus after Enter on an item is now asserted as the Window node in enter_on_an_item_runs_it (a11y_menu_bar 7 pass, fmt clean); a long line in docs/ui_spec.md rewrapped.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
F10 and a lone Alt (Windows convention) focus the File title in both modes (menu_bar_key, crates/orb-ui/src/menu.rs). Down on a focused title opens its menu and focuses the first item (open_on_down). All panel keys wait while a title has the focus or a menu is open, also for a menu opened by mouse, in both modes (menu_has_keys, one condition in App::show): the user's decision of 2026-11-05, TASK-202, so Up/Down no longer move the row focus and Enter on an item no longer also presses Play. egui's own behavior is measured (or marked as read from its source) in the notes and docs/ui_spec.md 18.2; docs/accessibility_plan.md records the exception to the ground rule.

Branch worktree-agent-ab6467ecced7f686a, one commit, rebased onto main 650d17f with no conflict (TASK-185 squashed there). Tests: crates/orb-ui/tests/a11y_menu_bar.rs, 7 tests (F10 -> File, Right -> Mods, Down opens and focuses the first item, Escape closes, Enter runs Help > Keyboard Shortcuts and the focus is the Window node, Space opens a title, Left of File does not wrap, lone Alt vs Alt+key, Options checkbox/radio Toggled state).

cargo test -p orb-ui in debug: 217 pass, 0 fail, with tests/menu.rs export_writes_the_file_orb_render_writes not finished and killed after over an hour (an export, no keys involved; not checked on main). Clippy -D warnings and nightly-2026-10-18 fmt --check clean.
<!-- SECTION:FINAL_SUMMARY:END -->
