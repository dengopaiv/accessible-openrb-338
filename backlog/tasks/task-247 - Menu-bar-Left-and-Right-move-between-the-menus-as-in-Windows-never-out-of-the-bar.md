---
id: TASK-247
title: >-
  Menu bar: Left and Right move between the menus as in Windows, never out of
  the bar
status: To Do
assignee: []
created_date: '2026-11-28T06:50:00Z'
updated_date: '2026-11-28T06:56:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - docs/ui_spec.md
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/tests/a11y_menu_bar.rs
priority: medium
type: bug
ordinal: 264000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: found by the user in the NVDA pass of 2026-11-28 (TASK-244, reported next to checklist item 26): "Left in the menus moves back to the menu bar, but doesn't allow to move then to the other menus. This should either be removed, or made consistent: up down move between menu items, left right move between menu columns ... One right from the help menu moves to the transport section, it shouldn't leave the menu like that." Asked how it should work, the user chose Windows' way (2026-11-28).

Today (docs/ui_spec.md 18.2): the menu bar's keys are egui 0.37's own (containers/menu.rs, popup.rs, MenuState::handle_keyboard). On a focused title Left / Right move egui's spatial focus to the neighboring title, without wrapping, and from the last title (Help) Right moves the focus out of the bar to a panel control (the Transport section). In an open menu Left returns the focus to the title, after which the other menus cannot be reached the same way. OpenRB-338 adds F10 / a lone Alt, Down opening a menu (open_on_down), the panel's keys withheld while the bar is in use (menu_has_keys, task-202) and, in screen reader mode, the focus back to the panel control when the bar is left (task-212, App::menu_bar_focus, menu_return). The bar has six titles: File, Edit, View, Mods, Options, Help.

Wanted, in screen reader mode (with the mode off the menus stay as they are, egui's, as upstream; the ground rule of docs/accessibility_plan.md Part 4):
- On a focused title: Left / Right move to the previous / next title, wrapping (Help -> File, File -> Help); the focus never leaves the bar for the panel.
- In an open menu: Up / Down walk its items as now; Left / Right close it and open the previous / next menu with the focus on its first item, wrapping the same way.
- A submenu (the Mods menu's merged names, whose files are a submenu): Right on its item opens it with the focus on its first file; Left inside it closes it and returns to its item; Right on a file of an open submenu goes on to the next menu.
- Escape, Enter, F10 and a lone Alt as today (task-212's focus back included).
NVDA should hear each title or item once per move (no double speech).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In screen reader mode, Left and Right on a title and in an open menu go through all six menus in both directions, wrapping at File and Help, an open menu opening the next with the focus on its first item; covered by a11y tests
- [ ] #2 Right from Help (its title or its open menu) goes to File, never to a panel control; a test
- [ ] #3 Right on a Mods submenu item opens it, Left in it goes back to its item; a test (with a mods folder of merged names, as the existing Mods menu tests make)
- [ ] #4 With the mode off the menu bar's keys are as before (the existing a11y_menu_bar and keyboard tests pass unchanged); docs/ui_spec.md 18.2, Help > Keyboard Shortcuts, CHANGELOG.md's next section, docs/accessibility_plan.md and docs/accessibility.md updated; one by-ear criterion added to TASK-244
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Supervisor, 2026-11-28 (review of the pass record): docs/ui_spec.md 18.2's last bullet still says the bar has five titles; it has six since task-233 (View). Fix it with this task's 18.2 update.
<!-- SECTION:NOTES:END -->
