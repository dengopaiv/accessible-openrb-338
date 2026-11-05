---
id: TASK-202
title: 'Decide: the panel''s keys while a menu has the focus'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T20:50:00Z'
updated_date: '2026-11-05T20:52:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
type: spike
ordinal: 222000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context (found in TASK-184): a menu walked from the keyboard collided with the panel's own keys. Up/Down moved the menu and the row focus at once; Enter on an item also pressed Play (keys.rs:188). This predates TASK-184 for menus opened by mouse.

Options:
1. Accept, in both modes: all panel keys wait while a menu is open or a menu title has the focus (menu_has_keys, also for a menu opened by mouse).
2. Only the menu's own keys (arrows, Enter, Escape) are withheld from the panel; other panel keys keep working.
3. Only in screen reader mode; with the mode off the app stays as upstream.

Recommendation: option 1 (simplest, no key reaches two things).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided 2026-11-05 by the user: option 1, accept in both modes: all panel keys wait while a menu is open or a menu title has the focus (menu_has_keys), also for a menu opened by mouse. Over 'only the menu's own keys' and 'only in screen reader mode'. Applied in TASK-184.
<!-- SECTION:FINAL_SUMMARY:END -->
