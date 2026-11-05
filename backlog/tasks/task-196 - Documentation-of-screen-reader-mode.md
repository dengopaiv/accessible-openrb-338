---
id: TASK-196
title: Documentation of screen reader mode
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
labels:
  - docs
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-186
  - TASK-187
  - TASK-188
  - TASK-189
  - TASK-190
  - TASK-191
  - TASK-193
  - TASK-194
  - TASK-195
  - TASK-184
  - TASK-192
references:
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: medium
type: docs
ordinal: 219000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-12; the analysis is docs/accessibility.md. Read both before starting.

- What: a README section "Using OpenRB-338 with a screen reader" (turning the mode on, the
  sections, the keys, programming a pattern, choosing a mod, known limits); docs/ui_spec.md's
  "Screen reader mode" section complete and consistent with the code; Help > Keyboard Shortcuts
  complete for the mode; docs/accessibility.md's status line set to what is implemented, with
  the measurements of section 1.1 taken again (`tools/uia_tree.ps1`) and recorded.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every key of the mode is in the README, the ui_spec and the key help, with the same meaning (a test compares the key help's table with the key map in code, if the key help has one).
- [ ] #2 The new measurement is in docs/accessibility.md next to the old one.
<!-- AC:END -->
