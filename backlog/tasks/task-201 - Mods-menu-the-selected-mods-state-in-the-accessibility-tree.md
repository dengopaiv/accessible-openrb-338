---
id: TASK-201
title: 'Mods menu: the selected mod''s state in the accessibility tree'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T20:48:00Z'
updated_date: '2026-11-07T09:10:00Z'
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
- [x] #1 A test opens the Mods menu with a song on a mod and finds exactly the playing kit's entry marked selected in the tree, for the built-in kit, a mod and a variant of a merged name
- [x] #2 The menu's look and behavior are unchanged (existing menu tests pass)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Test first (a11y_mod_chooser.rs): open Mods menu, find checked entries for built-in kit, mod, variant. 2. Found egui already sets Toggled from selectable_label (Button, Toggled::True/False), so the state was in the tree but as a plain Button. 3. Add checked_item() in menu.rs: selectable_label + accesskit_node_builder set_role(MenuItemCheckBox) so UIA/NVDA read 'menu item, checked'. Submenu button for shared names keeps Button role (expanded/popup semantics) with Toggled. 4. NVDA checklist item 25.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ad39d426048ea5753

Finding: the description says the entries had no checked state; measured: selectable_label already gives Toggled True/False, role Button (NVDA reads it as a toggle button, not 'checked'). Chosen: Role::MenuItemCheckBox on Standard ReBirth, each mod entry and each variant file (UIA menu item with Toggle pattern = checked); the variants submenu button stays Button + Toggled (keeps its submenu semantics, same as Options > Screen Reader Mode). Same in both modes; look/behavior unchanged (only an AccessKit role). Existing test with_the_mode_off_there_is_no_chooser asserted Role::Button for the first Mods item; updated to MenuItemCheckBox. fmt check ok, clippy -D warnings ok, cargo test --release -p orb-ui: all pass except the known crash.rs a_deliberate_panic_exits_101_and_writes_the_log (TASK-200).

Review round 1: checked_entries now collects every node with Toggled::True except the panel's own controls (layout::controls ids), so the test shows the playing kit is the ONLY checked entry: {Standard ReBirth}, {Zed}, {4.rbm  1.0.0, Shared}; the other variant file is asserted Toggled::False. fmt, clippy, orb-ui tests: only crash.rs (TASK-200) fails.

Merged by the supervisor 2026-11-07: review round 1 sent back one test defect (the test did not show the playing entry is the only checked one), fixed in 075466f; re-review merge. a11y_mod_chooser (13), a11y_menu_bar (13), a11y_names (6), a11y_sections (21) pass on main after the squash.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Mods menu entries (Standard ReBirth, each mod, each variant file) are Role::MenuItemCheckBox with Toggled from the selection via checked_item() in crates/orb-ui/src/menu.rs; the variants submenu button keeps Role::Button with Toggled::True when its mod plays. Test the_mods_menu_marks_the_playing_kit (a11y_mod_chooser.rs) checks exactly the playing kit's entry for built-in kit, a mod, and a variant (submenu button and file). Existing menu tests pass; only crash.rs fails (TASK-200). NVDA checklist item 25 added to docs/accessibility_plan.md 2.2 (unchecked). Branch worktree-agent-ad39d426048ea5753.
<!-- SECTION:FINAL_SUMMARY:END -->
