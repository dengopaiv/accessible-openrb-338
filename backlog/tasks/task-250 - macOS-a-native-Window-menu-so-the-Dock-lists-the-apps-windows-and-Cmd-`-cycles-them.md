---
id: TASK-250
title: >-
  macOS: a native Window menu, so the Dock lists the app's windows and Cmd-`
  cycles them
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-19T10:30:00Z'
labels:
  - ui
milestone: m-5
dependencies:
  - TASK-187
references:
  - crates/orb-ui/src/main.rs
  - crates/orb-ui/src/theme_view.rs
  - docs/ui_spec.md
  - task-187
priority: medium
type: enhancement
ordinal: 203500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user (2026-11-19), testing task-187's Theme Viewer (a second native window) on macOS: right-clicking the Dock icon shows only "Show All Windows"; the app's windows (the main window and the viewer) are not listed. AppKit fills the Dock menu's window list from the app's Windows menu (`NSApp.windowsMenu`); OpenRB-338's menus are drawn by egui inside the window, and the native menu bar winit sets up has no Window menu, so the list is empty. Cmd-` (Cycle Through Windows) most likely does not work either, for the same reason (verify).

What to do (standalone app only, macOS only; the plugin is untouched, the host owns its menu bar):
- At startup, after winit/eframe has built its default main menu, add a native "Window" menu to the menu bar with the standard items (Minimize Cmd-M, Zoom, a separator, Cycle Through Windows Cmd-`, Bring All to Front) and set it as `NSApp.setWindowsMenu`, using the objc2 / objc2-app-kit crates winit already depends on (same versions; no new heavy dependency). AppKit then lists every titled window in it and in the Dock menu.
- Check that the egui menus and shortcuts inside the window still work (no shortcut stolen: Cmd-M and Cmd-` are new, check they do not clash with orb's shortcuts in docs/ui_spec.md; if one clashes, report it rather than changing orb's).
- Check eframe does not replace the main menu later (e.g. on a new viewport) and drop the Window menu.
- docs/ui_spec.md says the macOS menu bar has a Window menu and why.

The user checks the dev app (`scripts/macos-dev-app.sh`) before the agent review. Depends on task-187 (the viewer is the second window; build on main after task-187 merges, or rebase onto it).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On macOS the menu bar has a Window menu (Minimize, Zoom, Cycle Through Windows, Bring All to Front) registered as NSApp's windows menu
- [ ] #2 Right-clicking the Dock icon lists the main window and, when open, the Theme Viewer by title, and picking one brings it to the front (checked by the user)
- [ ] #3 Cmd-` cycles between the main window and the viewer
- [ ] #4 No orb shortcut changes or is shadowed; the egui menus work as before; nothing changes on Windows, Linux or in the plugin
- [ ] #5 cargo test, clippy -D warnings (also for a Linux/Windows target where possible) and nightly fmt pass; docs/ui_spec.md updated
<!-- AC:END -->
