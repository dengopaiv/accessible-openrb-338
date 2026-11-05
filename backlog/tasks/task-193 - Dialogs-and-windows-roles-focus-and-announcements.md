---
id: TASK-193
title: 'Dialogs and windows: roles, focus and announcements'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-189
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/error_box.rs
  - crates/orb-ui/src/info.rs
  - crates/orb-ui/src/about.rs
  - crates/orb-ui/src/key_help.rs
  - crates/orb-ui/src/mod_info.rs
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 216000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-8; the analysis is docs/accessibility.md. Read both before starting.

- What: each window gets a Dialog role and its title as name (the error box: AlertDialog);
  when it opens, the focus moves into it (the prompt: to Save; the error box: to OK; Info: to
  the first field; About, Keyboard Shortcuts, Mod Info: to the window's text or its close
  button), and when it closes the focus returns where it was. The error box's message is read
  when it opens. The texts of About, Keyboard Shortcuts and Mod Info can be read line by line
  (labels in a scrollable region, reachable with Tab or the arrows).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: for each window, the focused node after it opens and after it closes.
- [ ] #2 With NVDA, each window is announced with its title and content when it opens (by hand; notes).
- [ ] #3 With the mode off, the windows behave as on main (their tests pass unchanged).
<!-- AC:END -->
