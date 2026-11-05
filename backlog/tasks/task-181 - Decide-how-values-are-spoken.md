---
id: TASK-181
title: 'Decide: how values are spoken'
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
  - crates/orb-ui/src/behavior.rs
  - crates/orb-ui/src/model.rs
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: spike
ordinal: 204000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-D5; the analysis is docs/accessibility.md. Read both before starting.

- Options: A. The 0 to 127 value the original stores, with words for switches ("square"),
  pan ("center", "left 20") and the displays' own units (BPM, steps, bar). B. Physical units
  (Hz, ms, dB) for the synth and effect knobs. C. Percent.
- Recommendation: A: the original panel shows no units, so B's numbers would be estimates; A
  is exact and matches what a song file holds.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
**Decided (2026-11-05): B, made as precise as the code allows.** The user's words: "We can,
though these are estimates, still have them as precise as possible as we infer them from the
code." The engine is a bit-exact reimplementation, so the mapping from each 0..127 value to
its DSP parameter is in `crates/orb-engine` (for example the 303 filter's coefficient table
`FILT[reso >> 4][cutoff]`, `tables/mod.rs:43`; the decay multipliers per sample pair
`TR808_DECAY`, `PCF_DECAY`, `tables/data.rs`). Each unit is derived from there and marked by
how exact it is: computed in closed form from the engine's formula, derived numerically from
its tables or by measuring its output, or estimated. A quantity with no physical unit
(resonance, distortion shape) is spoken as a percent of its range. Switches, pan and the
displays keep their words and units. The work is task A11Y-14; until it lands, the value
text is the 0..127 number. The node's numeric value stays 0..127 (what Increment and
SetValue act on).
<!-- SECTION:FINAL_SUMMARY:END -->
