---
id: TASK-212
title: 'Menu bar: the focus returns where it was when the bar is left'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-06T23:49:00Z'
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
- [ ] #1 Tests (screen reader mode): with the focus on a panel control, F10 then Escape gives the focus back to that control; the same with a menu opened (Down) and Escape; with a lone Alt instead of F10.
- [ ] #2 Tests: F10 then a second F10 (and a lone Alt then a second lone Alt), on a title and with a menu open, close the menu and give the focus back to the control.
- [ ] #3 Tests: choosing an item that opens no window (e.g. Options > Step Editor Follows Playback) gives the focus back to the control; an item that opens a window moves the focus into it, and closing the window returns it to the control (not the Window node).
- [ ] #4 With screen reader mode off, tests/a11y_menu_bar.rs and tests/keyboard.rs pass unchanged, except where the notes say the second F10 / Alt was made to work in both modes; docs/ui_spec.md 18.2 and the key help describe the behavior; the by-ear check is a new item of the NVDA checklist.
<!-- AC:END -->
