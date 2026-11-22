---
id: TASK-281
title: >-
  909 pattern editor: low/high/flam cells with an entry switch, on the 808
  editor's frame
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-22T09:49:00Z'
labels:
  - ui
milestone: m-9
dependencies:
  - TASK-280
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/controller.rs
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
  - docs/RBS42.txt
  - docs/initial_investigation.md
priority: medium
type: feature
ordinal: 260200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** milestone m-9's third editor, the TR-909's. It is built on the drum-grid frame of the 808 editor (task-280). The user designed it on 2026-11-22.

**The data** (docs/RBS42.txt, the "909 " chunk; docs/ui_spec.md, the 909 step buttons):
- A pattern is a shuffle byte, a length 1..16, and 16 steps × 12 bytes: AC, BD, SD, LT, MT, HT, RS, CP, CH, OH, CC, RC.
- An instrument byte: 0 off, 1 low, 2 high, 3 flam (a low hit plus the flam retrigger).
- The AC row is on/off, and the stored "on" is 2, as the panel writes it.
- A byte above 3 (a song may hold any): the editor draws it as flam, as the original and the panel do, and keeps the byte unless the user edits that cell. The editor never writes values above 3.
- Colors as on the panel: low orange, high red, flam green.

**Layout:** the 808 editor's frame.
- 11 drum rows (BD SD LT MT HT RS CP CH OH CC RC), then a separator and the AC row (tooltip "Accent: every drum on this step louder").
- Bank, pattern number with arrows, Length, Shuffle, Shift, Copy/Paste/Clear, and the one-level Undo, all as in the 808 editor.
- Its own window, separate from the 808's (the user's decision).

**The entry switch: Cycle | Low | High | Flam**, in the header. Cycle is the default.
- **Cycle:** a click on a drum cell goes off → low → high → off, like the panel's plain click.
- **Low / High / Flam:** a click sets that value. Clicking a cell that already holds that value clears it, and a cell holding another value is replaced in one click. This is the original panel's modifier rule.
- **Modifiers** switch to Low / High / Flam while held, and the switch returns to its own setting on release; the switch shows the held state:
  - macOS: Cmd = Low, Option = High, Cmd+Option = Flam;
  - Windows and Linux: Ctrl = Low, Alt = High, Ctrl+Alt = Flam.
  These are consistent with the main panel, which this task changes the same way (below).
- **AC row:** a click toggles it (0 ↔ 2), whatever the switch says.
- **Drag:** paints the first cell's new value along the same row only.
- **CH/OH:** entering a value on CH clears OH on that step, and vice versa, as on the original panel (docs/ui_spec.md: `sub_447153(9 or 8, step, 0)`). A song that already holds both is shown as is.
- Every edit is one whole-pattern write through `App::write_pattern` and undoable.

**The main panel's 909 modifiers (the user's decision: consistent across the app):**
- Today the panel reads the physical Ctrl and Alt on every platform (crates/orb-ui/src/controller.rs around line 380, `Mods { ctrl, alt }` → `model::step909_new_value` → `behavior::tr909_step_value`).
- Change macOS to Cmd for Ctrl and Option for Alt. Windows and Linux keep Ctrl and Alt.
- Update the panel's docs (docs/ui_spec.md, the 909 step buttons) and its tests.

**Opening:**
- Edit menu "909 Pattern Editor...".
- Cmd/Ctrl+E with the 909 row focused opens it, as task-280 set up for the other rows.
- Works in the plugin too.

**Keyboard shortcuts help (the user's requirement):** Help > Keyboard Shortcuts must list every key and modifier the 909 editor supports, including the entry-switch modifiers with their per-platform names, and the panel's changed 909 modifiers. Its tests cover them.

**Docs:** docs/ui_spec.md, the editor section, the 909 step buttons and the Edit menu.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Edit > 909 Pattern Editor... and Cmd/Ctrl+E with the 909 row focused open a separate floating 909 editor (standalone and plugin) on task-280's drum-grid frame: 11 drum rows, the AC row, bank, pattern number, Length, Shuffle, Shift, Copy/Paste/Clear, Undo
- [ ] #2 The entry switch Cycle | Low | High | Flam (default Cycle): Cycle cycles off-low-high-off; Low/High/Flam set the value, clear a cell holding the same value and replace another value in one click; modifiers (Cmd/Option on macOS, Ctrl/Alt elsewhere; both for Flam) select the switch while held and the switch shows it
- [ ] #3 Cells draw low orange, high red, flam green, values above 3 as flam and kept unless edited; the AC row toggles 0/2; drag paints along the row; entering CH clears OH on that step and vice versa; every edit is one undoable pattern write
- [ ] #4 The main panel's 909 step modifiers use Cmd/Option on macOS (Ctrl/Alt elsewhere), with docs and tests updated
- [ ] #5 Help > Keyboard Shortcuts lists every key and modifier of the 909 editor and the panel's 909 modifiers with per-platform names, and its tests cover them; docs/ui_spec.md updated; cargo test --workspace passes, clippy -D warnings and nightly fmt clean
- [ ] #6 The user reviewed the dev app and the plugin before the agent review
<!-- AC:END -->
