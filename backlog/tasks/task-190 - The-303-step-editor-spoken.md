---
id: TASK-190
title: The 303 step editor, spoken
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-188
  - TASK-189
references:
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/keys.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 213000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-6; the analysis is docs/accessibility.md. Read both before starting.

- What: in each 303 section, a "Step editor" node (focusable) whose value is the step being
  edited ("Step 5 of 16: C sharp, up, accent, slide", "Step 6: rest") and the editor's mode
  (pitch or time mode as the original has them). After each programming key, the step's new
  state is spoken (the node's value changes; if NVDA does not speak value changes of the
  focused node reliably, the announcer). Left and Right move the edit step where the original
  allows (`move_edit`), Home and End to the first and last step. F2 on a 303 section reads the
  whole pattern, one step after another, compactly ("1 C accent, 2 rest, 3 D sharp slide,
  ..."), in the pattern's length.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: program a pattern with the keys alone, read it back with F2, and compare with the pattern written by the mouse in the same steps: equal events and equal text.
- [ ] #2 The note names follow the original's keyboard (C to high C, with "sharp"), and up/down are spoken as octave up/down.
- [ ] #3 A short how-to in docs/ui_spec.md: programming a 303 pattern with a screen reader.
<!-- AC:END -->
