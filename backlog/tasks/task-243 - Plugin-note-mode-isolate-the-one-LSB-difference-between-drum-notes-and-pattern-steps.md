---
id: TASK-243
title: >-
  Plugin note mode: isolate the one-LSB difference between drum notes and
  pattern steps
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-18T05:44:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/tests/note_clock.rs
  - docs/plugin.md
  - crates/orb-engine/src/tr808.rs
priority: low
type: task
ordinal: 600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** task-242 compared an 808 and a 909 pattern played in Latch Pattern Mode with their drag-out (clip::notes) played into note mode, the notes put in the very blocks where the pattern plays its steps (tests/note_clock.rs, the 'pattern blocks' run of compare). The PCF's steps match block for block, but the outputs differ by one LSB of the int16 output (1/32768) in about 3400 of 176400 frames, first at frame 3394 (808_accent_level127), 3396 (909_accent_level127), 3438 (shuffle_drums_g64). The difference is there with the PCF off too, so it is the drum voices' path, not the PCF clock. docs/plugin.md 9.5 records it; section 4 (task-226) saw a similar 1-LSB difference between two later notes (state that runs on in silence).

**Hypothesis (unverified).** Pattern mode's clean start sends Stop, Locate 0, Play (src/core.rs sync): the Stop's slot 4/6 silences and resets voice or filter state that note mode leaves as the 256 settle blocks left it (tiny residual values that shift the int16 truncation). Note mode sends no Stop at activation. Other candidates: the process-global drum counters (ProcessState), the BD's alternating voice toggle, the snare's noise sequence.

**Work.** Find which state differs (compare the 808's voices after each block in both runs), record the cause in docs/plugin.md 9.5 with the code location, and propose (do not make) a change if one is worth it; a change of note mode's sound is a Decide: task for @pingw33n.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The state that makes the one-LSB difference is identified, with the code location and the first block it differs in, and recorded in docs/plugin.md 9.5
- [ ] #2 If a change is proposed, it is a Decide: task with the options and a recommendation; nothing in the engine's pattern path or the corpus changes
<!-- AC:END -->
