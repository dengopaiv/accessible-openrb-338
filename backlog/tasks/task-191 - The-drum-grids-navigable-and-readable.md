---
id: TASK-191
title: The drum grids, navigable and readable
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
  - crates/orb-ui/src/behavior.rs
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 214000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-7; the analysis is docs/accessibility.md. Read both before starting.

- What: in each drum section, a Grid node with 12 rows (the instruments, AC first, named as
  the panel's legend: "Accent", "Bass drum", ...; the 808's switchable slots by the voice they
  play now) and 16 columns (the steps, up to the pattern's length). Up and Down choose the
  instrument (`select_instrument`, so the panel's selection follows), Left and Right the step,
  Space sets the step as a click does (D2; 808: on/off; 909: the click cycle, with Ctrl, Alt and
  Ctrl+Alt as the mouse's modifiers, and the flam button's effect). Each cell reads "Bass drum,
  step 5, on" ("accent", "flam" on the 909). F2 on a drum section reads each instrument that
  has steps, with its steps ("Bass drum: 1, 5, 9, 13. Closed hi-hat: 3, 7, 11, 15").
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: setting steps through the grid gives the same events as clicks on the same steps, on both machines and with each 909 modifier.
- [ ] #2 Tests: F2 reads a known pattern correctly; steps past the pattern length are not offered.
- [ ] #3 The selection frame on the panel follows the grid's row (visible for sighted helpers).
<!-- AC:END -->
