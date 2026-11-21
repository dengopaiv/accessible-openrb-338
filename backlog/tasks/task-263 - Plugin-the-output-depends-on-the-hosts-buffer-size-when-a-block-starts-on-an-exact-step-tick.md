---
id: TASK-263
title: >-
  Plugin: the output depends on the host's buffer size when a block starts on an
  exact step tick
status: To Do
assignee: []
created_date: '2026-11-21T09:18:00Z'
updated_date: '2026-11-21T09:37:00Z'
labels:
  - player
  - tests
milestone: m-7
dependencies:
  - TASK-244
references:
  - crates/orb-plugin/src/core.rs
  - docs/plugin.md
  - crates/orb-plugin/tests/reaper/cases.rs
priority: medium
type: bug
ordinal: 250600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** task-244's REAPER test found that the plugin's output is not a function of the host's timeline alone. The host's tick at the start of engine block m is m * tempo * 512 / 110250 (tempo in BPM). At 125 BPM it is exactly 768 (bar 2) at block 1323. `Engine::sync` (crates/orb-plugin/src/core.rs) extrapolates the host's position from the record of the host call in force: beats = p + (h - start) * bpm / 60 / rate, then tick = beats * 192. Depending on where that call started, the tick at block 1323 comes out as 768.0 or as 767.9999999999999 (scratch computation, same f64 order: at 48 kHz, h = 92160, calls of 128, 256, 512, 1024, 2048 frames give either value by start). From that bar line on the output differs. docs/plugin.md section 4 claims the output does not depend on the buffer sizes; that holds only away from such exact ticks.

**Evidence (in-process, through `Core::process_with`, the host rolling from beat 0 at 125 BPM):**
- Latch Pattern Mode, full_mix, 48 kHz: calls of 256 and 512 frames differ from output frame 92161 (1.92 s) to the end, up to 0.16. REAPER 7.86's render matched calls of 1, 32, 256 and 1024 frames (VST3), and its CLAP render matched none of these exactly from bar 3 on.
- Note mode, 808_accent_level127 with the PCF on the 808 (wave 0), its drag-out clip, 44.1 kHz: calls of 64 and 1024 frames differ from frame 84608, up to 0.12 (the note-mode clock's PCF step, task-242).
- task-244's test avoids it with 120 BPM (the first block on a whole tick that is a multiple of 24 is 11025, 16 s), see docs/plugin.md 7.1.

**What to do.** Make the per-block host tick independent of the call it is extrapolated from at exact ticks, for example by computing the block's position from a call-independent quantity, or by snapping a tick within a tiny epsilon of an integer. This is a behavior change of the plugin's sync, so keep sync.rs, timing.rs, note_clock.rs and gate_mode.rs passing and add a test. Decide whether the fix belongs in the plugin or is acceptable as is; if it needs a product decision, open a Decide: task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A test in crates/orb-plugin/tests/timing.rs plays full_mix in Latch Pattern Mode at 48 kHz and 125 BPM in calls of 64, 256, 512 and 1024 frames and gets the same output for all
- [ ] #2 A test in crates/orb-plugin/tests/note_clock.rs plays note mode with the 808's drag-out clip and the PCF at 44.1 kHz and 125 BPM in calls of 64 and 1024 frames and gets the same output for both
- [ ] #3 docs/plugin.md section 4's buffer-size claim and section 7.1's note are updated
<!-- AC:END -->
