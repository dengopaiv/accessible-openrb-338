---
id: TASK-201
title: 'Mods menu: the selected mod''s state in the accessibility tree'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T20:48:00Z'
labels:
  - ui
  - rbm
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - docs/accessibility.md
priority: medium
type: enhancement
ordinal: 222000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: found by TASK-184 (the menu bar from the keyboard) and its review, 2026-11-05. In the accessibility tree the Mods menu's entries are Buttons with no checked state (`selectable_label` in crates/orb-ui/src/menu.rs, `menu_bar`), so a screen reader cannot tell which mod plays: the visual mark of TASK-093 (the current song's mod is selected) is not in the tree. The Options menu's checkbox and radios already carry their state (Toggled), so only the Mods entries and the Standard ReBirth entry lack it; the variants submenu (`SubMenuButton` with `selected`) likewise.

What: give each Mods menu entry (Standard ReBirth, each mod, each variant of a merged name) the selected state the panel shows, in the tree (e.g. a checked menu item role, or `Toggled::True` on the selected one), in both modes (it only adds information). The mod chooser dialog of TASK-195 (screen reader mode) marks "current" in its own list; this task is the menu itself.

References: crates/orb-ui/src/menu.rs (menu_bar, mod_checks), crates/orb-ui/tests/a11y_menu_bar.rs (TASK-184's tests), crates/orb-ui/tests/common/a11y.rs, docs/accessibility.md section 6.1.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A test opens the Mods menu with a song on a mod and finds exactly the playing kit's entry marked selected in the tree, for the built-in kit, a mod and a variant of a merged name
- [ ] #2 The menu's look and behavior are unchanged (existing menu tests pass)
<!-- AC:END -->
