---
id: TASK-283
title: Opening an already open window brings it to the front
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-22T16:27:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/info.rs
  - crates/orb-ui/src/mod_info.rs
  - crates/orb-ui/src/about.rs
  - crates/orb-ui/src/key_help.rs
  - crates/orb-ui/src/plugin_usage.rs
  - crates/orb-ui/src/editor_frame.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 261700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-22. Choosing a menu item or shortcut for a window that is already open does nothing visible today when the window is behind another one: e.g. File > Info with Song Information already open should bring Song Information to the front and focus it. The same goes for every window the app opens.

**The windows** (all egui::Window inside the one app viewport, so "focus" means raising the window's layer to the top, e.g. egui's Context::move_to_top on the window's LayerId, and giving it keyboard focus where it has a focusable field):
- Song Information (crates/orb-ui/src/info.rs, File > Info)
- Mod Information (crates/orb-ui/src/mod_info.rs)
- About (crates/orb-ui/src/about.rs)
- Keyboard Shortcuts (crates/orb-ui/src/key_help.rs)
- Plugin Usage (crates/orb-ui/src/plugin_usage.rs)
- the 303, 808 and 909 pattern editors (crates/orb-ui/src/editor_frame.rs; opened from the Edit menu and Cmd/Ctrl+E)
- any other window found in crates/orb-ui and crates/orb-plugin (search for egui::Window); list them all in the notes.

Opening a window that is not open keeps today's behavior (its saved position, if any). Raising an open window does not move it, resize it or reset its state (e.g. a typed value in a pattern editor stays).

Works in the standalone app and the plugin (the plugin opens windows from the menus only; it takes no keys, task-201).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every window opened from a menu or shortcut, listed in the notes, comes to the front when its command is chosen while it is already open, including when another window covers it
- [ ] #2 Raising an open window keeps its position, size and contents
- [ ] #3 A test per window (or one table-driven test) opens it, opens another window over it, repeats the first window's command and checks the first window's layer is on top
- [ ] #4 docs/ui_spec.md states the rule once, in the section that describes the app's windows
- [ ] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [ ] #6 The user has checked it in the app
<!-- AC:END -->
