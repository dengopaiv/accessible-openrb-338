---
id: TASK-253
title: >-
  Meters: the original's log scale for the mixer, effect and master meters
  (sub_424220)
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-19T12:49:00Z'
labels:
  - ui
  - re
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/theme_view.rs
  - docs/ui_spec.md
  - task-252
priority: low
type: enhancement
ordinal: 244000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-252 read, on the way to the gain-reduction meter, how the original scales its other meters (CPeakMeterView: the four mixer channels, the master's two, the delay, dist, PCF and comp input meters), and OpenRB-338 does not follow it. orb's link::meter_state maps the engine's meter byte linearly, ceil(byte*10/127) states (docs/ui_spec.md section 16, [G]); the main panel and the theme viewer (theme_view::header_meter_frame) both use it.

What the original does (Rebirth.exe, read in task-252, docs/ui_spec.md section 2 'Meters'): each section's poll (sub_412DC7 master, sub_413F47 mixer channel, sub_407F7A delay, sub_408AF5 dist, sub_41B914 PCF, sub_404396 comp input) passes the byte through sub_424220, a table L[0] = 0, L[i] = __ftol(log(i) / log(127.0) * 127.0) (0x424258..0x424294), and stores it with sub_424A7E (view +0x22); the update sub_424A96 copies it to +0x20 and redraws; the draw sub_4249A8 shows frame sub_467F31(L, F - 1, 127) = trunc(L * (F - 1) / 127), where F is the word at +4 of the view's bitmap object (read by 0x46DBC3) [G: that word is the strip's frame count, 11 for 12557 and 4 for 12558/12067].

What to do: confirm F (read the bitmap object's ctor/loader that sets +4, read only in IDA), then make one function byte -> frame for the original's strips (theme viewer) and decide how the main panel's 11-state and 5-segment meters follow it (the redesign's meters need not match exactly; if the change is visible ask the user via a Decide task first). Unit tests at the frame thresholds; docs/ui_spec.md sections 2, 15, 16, 21 updated [C].
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 docs/ui_spec.md gives the meter byte-to-frame mapping with addresses, F confirmed [C]
- [ ] #2 The theme viewer draws the meters by that mapping; tests at the thresholds
- [ ] #3 The main panel's meters follow the mapping or a Decide task records why not
<!-- AC:END -->
