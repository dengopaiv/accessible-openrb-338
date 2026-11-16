---
id: TASK-227
title: >-
  Decide: plugin host events reach the block at or after their frame (never
  early)
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-16T19:14:00Z'
updated_date: '2026-11-16T19:23:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/resample.rs
  - docs/plugin.md
priority: low
type: spike
ordinal: 232000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Context (found in task-226 round 3, 2026-11-16).** The plugin applies a host event (a note, a parameter change) at host frame t to the engine block that holds t (docs/plugin.md section 4, task-197): after the host's latency compensation the event sounds from that block's start, up to 63 engine frames (1.4 ms; up to about 70 host frames at 48 kHz) BEFORE its frame. Mid-song that is a small jitter. At a realtime transport start whose frame is not on the 64-frame block grid (any start after a stopped stretch that is not a whole number of blocks), the early part falls before the play start, where the host drops it: the note's first frames are cut. Measured with Core::process_with (Note mode, 909 bass drum at frame 0 of the first playing call): at 44.1 kHz after 1000/7777/20000 stopped frames the note's onset came 40/33/32 frames early and 0.0103 of its peak 0.216 was in the dropped part (the attack's first frames; the peak itself was kept); at 48 kHz similar (onset up to 44 frames early). This is NOT the cause of the user's muffled first hit in task-226 (that is Reaper's fade on playback start, see task-226 notes), so task-226 does not change it.

**Options.**
(a) Keep today's rule (events quantize down to the block that holds them: early by 0..63 engine frames).
(b) Events reach the first block that starts at or after their frame (late by 0..63, never early): render the blocks that play before the event's frame ahead of time before applying it (Direct path: a ring of 3 blocks instead of 1; resampler: render_before with the exact rule (b+delay)*den <= (host-1+latency)*num, ring capacity 2W + 4 blocks). Same latency, still independent of the host's buffer sizes. A prototype exists (task-226 round 3): it passed first_note.rs including a new unaligned-start test, and needed one-block shifts in tests/gate_mode.rs (4 tests) and tests/sync.rs (2 tests, the after() helper becomes f.div_ceil(64)*64 + 128).
(c) Re-phase the block grid at each transport start (drop the rest of the current block), exact at the start but not for later notes, and a cut of up to 63 frames of whatever sounds.

**Recommendation.** (b): a note never sounds before its time, hosts drop nothing at a start, and lateness of under 1.5 ms matches hardware. It changes the documented timing of task-197 and task-218's gate tests, hence a decision.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user chooses (a), (b) or (c); the outcome and the date are recorded in the final summary
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user (@pingw33n) on 2026-11-16: option (b). A host event reaches the first 64-frame block that starts at or after its frame: notes and parameter changes sound 0 to 63 engine frames late, never early. Implemented in task-226's commit (round 4): crates/orb-plugin/src/core.rs (render_before, a 3-block ring for the 44.1 kHz path), crates/orb-plugin/src/resample.rs (Resampler::render_before; ring capacity 2W + 4 blocks); docs/plugin.md section 4 and 11 and docs/midi.md updated.
<!-- SECTION:FINAL_SUMMARY:END -->
