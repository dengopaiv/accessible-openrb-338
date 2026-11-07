---
id: TASK-212
title: 'Menu bar: the focus returns where it was when the bar is left'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-06T23:49:00Z'
updated_date: '2026-11-07T08:47:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/tests/a11y_menu_bar.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 230000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user (NVDA, 2026-11-07) asked for the two gaps TASK-184 (the menu bar from the keyboard) left, recorded in its notes and docs/ui_spec.md 18.2:
1. After the menu bar is left with Escape, or after choosing an item that opens no window, the focus goes to the Window node (egui's close: measured in crates/orb-ui/tests/a11y_menu_bar.rs `enter_on_an_item_runs_it`), not back to the control that had it. NVDA has nothing to read there and the user must Tab back into the panel.
2. A second F10, or a second lone Alt, while a title has the focus does not hand the focus back to where it was, as Windows menu bars do.

What:
- Remember the focused widget when F10 or a lone Alt focuses File (`menu_bar_key`, crates/orb-ui/src/menu.rs), and also when a menu is opened by mouse or Enter from somewhere else if that is simple.
- When the menu bar is left (Escape on a title or in a menu; an item chosen that opens no window, a dialog or the system file dialog; a second F10 or lone Alt on a title or in an open menu, which also closes the menu), give the focus back to that widget. If it no longer exists, the panel's first control of the section it was in, else leave it as egui does.
- An item that opens a window (Keyboard Shortcuts, Info, About, Mods > Info, the mod chooser, the prompt, the error box) keeps TASK-193's behavior: the focus goes into the window, and returns when it closes to the control that had it before the menu (check that it is the control, not the Window node, and keep it so).
- Scope: in screen reader mode. With the mode off, the app behaves as now (the fork's F10 / Alt already work in both modes; whether the second F10 / Alt also returns the focus with the mode off is harmless and may be done in both modes, like F10, if it needs no extra code; say which in the notes).
- Not in scope: wrapping Left / Right at the ends of the bar, the Mods menu's checked state (TASK-201), the shortcut in item names.

References: crates/orb-ui/src/menu.rs (`menu_bar_key`, `open_on_down`, `menu_has_keys`, `menu_bar`), crates/orb-ui/src/app.rs (`window_focus`, the focus-return of TASK-193), crates/orb-ui/src/a11y/window.rs, crates/orb-ui/tests/a11y_menu_bar.rs, crates/orb-ui/tests/common/a11y.rs, docs/ui_spec.md 18.2 and 20.4. By-ear check: add it to the NVDA checklist in docs/accessibility_plan.md 2.2 as a new item (the supervisor confirms with the user).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests (screen reader mode): with the focus on a panel control, F10 then Escape gives the focus back to that control; the same with a menu opened (Down) and Escape; with a lone Alt instead of F10.
- [x] #2 Tests: F10 then a second F10 (and a lone Alt then a second lone Alt), on a title and with a menu open, close the menu and give the focus back to the control.
- [x] #3 Tests: choosing an item that opens no window (e.g. Options > Step Editor Follows Playback) gives the focus back to the control; an item that opens a window moves the focus into it, and closing the window returns it to the control (not the Window node).
- [x] #4 With screen reader mode off, tests/a11y_menu_bar.rs and tests/keyboard.rs pass unchanged, except where the notes say the second F10 / Alt was made to work in both modes; docs/ui_spec.md 18.2 and the key help describe the behavior; the by-ear check is a new item of the NVDA checklist.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
State in App: menu_return (panel_focus when the focus goes to a title or a menu opens) and bar_active. menu_bar: second F10/lone Alt while bar_active (screen reader mode) closes popups and requests the focus back; end of frame App::menu_bar_focus gives it back when the bar turns inactive and the focus is not on a panel control, in a window or in a modal. Windows keep window_focus (panel_focus return). Tests in a11y_menu_bar.rs; docs ui_spec 18.2, key help row, plan item 23.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a23ca8757e26132c2

Measured (tests/a11y_menu_bar.rs, 18 tests pass): F10/Alt then Escape, and Down then Escape (a single Escape: the focus went to Play at once), second F10/Alt on a title and with a menu open, Options > Step Editor Follows Playback chosen: all give the focus back to Play. Help > Keyboard Shortcuts: focus goes inside the window (a Label), Escape returns it to Play, not the Window node. enter_on_an_item_runs_it passes unchanged (Window node in the first frame). Screen reader mode only: the state (panel_focus) is tracked only there; with the mode off a second F10/Alt focuses File again as before (not made to work in both modes). Not done: fallback to the section's first control when the remembered control is gone (egui leaves the focus as it does). Mouse-opened menus: remembered the same way (any entry into the bar). Checks: fmt (nightly-2026-10-18) ok, clippy -D warnings ok, cargo test --release -p orb-ui: all pass except crash.rs a_deliberate_panic_exits_101_and_writes_the_log (TASK-200, known). By-ear: new NVDA checklist item 23 in docs/accessibility_plan.md 2.2, unchecked; TASK-211's becomes 24.

Review round 1: escape_in_an_open_menu_gives_the_focus_back now asserts has_play directly after one Escape (F10 and lone Alt); it passes, no code change. Plan item 23 and ui_spec 18.2 say one Escape. fmt, clippy ok; orb-ui tests: only crash.rs (TASK-200) fails.

Merged by the supervisor 2026-11-07: review round 1 sent back one test defect (the single Escape in an open menu was not asserted), fixed in 847a85d; re-review merge. a11y_menu_bar (13) and keyboard (12) pass on main after the squash.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
In screen reader mode the focus returns to the panel control that had it when the menu bar is left (Escape, an item that opens no window, a second F10 or lone Alt on a title or with a menu open); items that open a window keep TASK-193's return, now measured to land on the control. Code: App::menu_bar_focus, menu_return, bar_active (app.rs), menu_bar (menu.rs). 5 new tests in a11y_menu_bar.rs; docs/ui_spec.md 18.2, key help row, plan checklist item 23 (unchecked). Branch worktree-agent-a23ca8757e26132c2. Only known failure: crash.rs (TASK-200).
<!-- SECTION:FINAL_SUMMARY:END -->
