---
id: TASK-264
title: >-
  Plugin: REAPER test of a looped host transport (Latch Pattern Mode over a
  loop, real time)
status: To Do
assignee: []
created_date: '2026-11-21T09:18:00Z'
labels:
  - tests
  - player
milestone: m-7
dependencies:
  - TASK-244
references:
  - crates/orb-plugin/tests/reaper.rs
  - docs/plugin.md
priority: low
type: task
ordinal: 250700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** task-244 asked for Latch Pattern Mode over a looped time selection in REAPER. Its fixture latch_loop (LOOP 1, selection 2..4 s, render 0..6 s) showed that REAPER 7.86's command-line render (-renderproject) does not loop: it renders the linear timeline, bit-identical to the project without the loop (docs/plugin.md 7.1). So the plugin under a host transport that jumps back at a loop end is not covered by a real host.

**Context.** The test's harness (crates/orb-plugin/tests/reaper.rs) runs REAPER as -newinst on a scratch -cfgfile resource folder with only target/bundled on the plugin paths and REAPER's dummy audio device. One try at a real-time capture failed: a ReaScript passed on the command line (project then script) that record-armed the instrument track with I_RECMODE 1 (output), turned repeat on (GetSetRepeat(1)), ran Transport: Record (1013) for 7 s, then Stop (save all recorded media, 40667), recorded no media with the dummy device. Options to try: a send to a second track recording its input; 'Render: Render project, using the most recent render settings' does not loop either; 'Apply track FX' neither. A real-time capture also depends on REAPER's call sizes (task-263 first).

**Done when** a REAPER render or recording of Latch Pattern Mode over a loop runs in the opt-in test and is compared with the in-process host whose transport jumps at the same frames, or the task records why REAPER cannot do it headless.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A looped Latch Pattern Mode case runs in cargo test -p orb-plugin --test reaper with ORB_REAPER set, or the notes show why REAPER cannot record it headless
- [ ] #2 docs/plugin.md 7.1 describes the result
<!-- AC:END -->
