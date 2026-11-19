---
id: TASK-251
title: >-
  Decide: the main panel's gain-reduction LEDs as the original's bar from the
  left
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-19T11:59:00Z'
labels:
  - decision
  - ui
milestone: m-5
dependencies: []
references:
  - docs/ui_spec.md
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/theme_view.rs
priority: low
type: spike
ordinal: 243000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-187's like-for-like with the original on the test machine (2026-11-19, built-in kit, lossless screenshots) showed that the original's compressor meter (the nine LEDs under COMP, bitmap 12570) is a bar from the left: idle with the compressor on the master it lights LEDs 1 to 5 (up to "0"; the lit row is three green, two yellow, one yellow, three red, by place); with the threshold at its lowest while playing, all nine; with the master's COMP switch off, none. The engine byte behind it is the compressor's live byte 3, trunc(min(G*18, 127)) (0x1000EB48): 18 at unity gain, 0 at gain 0, 127 at the top. OpenRB-338's main panel (task-179, docs/ui_spec.md sections 15 and 16; crates/orb-ui/src/link.rs, the gr_leds block after the meters; crates/orb-ui/src/model.rs, the "meter" part with GR_OFF) lights the center LED green and the "reduction" amber then red to its LEFT, which the original never shows. The theme viewer (task-187) already draws the original's bar (theme_view::gr_lit). The main panel is the redesigned art, which need not match the original exactly, so how it shows the meter is the user's call.

Options:
A. The main panel follows the original: a bar from the left, lit by place in the colors the redesign gives each place (green, green, green, amber, amber, amber, red, red, red), none at a byte of 0, five at 18, nine at 127. The link would keep the raw byte (a new Panel field) instead of the 0..4 count, and the viewer would map the same byte.
B. Keep the redesign's center-out display, and only fix its direction and scale (the bar grows to the right of center as the byte grows above 18).
C. Keep it as it is.

Recommendation: A. It is what the meter means in the original (and in every mod's bitmap, whose lit row is colored by place), the mockup of task-179 already lit LEDs 1 to 5 by place, and one byte-to-LEDs mapping would serve the panel and the viewer. The scale between the three observed points (0, 18, 127) is a guess until sub_424A7E is read in IDA; that read belongs to the work if A is chosen.

Evidence: task-187's notes (the like-for-like section), docs/ui_spec.md section 21 ("The like-for-like with the built-in kit").
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The user chose an option (A, B or C) and the outcome and date are in the final summary
- [ ] #2 If A or B: a follow-up task for the change exists, with the IDA read of sub_424A7E for the scale
<!-- AC:END -->
