---
id: TASK-274
title: >-
  303 pattern editor window: piano roll, Up/Down/Accent/Slide rows, bank and
  pattern selection
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T17:52:00Z'
labels:
  - ui
milestone: m-9
dependencies: []
references:
  - docs/ui_spec.md
  - docs/RBS42.txt
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/shortcuts.rs
  - crates/orb-ui/src/controller.rs
  - crates/orb-ui/src/app.rs
priority: medium
type: feature
ordinal: 259500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Designed with the user on 2026-11-21. It's a floating editor for one 303 pattern: a step grid with a 13-key piano roll, inspired by modern 303 pattern editors. It edits only what RBS 4.2 stores for a 303 pattern (docs/RBS42.txt, the "303 " chunk); no new song data. The 808 and 909 editors are later tasks in m-9.

**Layout** (the user's wireframe, top to bottom):
- **Top row,** all standard egui controls:
  - a 303-1 / 303-2 switch;
  - bank buttons A B C D;
  - the pattern number 1-8 as a `DragValue`: drag up or down, or click to type;
  - Copy, Paste and Clear;
  - Length 1-16, also a `DragValue`;
  - a Shuffle toggle: the pattern's own shuffle on/off byte, not the song's shuffle amount.
- **Note grid:**
  - a vertical 13-key piano roll on the left, C to the C an octave above. Pitch 0x0c is real data, so the 13th key is needed.
  - 16 step columns. Black-key rows look different from white-key rows. Colors come from the app's palette, not the skin, so the editor looks the same in Classic and Modern.
  - The step that's playing is shown.
  - Transpose up/down buttons at the grid's right.
  - Shift left/right buttons under the grid.
- **Flag grid:** four rows, Up, Down, Accent and Slide, over the same 16 columns.

**Behavior:**
- **Notes:**
  - A click in a step's column at a pitch makes the step a note at that pitch.
  - A click on the step's lit cell makes it a pause.
  - Pauses are not shown: the column is empty. A pause's stored pitch is kept, unchanged and unshown.
- **Flags:**
  - Up, Down, Accent and Slide toggle per step.
  - Up and Down are independent, as in the file; both set cancel each other (manual page 45, docs/ui_spec.md).
- **Drag painting:**
  - In the piano roll, a drag paints wherever the mouse points: each step it crosses becomes a note at the pitch under the mouse.
  - In the flag rows, a drag paints only in the row it started in, with the value set by the first cell.
- **Steps beyond the length** are dimmed but still editable.
- **Shift < >** rotates the steps, wrapping within the pattern length.
- **Transpose ^ v** moves every step's pitch by one semitone, wrapping over the 13 rows (12 up goes to 0, 0 down goes to 12). The Up and Down flags don't change.
- **Copy and Paste** use the same pattern clipboard as Edit > Copy/Paste Pattern (docs/ui_spec.md section 16), on the editor's pattern.
- **Clear** leaves the empty pattern that Edit > Cut Pattern leaves, but keeps the pattern's length and shuffle switch, and doesn't touch the clipboard.
- **Undo:** every editor change goes through the existing one-level undo (`menu::Undo`, Cmd+Z), and a drag counts as one change.
- **Live:** edits go through the same model and controller path as the panel's own pattern edits, with no second path, so they're heard at once. The song is marked modified, and the panel's step lights update.

**Pattern selection:**
- **Pattern mode:** choosing 303-1/2, a bank or a number in the editor selects that pattern on the panel too, so you hear it. Selecting a pattern on the panel changes the editor too.
- **Song mode:** the two are independent. The editor's selector only chooses what the editor shows; it never changes the panel's selection and never records. The panel's playing pattern doesn't move the editor.
- There is no Follow button: the user dropped it.

**The window:**
- It floats inside the main window (`egui::Window`): draggable and closable, not resizable. It's the same in the standalone and in the plugin, with no separate OS window.
- **Opening:**
  - from Edit > 303 Pattern Editor, and with Cmd+E (Ctrl+E) in the standalone; the plugin's editor has no keyboard shortcuts (task-201), so there it's the menu only.
  - It opens on the selected 303's active pattern. If it's already open, it comes to the front and switches to that pattern.
- Its position is saved in ui.toml in the standalone, and its open state isn't.

**Docs:** add a docs/ui_spec.md section (the window, the controls, the selection rules), and the Edit menu row and the Cmd+E shortcut in its menu and shortcut tables.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Edit > 303 Pattern Editor and Cmd+E (standalone) open the floating editor on the selected 303's active pattern; when it's open, they bring it to the front and switch it to that pattern; it works in the plugin from the menu
- [ ] #2 The top row has the 303-1/2 switch, banks A-D, a 1-8 DragValue, Copy, Paste, Clear, a Length 1-16 DragValue and a Shuffle toggle; the window is not resizable; its position is kept in ui.toml in the standalone
- [ ] #3 The note grid has a 13-key piano roll with distinct black and white rows in the app palette and shows the playing step; pauses are not shown, and a pause's stored pitch is kept
- [ ] #4 Clicks and drags edit as specified: the piano roll paints along the mouse; flag rows paint only within their row with the first cell's value; Up and Down are independent; steps beyond the length are dimmed and editable
- [ ] #5 Shift wraps within the length; Transpose wraps over the 13 rows and leaves the Up/Down flags; Copy and Paste share the Edit menu's pattern clipboard; Clear leaves the Cut Pattern result but keeps the length and shuffle
- [ ] #6 Each change (a drag is one) is undoable with the existing one-level Undo; edits are heard at once, mark the song modified and update the panel, through the existing edit path with no second one
- [ ] #7 Pattern mode: editor and panel selection follow each other both ways. Song mode: the editor's selection is independent and never changes the panel or records
- [ ] #8 Tests cover the step edits, drag painting, shift, transpose, clear, the undo of each and both selection rules; docs/ui_spec.md describes the editor and its menu item and shortcut
- [ ] #9 The user reviewed a dev app (and the plugin) before the agent review
<!-- AC:END -->
