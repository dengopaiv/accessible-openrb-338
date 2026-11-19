---
id: TASK-254
title: 'Decide: the main panel''s meters on the original''s log scale'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-19T14:00:00Z'
updated_date: '2026-11-19T14:20:00Z'
labels:
  - decision
  - ui
milestone: m-5
dependencies: []
references:
  - docs/ui_spec.md
  - crates/orb-ui/src/model.rs
  - task-253
priority: low
type: spike
ordinal: 245000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-253 read the original's peak-meter scale (Rebirth.exe; docs/ui_spec.md section 2, 'Meters' [C]). The original maps the engine's meter byte b (0..127; the mixer channels', the master's two, the delay, dist, PCF and comp input meters) through the table sub_424220, L(b) = trunc(ln(b) / ln(127) * 127), and shows frame trunc(L * (F - 1) / 127) of a strip of F frames (sub_4249A8, sub_467F31; F = 11 for the master's 12557, 4 for the mixer's 12558 and the effects' 12067, set by sub_46DB82 from 0x4287A0's resource list). A logarithmic scale: the master's 11 frames step at the bytes 2, 3, 5, 7, 12, 19, 30, 49, 81, 127; the four-frame header meters are green from 6, amber from 26 and red only at 127.

OpenRB-338's main panel (task-179's redesign; model::meter_state, model::header_meter_segments) is linear: state = ceil(b * 10 / 127), so the master's 11 states step at 1, 13, 26, 39, 51, 64, 77, 89, 102, 115, and the header meters (4 segments, 5 states, [0,1,1,1,2,2,2,3,3,4,4][state]) light at 1, 39 (2 segments), 77 (3), 102 (4, red). The theme viewer (Mods > Theme Viewer) follows the original since task-253. Following the original makes the main panel's meters visibly livelier: a quiet signal lights much more of them (byte 30 lights 7 of the master's 10 segments, against 3 now).

Options:
A. One scale, the original's: model::meter_state becomes the original's 11-frame mapping (peak_meter_frame(b, 11)); the master's meters then match the original exactly, and the header meters keep header_meter_segments over it (segments from the bytes 2, 7, 30, 81: amber and red where the master's amber and red segments light, as task-179 chose).
B. The original's formula per strip: the master as in A; the header meters as peak_meter_frame(b, 5) for their 5 states (segments from the bytes 4, 12, 39, 127: red only at full scale, as the original's red frame).
C. Keep the linear scale (no change).

Recommendation: A. The master's meters then show what the original shows, and the header meters stay consistent with the master's colors, as the redesign intended; B's red top segment would light only at byte 127, so it would almost never show in the 4-segment headers. The change is a few lines in model.rs (meter_state) and its tests; Panel::meters already holds the raw byte (task-253).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user's choice is recorded in the final summary with the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-19: option A. The main panel's meters follow the original's log scale with one scale: model::meter_state is the original's 11-frame master meter, peak_meter_frame(byte, 11). The master's states step at the bytes 2, 3, 5, 7, 12, 19, 30, 49, 81 and 127. The header meters keep header_meter_segments over that state, with segments from the bytes 2, 7, 30 and 81. Applied in task-253's commit.
<!-- SECTION:FINAL_SUMMARY:END -->
