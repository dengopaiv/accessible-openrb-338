---
id: TASK-222
title: 'Decide: a de-click ramp for the plugin''s Gate Pattern Mode stop'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-16T16:17:00Z'
updated_date: '2026-11-16T16:52:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies:
  - TASK-218
references:
  - docs/plugin.md
  - crates/orb-engine/src/frame/mod.rs
  - crates/orb-plugin/tests/gate_mode.rs
priority: medium
type: spike
ordinal: 228000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Context.** task-218 built Gate Pattern Mode in the plugin: a section sounds only while a note 0..31 on its channel is held, and the note-off stops it at once (the user's choice 1 of 2026-11-16). The stop is a hard cut: when the gate closes, the engine zeroes the section's channel bus from the next 64-frame block on (crates/orb-engine/src/frame/mod.rs, Frame::block, right after the device's slot 14 and before the inserts; the voices get slot 6). Measured over 40 note-off positions at the default levels (test the_stop_is_a_hard_cut in crates/orb-plugin/tests/gate_mode.rs, --nocapture prints them): the largest step from the last sample to 0 is 0.036 of full scale for a 303 (its peak 0.125), 0.047 for the 808 (peak 0.077) and 0.037 for the 909 (peak 0.069), i.e. 30..60 % of the section's peak (about -27 dBFS). On a sustained 303 note this is an audible click; on drums it truncates the decays. The start has no such problem: a gate opens on a step's trigger, and the voices start as they always do.

**Options.**
1. Keep the hard cut (what task-218 ships): exact 'stops at once', a click on sounding material.
2. A short linear fade of the closed section's bus, e.g. one 64-frame block (1.45 ms) from the gate's close, then zero: the section is silent one block later than now, no click. The fade would be applied in Frame::block for a gate that just closed (engine code: f64 math, rounded to f32 where the bus is stored, no FMA). Not in the original, gate only, so the corpus stays equal.
3. A longer fade (e.g. 5 ms = 3.4 blocks) for softer drum tails, at the cost of the 'at once' timing.

**Recommendation.** Option 2: a one-block linear ramp. It removes the click, keeps the stop within the 64-frame block resolution the plugin already has for every event, and needs no setting.

**Acceptance (for the follow-up, if a ramp is chosen).** The step to silence at a gate stop is below 0.002 of full scale in the_stop_is_a_hard_cut (renamed); the section is silent from the second block after the note-off; songs, Latch and Note mode unchanged (full corpus run equal).
<!-- SECTION:DESCRIPTION:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
task-218 review round 1: the test is now the_stop_is_silent_within_two_blocks (crates/orb-plugin/tests/gate_mode.rs); it prints the step to silence with --nocapture and asserts only silence within two blocks of the note-off, which holds with the hard cut and with a one-block ramp. A ramp's acceptance should add its own bound on the printed step.

Correction (task-218 review round 2): the test is the_stop_fades_out_over_one_block (crates/orb-plugin/tests/gate_mode.rs), not the_stop_is_silent_within_two_blocks; the fade replaced that interim name.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user (@pingw33n) on 2026-11-16, relayed by the supervisor: option 2, a one-block linear fade on the Gate stop only. Implemented in task-218's commit: Frame::gate_fade / gate_fade_gain fade the closed section's bus by (63 - i)/64 over the 64-frame block after the note-off, then slot 6 and silence. The step to silence is now below 0.002 of full scale (0.0016 for a 303, 0.0007 for the 808, 0.0008 for the 909, against 0.036/0.047/0.037 with the hard cut); test the_stop_fades_out_over_one_block in crates/orb-plugin/tests/gate_mode.rs. docs/plugin.md section 11.

Level dependence: the linear ramp's last non-zero sample has gain 1/64, so the step to silence is about peak/64. It is below 0.002 only for a section peak below about 0.128; the 303's default peak, 0.125, passes with little margin (0.0016 measured). A full-scale section gives about 0.016, still about 9 times smaller than a hard cut at that level.
<!-- SECTION:FINAL_SUMMARY:END -->
