---
id: TASK-208
title: 'Decide: Alt+Space on a TR-909 grid opens the system menu'
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-06T08:41:00Z'
updated_date: '2026-11-06T21:50:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - docs/ui_spec.md
  - crates/orb-ui/src/a11y/grid.rs
priority: low
type: spike
ordinal: 227000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Found in TASK-191 (the drum grids, docs/ui_spec.md 20.6). In screen reader mode, Space on a focused drum grid sets the step as a mouse down does, and on the TR-909 the mouse's modifiers are on Space too: Ctrl+Space (a note on/off), Alt+Space (an accent on/off), Ctrl+Alt+Space (a flam on/off), as the task asked.

The conflict: on Windows, Alt+Space opens the window's system menu (Restore, Move, Size, Minimize, Maximize, Close). winit 0.30.13 hands every WM_SYSKEYDOWN to DefWindowProc (winit src/platform_impl/windows/event_loop.rs line 1767), so the app gets the key and sets the accent, and then Windows opens the system menu over it; NVDA speaks the menu and the user has to press Escape. Ctrl+Alt+Space is a WM_KEYDOWN and is not affected. Not checked by hand (no keystrokes sent to the window in TASK-191); it follows from the code and Windows' standard behavior.

An accent can always be reached without Alt: plain Space cycles a 909 step off, note, accent, off (the mouse's cycle), and the accent row is on/off.

Options:
A. Keep Alt+Space as it is (the mouse's modifier, as the task says) and document that the system menu also opens; Escape closes it.
B. Add Shift+Space on a 909 grid as the accent key (the mouse's Alt), keep Alt+Space too.
C. Add Shift+Space as the accent key and swallow Alt+Space on a focused grid so the system menu does not open (needs a winit hook or a message filter; egui/eframe offers none directly, so this is more work and touches the window's message handling).
D. Replace Alt+Space by Shift+Space (Alt+Space then opens only the system menu, as in any Windows program).

Recommendation: B. Shift+Space is free in the grid (TASK-191 leaves it to the key map, where it does nothing on a grid), costs a few lines in crates/orb-ui/src/a11y/grid.rs (`GridKey::of`), and leaves Alt+Space working for whoever expects the mouse's modifier.

Context: crates/orb-ui/src/a11y/grid.rs (GridKey::of), crates/orb-ui/src/app.rs (App::grid_key), docs/ui_spec.md 20.6, Help > Keyboard Shortcuts (crates/orb-ui/src/key_help.rs).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The user chose an option (A, B, C or D); the outcome and the date are in the final summary
<!-- AC:END -->
