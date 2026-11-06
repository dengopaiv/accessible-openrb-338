---
id: TASK-205
title: 'Decide: Page Up, Page Down, Home and End on a spin button'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-06T05:51:00Z'
updated_date: '2026-11-06T06:26:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/a11y/drive.rs
  - docs/accessibility.md
priority: medium
type: spike
ordinal: 223000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Found while doing TASK-187 (A11Y-3), which settles the keys of the sliders (Up/Down 1, Page Up/Down 8, Home 0, End 127) and of the pattern selector (Up/Down 1 pattern, Page Up/Down 1 bank, Home A1, End D8), but whose description gives the spin buttons only Up and Down: each is one click of the display's up or down arrow (`Panel::arrow`), so the engine hears what a click sends.

The spin buttons (crates/orb-ui/src/a11y/drive.rs, `SPIN_DISPLAYS`): tempo (1 BPM per click, 20..500 per `Transport::tempo_step`), bar, loop start, loop length (1 bar per click; song mode only), each row's pattern length (1..16 steps), delay steps (1..32), PCF pattern (0..54). Today Page Up, Page Down, Home and End do nothing on them (`drive::step`).

Options:
A. Leave them doing nothing (as now): one click per key, as the mouse.
B. Page Up/Down = 10 clicks (tempo 10 BPM, 10 bars), Home/End = clicks up to the end of the range (pattern length 1/16, delay 1/32, PCF 0/54; tempo 20/500 would be up to 480 clicks, each an engine event).
C. As B, but Home/End only where the range is short (pattern length, delay steps, PCF pattern), not tempo and the transport positions.

Each click is a separate engine event (and a separate recorded event in song mode), as a mouse user holding the arrow makes them.

Recommendation: C. Page Up/Down 10 clicks everywhere; Home/End on the short ranges only, so a slip on Home cannot send a burst of hundreds of tempo events.

Acceptance: the user's choice is recorded here with the date; the implementing change (drive.rs `step` for `Target::Spin`, docs/ui_spec.md section 20, tests in crates/orb-ui/tests/a11y_drive.rs) is a follow-up task or part of TASK-188.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user's choice and its date are in the final summary
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-06: option C, the recommendation. On a spin button in screen reader mode, Page Up and Page Down are 10 clicks of the up or down arrow (each sent as a click sends it); Home and End click to the end of the range only on the short ranges (each row's pattern length 1..16, delay steps 1..32, PCF pattern 0..54) and do nothing on tempo, bar, loop start and loop length. The pattern selector keeps its keys (Page Up/Down one bank, Home A1, End D8).

Implemented in TASK-187's commit: crates/orb-ui/src/a11y/drive.rs (`step` for `Target::Spin`, `short_range`, unit test the_short_ranges_are_the_arrows_clamps), crates/orb-ui/tests/a11y_drive.rs page_home_and_end_on_a_spin_button (Page Up/Down give the events of 10 arrow clicks on all 10 spin buttons; Home/End reach the bounds of the short ranges; nothing on tempo and the transport positions), docs/ui_spec.md 20.1, docs/accessibility.md 3.3.
<!-- SECTION:FINAL_SUMMARY:END -->
