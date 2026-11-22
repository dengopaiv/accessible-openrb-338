---
id: TASK-280
title: '808 pattern editor: a floating window with drum rows and the accent row'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-22T09:48:00Z'
updated_date: '2026-11-22T09:49:00Z'
labels:
  - ui
milestone: m-9
dependencies: []
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/tests/pattern_editor.rs
  - crates/orb-plugin/tests/editor.rs
  - crates/orb-ui/src/key_help.rs
  - crates/orb-ui/src/menu.rs
  - docs/ui_spec.md
  - docs/RBS42.txt
priority: medium
type: feature
ordinal: 260100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** milestone m-9 adds pattern editors for the 303, 808 and 909. The 303 editor is done (task-274, crates/orb-ui/src/pattern_editor.rs). This task adds the TR-808's, in the same frame and with the same conventions. The user designed it on 2026-11-22.

**The data** (docs/RBS42.txt, the "808 " chunk; docs/ui_spec.md, the 808 section):
- A pattern is a shuffle byte, a length 1..16, and 16 steps × 12 bytes.
- The 12 bytes, in RBS column order: AC, BD, SD, LT/LC, MT/MC, HT/HC, RS/CL, CP/MA, CB, CY, OH, CH.
- Each byte is 0 or 1: there is no per-drum level on the 808. AC is the per-step accent.
- The LT/LC, MT/MC, HT/HC, RS/CL and CP/MA switches are song controllers (kit settings for every pattern), not pattern data. **The editor has no such switches** (the user's decision). The row labels show the names the panel's switches currently select, and update when the panel changes.
- Only what RBS stores is edited. No extra data.

**Layout:** like the 303 editor, a floating in-app egui window, non-resizable.
- Header: bank buttons, the pattern number field with left/right arrows (across banks; the 808 has no 1/2 switch), Length, Shuffle, and Copy / Paste / Clear.
- 11 drum rows in place of the piano roll, labeled on the left in the 303 editor's label column style and size.
- A separator, then the AC row (labeled "AC", tooltip "Accent: every drum on this step louder"), like the 303's flag rows.
- The shift arrows under the grid, centered. No transpose.
- The grid fills the width, and the step and current-step indication works as in the 303 editor.

**Editing:**
- A click toggles the cell (0 ↔ 1).
- A drag paints the first cell's new value along the same row only, as on the panel.
- Shift wraps within the length.
- Clear: like Cut but keeps the length and shuffle, as in the 303 editor.
- Copy and Paste go through the existing pattern clipboard.
- Every edit is one whole-pattern write through `App::write_pattern` (→ `Link::set_pattern`) and is undoable with the one-level Undo, as in the 303 editor.

**Selection:**
- In pattern mode the editor and the panel select each other's pattern.
- In song mode they are independent.
- There is no Follow button.

**Opening:**
- Edit menu "808 Pattern Editor..." (with "303 Pattern Editor..." and "909 Pattern Editor...", task-281).
- Cmd/Ctrl+E opens the editor for the panel's focused row: 303, 808, or 909 once task-281 is done. It focuses the window if it is already open and switches it to the active pattern.
- Works in the plugin too, as the 303 editor does.

**Keyboard shortcuts help (the user's requirement):** Help > Keyboard Shortcuts (crates/orb-ui/src/key_help.rs) must list every key and modifier the 808 editor supports, under its own group, including any keys and mouse modifiers it handles. Its bind tests must cover them, the way they cover the panel. A full audit of every context follows in task-282.

**Shared code:** the 909 editor (task-281) has the same frame. Factor the drum-grid editor so the 909 adds only its cell states and entry switch, rather than copying.

**Docs:** docs/ui_spec.md, the pattern editor section and the Edit menu.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Edit > 808 Pattern Editor... and Cmd/Ctrl+E with the 808 row focused open a floating, non-resizable 808 editor in the standalone app and the plugin; it focuses and switches to the active pattern when already open
- [ ] #2 11 drum rows (labels follow the panel's current LT/LC, MT/MC, HT/HC, RS/CL, CP/MA switch positions; no switches in the editor) plus the AC row below a separator; bank, pattern number with arrows, Length, Shuffle, Shift (wraps), Copy/Paste/Clear as in the 303 editor
- [ ] #3 Click toggles a cell; drag paints the first cell's new value along its row only; every edit is one pattern write and undoable; pattern-mode selection is linked with the panel, song mode independent
- [ ] #4 Help > Keyboard Shortcuts lists every key and mouse modifier the 808 editor supports, and its tests cover them
- [ ] #5 The drum-grid editor is shared code ready for the 909 (task-281); tests cover the layout, the edits, undo, the selection link and the plugin's Edit menu; docs/ui_spec.md updated; cargo test --workspace passes, clippy -D warnings and nightly fmt clean
- [ ] #6 The user reviewed the dev app and the plugin before the agent review
<!-- AC:END -->
