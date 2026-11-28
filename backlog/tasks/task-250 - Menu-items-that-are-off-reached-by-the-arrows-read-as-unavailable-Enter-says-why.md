---
id: TASK-250
title: >-
  Menu items that are off: reached by the arrows, read as unavailable, Enter
  says why
status: Done
assignee: []
created_date: '2026-11-28T10:55:00Z'
updated_date: '2026-11-28T11:30:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - docs/accessibility_plan.md
  - docs/accessibility.md
  - docs/ui_spec.md
priority: high
type: bug
ordinal: 267000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user's NVDA pass of 2026-11-28 (docs/accessibility_plan.md Part 3, items 26, 29 and 38). The user could not find File > Export Loop as Audio File, nor Edit > Initialize Song from Pattern Mode ("There's no initialize song in the edit menu"). Both are turned off outside song mode (menu.rs: `ui.add_enabled(ok, ...)` for Export, `item_if(ui, self.can_init(), ...)` for the Initialize items), and egui does not give the focus to a widget that is turned off, so Up and Down in an open menu skip it and NVDA never says it is there. A user who does not know the item exists cannot learn that it needs song mode.

The user's decision, 2026-11-28: Windows' way. In a Windows menu (Notepad, Explorer) an item that is grayed out is still reached by Up and Down, and the screen reader reads it with "unavailable". The user chose that, and that Enter on such an item says why it is off.

What to build, in screen reader mode only (with the mode off, the menus stay exactly as they are: the ground rules, docs/accessibility_plan.md Part 4):
- Every menu item that menu.rs draws turned off stays in the Up/Down order of its menu (and Home/End if the menus have them), and its AccessKit node carries the disabled state, so NVDA reads it as "<name>, unavailable" (check the node in the a11y tree tests; egui sets `is_disabled` for a widget that is off). The items today: File > Info... (no song), File > Export Loop as Audio File... (not song mode, or an export running), Edit > Undo (nothing to undo), the five pattern commands and the four pattern editors (no song), Edit > Initialize Loop / Song from Pattern Mode (no song, or not song mode). Look for any other item that is off in the bar (Mods, Options, Help) and treat it the same.
- Enter or Space on such an item does not run its command and does not close the menu; the focus stays on the item, and the reason is spoken once (polite announce), short and plain, with the key that fixes it where there is one. Suggested texts: "Needs song mode; F12 switches" (Export, the Initialize items), "An export is running" (Export during one), "Needs a song" (no song), "Nothing to undo" (Undo). The existing hover texts ("Export needs song mode.", "Needs song mode.") stay for the mouse.
- Nothing else changes: the enabled items, the order, Left/Right between menus (task-247), the questions that a menu command opens (task-249).

The reason texts and how an item that is off is made focusable are the agent's design; note them in the task. A clean way may be to draw the item enabled in screen reader mode and mark its node disabled, or egui may offer a focusable-but-disabled path; whichever is used, the mouse in screen reader mode must still not run a command that is off.

Docs: docs/accessibility.md (the menus' part, marked "Does, not yet heard"), docs/accessibility_plan.md (Part 1.2 the work, Part 2.1 the waiting table, and a new Part 3 checklist item with the steps; items 26, 29 and 38 already say to press F12 first), docs/ui_spec.md where the menu bar's screen reader keys are (18.2). Add the by-ear check as a new acceptance criterion of TASK-244 (with the checklist item's number).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 In screen reader mode, Up and Down in an open menu reach every item that is off, in its place, and its AccessKit node is marked disabled (a test over the a11y tree, for File > Export in pattern mode and Edit > Initialize Song from Pattern Mode in pattern mode at least)
- [x] #2 In screen reader mode, Enter or Space on an item that is off runs nothing, keeps the menu open with the focus on the item, and announces the reason once (tests for Export and Initialize Song in pattern mode, and for one item that is off for lack of a song or of undo)
- [x] #3 With screen reader mode off the menus behave as before: items that are off are skipped and inert (a test)
- [x] #4 docs/accessibility.md, docs/accessibility_plan.md (Part 1.2, 2.1, a new Part 3 item) and docs/ui_spec.md describe it; TASK-244 has the by-ear check as a new criterion naming the checklist item
- [x] #5 cargo test, clippy -D warnings and nightly fmt --check pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. menu.rs: item_off_r draws an item that is off enabled+grayed in screen reader mode, marks the AccessKit node disabled, and on click/Enter/Space sets a reason (not closing the menu); menu_items announces it once at the end. Applied to Info, Export, Undo, pattern commands/editors, Initialize items, Demo Song.
2. Mouse still runs nothing (go=false). Mode off: unchanged path (item_if).
3. Test tests/a11y_menu_off.rs. 4. Docs: accessibility.md, accessibility_plan.md (1.2, 2.1, item 39), ui_spec.md 18.2; TASK-244 criterion 11.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-af9f7314702c54e14

Design: item_off_r (menu.rs). In screen reader mode an item that is off is drawn enabled in the weak text color (so egui gives it the focus), its AccessKit node gets set_disabled() (NVDA: unavailable), and click/Enter/Space sets a static reason, no command, no ui.close(); menu_items announces it once at the end via App::announce. Mouse click gets the same path, so nothing off runs. Mode off: the old item_if path, hover texts kept. Reasons: Needs a song / Needs song mode; F12 switches / An export is running / Nothing to undo / This mod has no demo song (Demo Song also treated, the only other item off in Mods/Options/Help). Existing test a11y_menu_bar on_first_item expected the focus to stay on the Edit title with no link; now Undo is the first reachable item in screen reader mode, test updated. Tests: tests/a11y_menu_off.rs (4). cargo test --release --workspace 1230 pass, clippy and nightly fmt clean.

Review round 1 (send back), fixed: (1) Export with no song says 'Needs a song' (link None), 'An export is running' in song mode, else 'Needs song mode; F12 switches'; new test export_with_no_song_says_it_needs_a_song. (2) hover texts ('Export needs song mode.', 'Needs song mode.') now show in screen reader mode via on_hover_text; ui_spec 18.2 says so. (3) vacuous assertion removed from the mode-off test (the reach check is the direct one). orb-ui a11y_menu_off 5 and a11y_menu_bar 19 pass, clippy and nightly fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
In screen reader mode every menu item that is off (File > Info, Export, Edit > Undo, pattern commands and editors, Initialize Song and Loop, Mods > Demo Song) stays in the Up/Down order, its node is disabled, and Enter or Space or a click runs nothing, keeps the menu open on the item and announces the reason once. Mode off unchanged. Code: crates/orb-ui/src/menu.rs (item_off_r); tests crates/orb-ui/tests/a11y_menu_off.rs, a11y_menu_bar.rs adjusted; docs accessibility.md, accessibility_plan.md (1.2, 2.1, item 39), ui_spec.md 18.2; TASK-244 criterion 11 added. cargo test --release --workspace 1230 pass, clippy -D warnings and nightly fmt --check clean. Branch worktree-agent-af9f7314702c54e14.
<!-- SECTION:FINAL_SUMMARY:END -->
