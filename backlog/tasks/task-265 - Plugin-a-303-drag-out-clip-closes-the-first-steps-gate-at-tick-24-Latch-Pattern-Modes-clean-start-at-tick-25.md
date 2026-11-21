---
id: TASK-265
title: >-
  Plugin: a 303 drag-out clip closes the first step's gate at tick 24, Latch
  Pattern Mode's clean start at tick 25
status: To Do
assignee: []
created_date: '2026-11-21T09:40:00Z'
labels:
  - player
  - engine
milestone: m-7
dependencies:
  - TASK-244
references:
  - crates/orb-plugin/src/clip.rs
  - crates/orb-plugin/tests/reaper/cases.rs
  - docs/plugin.md
  - docs/midi.md
priority: low
type: bug
ordinal: 250650
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** task-244's REAPER test compares a 303 pattern's drag-out clip (`clip::notes`, task-203), played in note mode, with the same pattern in Latch Pattern Mode. It places every note in the engine block where the pattern plays its step (the 'aligned' variant). The two renders agree within one LSB everywhere except the first note's release. There they differ by up to 0.0045 (diag song 303_slide at 120 BPM, 44.1 kHz, from output frame 2757 into the second note).

**Cause, as measured.** Latch Pattern Mode's clean start (Stop, Locate 0, Play; crates/orb-plugin/src/core.rs `Engine::sync`) makes the sequencer's clock step the devices from tick 1, so step 1 plays at tick 1. Its gate closes at v + 24 = tick 25 (`gate_off = v + 24`, 0x100254D3). The clip closes every unslid step's gate 24 ticks after the step's start (`clip::GATE`), so the first note ends at tick 24. Moving the clip's first note-off to tick 25 made the whole aligned render equal within one LSB; at 125 BPM the in-process check gave max 3.05e-5. task-244's fixture tb303_slide_aligned now does this explicitly (cases.rs, `notes`, the first step's note-off). Not covered by task-227 (the one-block timing gap of host events) or task-243 (the drums' one-LSB difference).

**Question.** Which is right for the first step, and should the drag-out clip's first note end at tick 25? Find out from the original what Play from a bar start does to the first step: does the clock step it at tick 0 or tick 1 (`sub_1001EF20`, docs/initial_investigation.md section 5), and when does its gate close? Then make the clip and the pattern agree. If the answer is a product choice (for example, the clip models a looping pattern, where every later gate closes at step + 24), open a Decide: task for @pingw33n with the options instead.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The notes record, with addresses, at which tick the original plays the first step after Play from a bar start, and when its gate closes
- [ ] #2 The clip and Latch Pattern Mode agree on the first 303 note's end, or a Decide: task holds the choice
- [ ] #3 If the clip changes, src/clip/tests.rs covers it, and task-244's tb303_slide_aligned fixture drops its explicit tick-25 note-off (crates/orb-plugin/tests/reaper/cases.rs) and still matches within one LSB
<!-- AC:END -->
