---
id: TASK-252
title: 'Main panel: the gain-reduction LEDs as the original''s bar from the left'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-19T12:38:00Z'
labels:
  - ui
  - re
milestone: m-5
dependencies:
  - TASK-251
references:
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/theme_view.rs
  - docs/ui_spec.md
  - task-251
  - task-187
  - task-179
priority: medium
type: enhancement
ordinal: 203600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Decision task-251 (the user, 2026-11-19, option A). task-187's like-for-like with the original showed that the compressor's gain-reduction meter (the nine LEDs under COMP; bitmap 12570 in the original) is a bar from the left: idle with the compressor on the master it lights LEDs 1 to 5; with the threshold at its lowest while playing, all nine; with the master's COMP switch off, none. The engine byte behind it is the compressor's live byte 3, trunc(min(G*18, 127)) (0x1000EB48): 18 at unity gain, 0 at gain 0, 127 at the top. OpenRB-338's main panel (task-179's redesign; docs/ui_spec.md sections 15 and 16; crates/orb-ui/src/link.rs, the gr_leds block after the meters; crates/orb-ui/src/model.rs, the "meter" part with GR_OFF) lights the center LED green and the reduction amber then red to its left, which the original never shows. The theme viewer already draws the bar (`theme_view::gr_lit`), with the 6 and 8 LED steps guessed.

What to do:
- Read the original's meter drawing in IDA (read only): sub_424A7E and what it calls, to get the exact byte-to-LED-count mapping (thresholds for 1..9 LEDs), with addresses; document it in docs/ui_spec.md (sections 15/16 and 21) as [C].
- The link keeps the raw byte (a Panel field) instead of the 0..4 count; one shared mapping function serves the main panel and the theme viewer (replace `gr_lit`'s guess).
- The main panel lights a bar from the left by place, in the redesign's colors by place: green, green, green, amber, amber, amber, red, red, red; none when the master's COMP is off or the byte is 0.
- Update the main panel's art/model code and its screenshot test if any, and the viewer's tests to the read thresholds.

The user checks the dev app (scripts/macos-dev-app.sh) before the agent review. IDA databases: /Users/pingw33n/dev/openrb-338/ida/ReBirth.dll.i64 and Rebirth.exe.i64, through the idalib MCP tools, read only, never saved; if the tools are unavailable, say so and stop before guessing the scale.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 docs/ui_spec.md gives the original's byte-to-LED-count mapping of the gain-reduction meter with addresses ([C]), read from sub_424A7E
- [ ] #2 One mapping function from the compressor byte to the lit LED count is used by both the main panel and the theme viewer, with tests at the thresholds and at 0, 18 and 127
- [ ] #3 The main panel lights a bar from the left in the colors green x3, amber x3, red x3 by place, none with the master's COMP off; covered by a test
- [ ] #4 cargo test, clippy -D warnings and nightly fmt pass; the user checked the dev app
<!-- AC:END -->
