---
id: TASK-246
title: 'Decide: how note mode plays the 909 flam'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-18T08:25:00Z'
labels:
  - decision
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - task-245
  - docs/midi.md
  - docs/plugin.md
priority: high
type: spike
ordinal: 242000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Context (task-245).**
- A 909 step of value 3 or more (with flam) is retriggered by the original's flam pass, 0x1002AAC0 (flamTick 0x1002ABBC, the value > 2 test at 0x1002B324). The retrigger plays at accent×1.5, a Flam-knob time after the step (28.6 ms at knob 54 and 120 BPM).
- A note-mode note cannot carry the flam: the 909's four velocity bands give value 1 or 2, with or without AC, and `Tr909::key_on` arms no flam.
- So a dragged-out pattern loses the flam. In the user's pattern A6 the clap sounds shorter and 5.5 dB softer than in the preview.

**Options.**
1. The velocity bands carry the flam: four bands become eight, and a flam band's note arms a flam on the note-mode clock. Exact, but narrower bands.
2. The drag-out writes a second note at the flam's tick. Approximate in gain and timing.
3. A CC or a spare note arms the flam. Exact, but stateful.
4. Keep it as a documented limitation.

Recommendation: option 1.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The user decided on 2026-11-18: option 1, the velocity bands carry the flam. The 909's note velocity bands become eight (value 1 or 2, with or without AC, with or without flam). A note in a flam band arms the flam as the step function's flam pass does (the Flam knob and the tempo, timed on the note-mode clock) and retriggers at accent×1.5. The drag-out writes the flam band for a flam step. Implemented in task-245; docs/midi.md 5.5 and 5.6 and docs/plugin.md 9.2 are updated there.
<!-- SECTION:FINAL_SUMMARY:END -->
