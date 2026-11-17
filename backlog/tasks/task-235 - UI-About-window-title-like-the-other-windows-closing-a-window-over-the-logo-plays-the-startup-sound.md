---
id: TASK-235
title: >-
  UI: About window title like the other windows; closing a window over the logo
  plays the startup sound
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T11:09:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src
  - crates/orb-plugin/src/editor.rs
  - task-221
priority: medium
type: bug
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Two small UI bugs the user found in the plugin (2026-11-17); check the standalone app too, since the windows come from orb-ui.

1. The About window's title bar looks different from the other windows (File > Song Info, Mods > Info, Help > Plugin Usage, task-221). Make About's title bar match theirs (style, font, close button, layout), in the plugin and the standalone.

2. Closing a window whose X (close) button lies over the panel's logo also clicks the logo: the click reaches the panel under the window, and the mod's startup sound plays. A click on a window's close button (and, check, anywhere in a window's frame or body) must never reach the panel underneath. Fix it once for all orb-ui windows, not only About, and look for the same click-through on other panel controls under a window.

Help > Plugin Usage is on task-221's branch, not yet on main; match the windows on main and, once task-221 lands, the supervisor rebases. Visual change: the user checks it in the plugin before merge.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The About window's title bar matches the other windows (side-by-side screenshots of About and Song Info, plugin and standalone).
- [ ] #2 Closing any orb-ui window with its close button over the logo (or another panel control) does not activate the control underneath (test with a headless egui click at the close button's position over the logo).
- [ ] #3 The standalone's --screenshot changes only if About is in it; say which in the notes.
- [ ] #4 cargo test, clippy -D warnings and nightly fmt pass; the user's check in the plugin.
<!-- AC:END -->
