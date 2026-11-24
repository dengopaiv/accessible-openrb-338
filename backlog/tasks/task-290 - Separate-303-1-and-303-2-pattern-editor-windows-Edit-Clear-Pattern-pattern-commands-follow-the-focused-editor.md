---
id: TASK-290
title: >-
  Separate 303-1 and 303-2 pattern editor windows; Edit > Clear Pattern; pattern
  commands follow the focused editor
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-24T12:27:00Z'
updated_date: '2026-11-24T12:27:00Z'
labels:
  - ui
  - player
milestone: m-9
dependencies: []
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/src/editor_frame.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/shortcuts.rs
  - crates/orb-plugin/src/clip.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 265700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-24 for three changes in one task.

**1. Two 303 editor windows.** Today one window, "303 Pattern Editor" (crates/orb-ui/src/pattern_editor.rs around 624, task-274), edits either 303 and switches between them with its own 303-1 / 303-2 buttons (pattern_editor.rs around 377).
- Make it two independent windows, titled "303-1 Pattern Editor" and "303-2 Pattern Editor", each fixed to its 303. Either or both can be open, as the 808 and 909 editors are (crates/orb-ui/src/drum_editor.rs, editor_frame.rs).
- The in-window 303 selector goes.
- The Edit menu gets "303-1 Pattern Editor..." and "303-2 Pattern Editor..." in place of "303 Pattern Editor...".
- Cmd/Ctrl+E opens the focused row's editor, as for the 808 and 909.
- Each window keeps its own position in ui.toml (today pattern_editor_pos; e.g. pattern_editor_303_1_pos and _2_pos). The old key seeds 303-1's position once, so a saved position is not lost.
- Bringing an already open window to the front (task-283) works for both.
- The plugin gets the same windows and menu items; it takes no keys (task-201).

**2. Edit > Clear Pattern**, with no keyboard shortcut. It empties the target pattern's steps the way the editors' Clear button does (emptied_pattern: the shuffle and length bytes stay), as one write_pattern with Undo.

**3. Which pattern the Edit menu's pattern commands act on.** Copy, Cut, Paste, Duplicate and Clear Pattern act on:
- the pattern selected in the pattern editor window that has the focus, when one does: the editor's own selected pattern and its device;
- otherwise, the selected pattern of the focused row in the main panel, as today (`App::pattern_target`, crates/orb-ui/src/menu.rs around 1054).
Define "has the focus" in the notes: egui's focused or topmost window, a click into it, or similar, whatever is reliable in both the app and the plugin. Paste Pattern follows the same rule (the user, 2026-11-24): it pastes into the focused editor's selected pattern, and the device check is against that editor's device.

**4. Survey the names of the two 303s** in every user-visible place: window titles, menus, status messages, tooltips, the Keyboard Shortcuts help, the plugin's MIDI clip sections and drag-out file names, the docs (docs/ui_spec.md, docs/plugin.md, the site and README), and the CLI. The supervisor's quick grep on 2026-11-24 found three forms:
- "303 1" / "303 2": the status line row names (menu.rs ROW_NAMES);
- "303-1" / "303-2": the 303 editor's selector buttons and the plugin's drag-out clip file names, e.g. "303-1 A1.mid" (crates/orb-plugin/src/clip.rs);
- "303 #1" / "303 #2": the plugin's MIDI clip sections (clip.rs SECTIONS) and the Keyboard Shortcuts help's Select Patterns rows (key_help.rs around 497).
List every occurrence with file:line in the notes, as a table by form. Do not unify them in this task. The new window titles use "303-1" / "303-2" as the user named them. Report the table to the user, who decides on one form; that becomes a follow-up task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 "303-1 Pattern Editor" and "303-2 Pattern Editor" are separate windows, each fixed to its 303, both can be open, with Edit menu items, Cmd/Ctrl+E for the focused row, positions kept per window (the old key seeds 303-1), in the app and the plugin
- [ ] #2 Edit > Clear Pattern (no shortcut) empties the target pattern's steps with Undo, keeping shuffle and length
- [ ] #3 The notes hold a table of every user-visible 303-1 / 303 1 / 303 #1 form (and any other) with file:line, and the report gives it to the user; nothing is unified yet
- [ ] #4 docs/ui_spec.md describes the two windows, Clear Pattern and the target rule; the Keyboard Shortcuts help stays consistent with the user's cuts (task-284)
- [ ] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [ ] #6 The user has checked it in the app and the plugin
- [ ] #7 Copy, Cut, Paste, Duplicate and Clear Pattern act on the focused pattern editor's selected pattern, else on the main panel's focused row's selected pattern; tests cover both cases for a 303, the 808 and the 909
<!-- AC:END -->
