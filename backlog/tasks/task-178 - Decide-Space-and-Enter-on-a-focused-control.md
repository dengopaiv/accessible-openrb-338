---
id: TASK-178
title: 'Decide: Space and Enter on a focused control'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:31:00Z'
updated_date: '2026-11-05T19:31:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/keys.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: spike
ordinal: 201000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-D2; the analysis is docs/accessibility.md. Read both before starting.

- Why: in Windows dialogs Space presses or toggles the focused control; in ReBirth, and in
  every music program, Space plays and stops.
- Facts found while deciding: no function key (F1 to F12) is used by orb-ui (no `Key::F..` in
  `crates/`) or by the original's key map (docs/ui_spec.md section 10 names none), so all twelve
  are free. Enter is the 303's Step key in Program Synth mode with a 303 focused, pressed after
  every programmed note, and Play everywhere else (`keys.rs:158` and 188).
- Options: A. Space stays Play/Stop everywhere; Enter presses or toggles the focused control.
  B. Space toggles the focused control as in dialogs; Play/Stop moves to another key. Asked
  again once the function keys were known to be free: Space and F5 both Play/Stop with Enter
  toggling; Space toggling with F5 Play/Stop; Space Play/Stop with F5 left free. And for Enter:
  keep the original's meaning; press focused buttons only; always press the focused control.
- Recommendation: A at first; then, for Enter, keep the original's meaning, so that
  programming a 303 works as in ReBirth.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
**Decided (2026-11-05): B, with F5 as Play/Stop; Enter keeps the original's meaning.** In
screen reader mode Space presses or toggles the focused control (a button, a toggle, a
switch, a step of a drum grid) and is no longer Play/Stop; F5 is Play/Stop. Enter does what
it does in the original whatever has the focus: the 303's Step in Program Synth mode with a
303 focused, else Play. With the mode off, Space is Play/Stop as before.
<!-- SECTION:FINAL_SUMMARY:END -->
