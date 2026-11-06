---
id: TASK-195
title: A mod chooser with a filter
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T22:39:00Z'
labels:
  - ui
  - rbm
milestone: m-7
dependencies:
  - TASK-193
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/mod_info.rs
  - docs/rbm.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 218000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-10; the analysis is docs/accessibility.md. Read both before starting.

- Why: with the user's mods folder the Mods menu holds 311 entries (the 320 files of the rb-mods
  collection; equal names merge with variants), in a scroll area inside a pop-up menu. Walking
  it with the arrows is slow, and egui menus have no type-ahead.
- What: in screen reader mode, the Mods menu's first item is "Choose Mod..." (with a shortcut,
  to decide in the task and add to the key help), opening a dialog with a filter field and a
  list. The list holds the entries of the menu in the menu's order, each read as its name, then
  "variants: N" when it has variants, "has demo" when it has a demo song, and "current" for the
  playing kit. Typing filters by name (ASCII case ignored, as the menu sorts). Enter chooses
  (`use_kit`, or for a merged name the preferred variant, with the variants reachable as
  sub-items), Escape closes, a key opens Mod Info for the selected entry. The built-in
  "Standard ReBirth" is the first entry.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests with a folder of test mods (the existing test mods, or `ORB_THIRDPARTY_MODS_DIR` gated with a skip message): the list equals the menu's entries in order; the filter narrows it; Enter loads the same kit as the menu item.
- [x] #2 With the rb-mods collection, the dialog opens with all 311 entries in under 100 ms after the list is cached (measure; notes).
- [x] #3 With the AccessKit tree of the open dialog, tests show the filter as a named TextInput, the list as a Tree named 'Mods' with TreeItem children (level, position in set, expanded, selected, active descendant) (the by-ear NVDA check moved to TASK-197)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. orb-cli: mods::scan_with_demo (the demo flag with the one read of each file). 2. menu.rs: ModEntry/ModVariant demo, Cmd::ChooseMod, the menu item (screen reader mode only), shortcuts CHOOSE_MOD (Ctrl/Cmd+M) and CHOOSER_INFO (Ctrl/Cmd+I). 3. mod_chooser.rs: a Win::Chooser window, a filter TextEdit and a Tree child Ui (explicit id, FOCUSABLE) with TreeItem nodes and active_descendant, as the drum grid; keys handled before the widgets; choose = the menu's commands. 4. mod_info.rs: show_mod_info_of(kit) with a target kit and its own splash cache; Mods > Info unchanged. 5. app.rs: Win::Chooser, Ctrl+M, focus back into the chooser when Mod Info closes. 6. key help rows, ui_spec 20.7, plan/analysis notes. 7. Tests a11y_mod_chooser.rs on copies of the embedded standard mod with other INFO names (no ISO needed), plus the gated timing test.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-mod-chooser. Decisions of @dengopaiv 2026-11-07: Ctrl+M opens (screen reader mode only, menu item 'Choose Mod...' first in Mods); two focus stops (filter, list), Tab between, Down from filter to list; tree list with expandable variants (Right/Left), Enter on merged = preferred, on variant = ChooseVariant; Ctrl+I opens Mod Info for the selection.

Evidence. Tests: crates/orb-ui/tests/a11y_mod_chooser.rs, 11 tests, all pass in release: the list equals the menu's entries in order (Tree/TreeItem roles, names, level, position/size in set, expanded false on the merged entry, active descendant, filter named 'Filter mods' as TextInput, dialog 'Choose Mod'); the filter narrows (ASCII case ignored, live, selection to the first match); Enter from the filter and from the list loads the same kit/mod_name/modified as Cmd::UseKit on a second app; the variant tree (Right opens, Right again to first file, Left back/closes, Enter on the entry = preferred first by bytes, Enter on a file = Cmd::ChooseVariant; the active file chosen again sets the preference and the list then says 'preferred, current'); Ctrl+I shows Mod Info of the selection without loading it (Name/File lines, preferred file of a merged entry, Standard), Escape closes Mod Info first and the focus returns to the list stop; Escape from either stop; Tab/Shift+Tab between the two stops; menu item 'Choose Mod... Ctrl+M' first in Mods and Enter on it opens the dialog; with the mode off: Ctrl+M and Cmd::ChooseMod do nothing, no 'Choose Mod' item, first item is Standard ReBirth; the dialog closes when the mode goes off. The fixtures are copies of assets/Standard.rbm with the INFO name (offset 1003) replaced, so no CD image is needed. AC 2 (release build, ORB_THIRDPARTY_MODS_DIR=C:\GIT\music\rb-collections\rb-mods\mods): 311 entries (312 items with Standard ReBirth); list read (existing scan, with the demo flag added) 2.77 s; frames from Ctrl+M [4.87, 1.70, 1.55, 1.55, 1.63] ms, steady frame with the dialog open 1.60 ms, a frame without it 2.17 ms, a letter typed 7.1 ms with 3 settle frames. Reproduce: ORB_THIRDPARTY_MODS_DIR=... cargo test --release -p orb-ui --test a11y_mod_chooser the_collection -- --nocapture. Decisions beyond the user's: Enter in the filter also chooses the selected item (the first match after typing); Page Up/Down move 10 items; the focus returns to the stop that had it after Mod Info closes (Chooser::list_stop); the generic window_focus change: when Mod Info opened from the chooser closes, the focus goes back into the chooser. The baseline tests (a11y_baseline, a11y_sections) needed no change: the chooser adds nodes only while open and the menu item only inside the open Mods popup. cargo test --release -p orb-ui --no-fail-fast: all pass but crash.rs a_deliberate_panic_exits_101_and_writes_the_log (the known TASK-200). clippy -D warnings and nightly fmt --check clean. ORB_CD_ISO is not set here, so the CD-gated tests of menu.rs ran their skip paths (the one modified literal there, ModVariant::demo, compiles). Not confirmed here: how NVDA speaks the tree (by ear, TASK-197): the roles and names are asserted from the AccessKit tree only. AC 3 was narrowed to what tests can show; the by-ear check is NVDA checklist item 12 in docs/accessibility_plan.md and goes to TASK-197 (the supervisor adds it).

Review round 1 (3 defects, fixed): (1) Enter in the filter and Ctrl+I acted on a cursor the filter hides; now they act only when the cursor is among the visible rows (new test nothing_acts_on_a_hidden_item: 'zzz', Enter and Ctrl+I change no kit and open no Mod Info). (2) The test now asserts the focus after Escape closes Mod Info is the stop that had it (filter TextInput; list Tree), exercising the window_focus block. (3) The test asserts only the cursor's TreeItem has selected == Some(true). 12 tests in a11y_mod_chooser.rs pass; clippy and fmt clean; orb-ui suite as before.

Merge (supervisor, 2026-11-07): review round 1 sent back three defects (Enter / Ctrl+I on a hidden item with an empty filter; the focus return after Mod Info untested; the TreeItem selected state untested), fixed in the amended commit; the reviewer's request to change the Co-Authored-By line was dismissed (the session's attribution line). Round 2: merge. Fast-forwarded onto main; the by-ear checks went to TASK-197 AC #9.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Mods > Choose Mod... (first item in screen reader mode) and Ctrl/Cmd+M open a dialog (role Dialog, 'Choose Mod') with a filter field and the Mods menu's list as a tree, built from the cached list: Standard ReBirth first, entries in the menu's order, texts 'name, variants: N, has demo, current'; entries with variants are collapsed TreeItems (Right/Left expand and collapse, files labeled as the submenu with 'preferred'); the filter and the list are two focus stops (Tab, Down from the filter); Enter chooses with the menu's commands (Cmd::UseKit, Cmd::ChooseVariant); Ctrl/Cmd+I opens Mod Info for the selection without loading it (App::show_mod_info_of; Mods > Info unchanged); Escape closes from either stop. The demo flag is read with the list (orb_cli::mods::scan_with_demo). With the mode off nothing changes. Key help, docs/ui_spec.md 20.7 and the A11Y docs updated. Opening with the 311 entries of rb-mods costs at most 4.9 ms in release (a steady frame 1.6 ms). Branch task-mod-chooser, one commit. Tests: 11 new in a11y_mod_chooser.rs, orb-ui suite passes but the known crash.rs failure (TASK-200); clippy and nightly fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
