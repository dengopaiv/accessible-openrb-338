---
id: TASK-207
title: 'Decide: Enter in a section that is not a 303''s'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-06T07:38:00Z'
updated_date: '2026-11-06T07:38:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
priority: high
type: spike
ordinal: 226000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Raised in the review of TASK-188 (A11Y-4, screen reader mode). Decision A11Y-D2 (TASK-178) says Enter keeps the original's meaning: the 303's Step in Program Synth mode with a 303 row focused, else Play. In the original the row focus is the focus bar a click or Up/Down moves. In screen reader mode the row focus follows the section the navigation focus enters, and only a machine's section moves it. So after Shift+F6 from Bass Line 1 to the Transport (or Tab into PCF, Delay, Distortion, Master), the row focus is still Bass Line 1 and Enter steps that 303 although the focus is on Play.

Options:
- A. Enter follows the section of the focused control: in a 303's section the 303's Step (Program Synth mode); in the Transport, the effect sections and the drum machines' sections, Play. With no panel control focused, the row focus decides. The row focus still decides where the pitch and drum keys go (the last machine entered).
- B. Enter follows the row focus, as in the original: the 303's Step wherever the focus is, as long as a 303 row has the row focus.

Recommendation: A, so that Enter does what the focused place suggests; programming a 303 is unchanged, because its controls are in its section.

References: crates/orb-ui/src/app.rs (App::keyboard), docs/ui_spec.md section 10 (Screen reader mode), docs/accessibility.md 3.4.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user chose between following the section and following the row focus
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-06: A. In screen reader mode Enter follows the section of the focused control: a 303's section gives that 303's Step (Program Synth mode), any other section Play; with no panel control focused the row focus decides. The pitch and drum keys still go to the row focus. Implemented in TASK-188 (App::keyboard; docs/ui_spec.md section 10, Screen reader mode; Help > Keyboard Shortcuts). With the mode off Enter is unchanged.
<!-- SECTION:FINAL_SUMMARY:END -->
