---
id: TASK-184
title: The menu bar from the keyboard
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:31:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-183
references:
  - crates/orb-ui/src/menu.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: task
ordinal: 207000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-11; the analysis is docs/accessibility.md. Read both before starting.

- What: find out how egui 0.37's menu bar is reached and walked from the keyboard (today the
  four menu titles are focusable buttons; whether Alt or F10 focuses the bar, whether the arrow
  keys open and walk a menu, whether Escape returns). Make F10 (and Alt pressed and released
  alone, on Windows) focus the File menu, the arrows walk the bar and the items, Enter choose,
  Escape close, in both modes if harmless, else in screen reader mode. Checkboxes and radio
  items in the Options menu read their state.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: F10 focuses File; Right moves to Mods; Down opens it and focuses its first item; Escape closes; Enter on an item runs it.
- [ ] #2 The findings about egui's own behavior are in the notes and in docs/ui_spec.md section 18.
<!-- AC:END -->
