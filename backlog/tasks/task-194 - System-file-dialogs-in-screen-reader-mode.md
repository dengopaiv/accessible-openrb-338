---
id: TASK-194
title: System file dialogs in screen reader mode
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-180
references:
  - crates/orb-ui/src/browser.rs
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/Cargo.toml
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 217000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-9; the analysis is docs/accessibility.md. Read both before starting.

- What (D4's recommendation): in screen reader mode, `open_browser` opens the system dialog for
  the mode instead (filters: songs `.rbs`, mods `.rbm`, export WAV/FLAC/Ogg as the browser
  offers), starting in the folder the browser would start in, and passes the result to the same
  `chosen` path. Options menu items "Set Songs Folder..." and "Set Mods Folder..." with a folder
  dialog, doing what the browser's "Use as ... folder" buttons do. The dialog blocks the frame
  loop while open; the audio thread keeps playing (check, and note it).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In screen reader mode each of the four menu commands opens the system dialog, and a chosen file opens, saves, exports or loads exactly as through the browser (by hand on Windows; the shared `chosen` path is tested headless).
- [ ] #2 The folder items change `ui.toml` as the browser's buttons do (test).
- [ ] #3 With the mode off, the in-app browser opens as before.
<!-- AC:END -->
