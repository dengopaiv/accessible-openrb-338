---
id: TASK-245
title: >-
  Plugin note mode: the 909 clap sounds shorter than the same pattern in the
  preview
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-18T08:16:00Z'
updated_date: '2026-11-18T08:52:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - /Users/pingw33n/Documents/REAPER Media/909_a6_clap.RPP
  - crates/orb-engine/src/tr909.rs
  - crates/orb-plugin/src/core.rs
  - docs/plugin.md
  - task-200
  - task-203
  - task-242
priority: high
type: bug
ordinal: 450
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user (2026-11-18) dragged the 909's pattern A6 out of the plugin into REAPER and played it in note mode. The hand clap sounds shorter than in the preview of the same pattern.

**The project.** The user's REAPER project is `/Users/pingw33n/Documents/REAPER Media/909_a6_clap.RPP`: 120 BPM, 44.1 kHz. Read it; do not modify it.

**The notes.** One 2-second MIDI item, "909 A6", is looped, on MIDI channel 4 (status 0x93). It has 192 ticks per quarter note, so a step (a 16th) is 48 ticks, and every note is 24 ticks long, half a step.
- 36 (BD) at velocity 127;
- 42 (CH) and 46 (OH) at velocity 104;
- 39 (clap) at velocity 104, on steps 5 and 13.
- The item ends with CC 123.

**The plugin state.**
- It is in the `<VST ...>` block. Each line is its own base64 chunk: decode the lines one by one and join the bytes.
- Inside is a JSON object with `params`, and `fields.orb`, a JSON string holding the song, the patterns, `mode` (`note`) and `preview`.
- The preview set differs from the parameters only in `d4c1` (the 909's pattern selection: 0 against 5, A6). So the sounds and the mixer are the same in both modes.

**Where to look.**
- docs/plugin.md 9.2 says a 909 note plays its instrument through the step trigger code, and a note-off does nothing. Check that against the clap's behavior:
  - the clap's voice and its envelope or retrigger;
  - the shared RS/CP slot;
  - how the next steps' instruments (CH, OH, BD) affect a sounding clap in pattern playback versus note mode;
  - the velocity bands (104 is value 1 + AC 2) against the pattern step's actual value and accent, as the drag-out (task-203) writes them;
  - the flam;
  - anything the step function does to the clap that `Tr909::key_on` skips.
- Compare a render of pattern A6 in the preview or Latch Pattern Mode with the drag-out's notes in note mode. Use the plugin core with the restored state, as `crates/orb-plugin/tests/note_clock.rs` does. Find where the clap's samples diverge, then find the cause in the code and the original's disassembly, with addresses.

**Fix.** A note must sound as the pattern's step does. A difference that comes from the drag-out's encoding is fixed there. Record the cause in docs/plugin.md 9.2, and in docs/ if it is a fact about the original.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The cause of the shorter clap is identified with evidence (render comparison, code paths, and the original's addresses where it is a fact about the original) and recorded in the task notes and docs/plugin.md
- [x] #2 A test plays 909 pattern A6 of the user's state (or an equivalent pattern with the clap followed by CH/OH/BD steps) as drag-out notes in note mode and as the pattern, and the clap's samples match (within the documented tolerance of task-242/task-243)
- [x] #3 Pattern playback and the corpus are unchanged: full cargo test (corpus_golden included) passes; clippy -D warnings and nightly fmt clean
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Decision task-246 (the user, 2026-11-18): option 1, the velocity bands carry the flam.
1. tr909::velocity_band returns the step value 1..4 (3, 4 = with flam) and the AC byte: a flam slice at the bottom of each of the four bands (1..7, 48..55, 96..99, 112..115), so the drag-out's old velocities 40, 80, 104, 127 keep their meaning.
2. Tr909::key_on: a flam value arms a per-instrument note flam (Tr909::note_flam); Tr909::note_flams, run by the 909's audio slot before each block with the block's tick and tempo (the note-mode clock's), sets its tick as the step does (0x1002ABBC) on the first block and fires it as the flam pass (0x1002B320, accent*1.5) when the tick is past it. A plain note of the instrument, slot 6 and Play forget it.
3. clip: VEL_909 by flam, high, AC; values 3 and 4 written in the flam slices (4, 52, 98, 114).
4. Tests: engine note_tests (bands, a flam note against a flam step bit for bit, forgetting); clip tests; tests/note_clock.rs plays the user's A6 as drag-out notes and as the pattern.
5. Docs: midi.md 5.4, 5.5, 5.6; plugin.md 9.2, 9.5; plugin_usage.md.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-af7892f223f196b4b (based on main af9ef4f).

Investigation (2026-11-18).

The user's state (decoded from the RPP's <VST> block, line by line base64): 909 pattern A6 (pattern index 5, 194 bytes at 5*194 of the 909's array), every step AC byte 2. The clap column (7) holds value 3 on steps 5 and 13 (0-based 4 and 12): value 3 = low + flam. Other columns: BD 2 on 1,5,9,13; CH 1 on 1,2,5,6,9,10,13,14; OH 1 on 3,7,11,15; CC (49) 1 on step 1 (the RPP's note 0x31 = 49, which the task's list omits). Flam knob d4c29 = 54, accent knob d4c2 = 127, CP level d4c21 = 121. The clip writes the clap as velocity 104 (value 1 + AC), at tick 192 and 960, 24 ticks long: docs/midi.md 5.6 already says a value 3 is exported as value 1 and the flam is lost ("909 flam: no note", 5.4).

Cause: the flam. In the pattern, the 909 step 0x1002AAC0 arms the flam on every new step (flamTick = tick + (flam*tempo)/700000 + 1, 32-bit imul/cdq/idiv by 0xAAE60 at 0x1002ABBC, +0x1914, armed flag +0x1910) and a later call on the same step with tick > flamTick runs the flam pass: every column above 2 (cmp byte,2; jbe at 0x1002B324) retriggers at f32(accent*1.5) (dbl_10154350). docs/initial_investigation.md 5 [C] (task-025, three 909_flam_interval songs byte exact); not re-opened in IDA here. At 120 BPM and flam 54: d = 54*120000/700000 = 9, flamTick = 192+10, the pass runs at tick 203, 28.6 ms after the step. The clap is one SIMPLE voice (+0x18E0 slot 1, decay byte 127: DT = 1.0, no decay, the sample plays out), so the retrigger restarts the clap sample 28.6 ms later at gain accent*1.5 instead of accent*0.8 (x1.875, +5.5 dB): in the pattern the clap ends 28.6 ms later and its tail is 5.5 dB louder. Note mode plays one hit at accent*0.8 and arms no flam (Tr909::key_on; docs/plugin.md 9.2), so it sounds shorter and softer.

Render evidence (scratch test, the user's params and fields restored as nice-plug does, preview set applied: d4c1 = 5, d3c0 = 0; Latch Pattern Mode with the host rolling from bar 1 at 120 BPM, against the drag-out's notes (clip::notes) put in the blocks where the pattern plays its steps, as tests/note_clock.rs, 4 s, frame by frame):
- as is: first sample differing by more than one LSB at frame 23360, the block of tick 203 (the flam pass), 213902 frames over one LSB, max 0.94. RMS in 20 ms windows from the clap's step: identical for the first window, then pattern 0.268 / note 0.223 at 39 ms, 0.260 / 0.184 at 59 ms; the difference lasts the clap's whole length (470 ms).
- the same with the clap's value 3 changed to 1 in A6 (no flam) in both runs: no sample differs by more than one LSB (max 3.05e-5). So the flam is the only cause; the bands, the CH/OH/BD steps after the clap, the RS/CP voices and the trigger code give no difference.
- Failed hypothesis / setup error: a first run differed from frame 128 on because the parameter set has the 808 on (d3c0 = 1) and Latch mode played the 808's pattern; the preview set has it off (d3c0 differs too, not only d4c1 as the task says).
Scratch files: /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/t245/ (decode.py, scratch_t245.rs, params.json, orb.txt).

Stopped for a product decision (the supervisor's instruction). The drag-out cannot carry a flam with the current note mapping (docs/midi.md 5.4/5.5: four 909 velocity bands, no flam), and note mode arms none, so "a note must sound as the pattern's step" needs a change to the 909's note mapping or to note mode. Options:
1. Velocity bands carry the flam. The 909's four bands become eight (value 1/2 x AC x flam), the clip writes value 3/4 into the flam bands, and Tr909::key_on in a flam band arms a per-note flam on the note clock (task-242's clock): flamTick = tick + (FLAM knob * tempo)/700000 + 1 as the step, and the first block with tick > flamTick retriggers that instrument at f32(accent*1.5). Matches the pattern within the block timing, follows the Flam knob and the tempo like the pattern. Cost: narrower velocity bands for live playing; docs/midi.md 5.5 and the bands of the Standard Mapping docs change; flam per note rather than per step (the step's flam pass retriggers every flam column of the step together, which per-note arming reproduces when the notes share a block).
2. The clip writes the flam as a second note at the flam's tick. No engine change, but the retrigger's gain is a band's (0.8 or 2.0 x accent, not 1.5), its time is baked at the drag's tempo and Flam knob (10..11 ticks at 120 BPM, wrong at another tempo), and a 909 note-off then cuts nothing anyway. Close, not equal.
3. A separate flam control: a CC (or a spare note) that arms the flam for the next 909 notes, written by the clip before each flam step. Exact like 1 without touching the bands, but a new mapping element, two events per flam hit, and a stateful control live players must know.
4. Keep the loss: document the flam as not carried (it is already in docs/midi.md 5.6), close task-245 as a known limitation.
Recommendation: option 1. It keeps one note per hit, plays the original's flam pass with its knob and tempo, and needs no extra events; the band split can keep today's four centers (40, 80, 104, 127) as the non-flam velocities.

Implementation (after the decision task-246, option 1; rebased onto main 4a923a4).
- Bands: the flam slice at the bottom of each band (1..7 low + flam, 48..55 high + flam, 96..99 low + AC + flam, 112..115 high + AC + flam). Chosen so that the four velocities the drag-out wrote before (40, 80, 104, 127; the user's REAPER item holds 104 and 127) keep their non-flam meaning, and a hard hit within a band never flams. The clip writes 4, 52, 98, 114 for values 3/4 without and with AC. Value 5..255 (flam hit alone, hand-made files) stays exported as value 2 (no band holds a flam without its hit).
- Engine: Tr909::note_flam (per instrument column), armed by key_on in a flam slice; Tr909::note_flams runs in the 909's process (slot 14) before the voices, only while one is armed: the first block sets tick = block tick + (FLAM*tempo)/700000 + 1 (32-bit, as 0x1002ABBC), a later block with tick > it retriggers at f32(accent*1.5) with the note's AC (0x1002B320), columns 11..1. In the pattern the step function (and so the flam pass) runs in the sequencer's tick before the block's audio, and the block's tick there is the position after the advance; the note-mode clock's block tick is the same (docs/plugin.md 9.5), so the flam fires in the same block. Per-instrument rather than device-wide: a later plain note of the same instrument forgets it (as a new step disarms), other instruments' notes do not (live playing); identical for a clip, whose steps are 48 ticks apart (more than the longest flam below ~264 BPM). Slot 6 (silence) and slot 3 (start) forget all.
- Results: the user's real state (scratch test, preview set, notes in the pattern's blocks): before, max 0.94 from frame 23360; now no sample differs by more than one LSB (max 3.05e-5). New committed test tests/note_clock.rs the_dragged_909_flam_pattern_plays_as_the_pattern (the user's A6 bytes, Flam 54, Accent 127, CP level 121, written into 909_accent_level127): aligned max 3.05e-5 (one LSB), host frames max 1.9999695 (task-227's block shift); with the clip's flam velocities replaced by the plain ones the aligned max is 0.51 (the test fails), checked by a temporary edit.
- Engine unit tests (tr909/note_tests.rs): the 16 band edges; a flam note against a flam step bit for bit, block by block, at Flam 0, 54, 127, AC 0 and 2, four instrument sets (CP; CP+BD; OH, CH, SD; RC, LT, BD); a plain note, slot 6 and Play forget the flam.
- Checks: cargo test (whole workspace, corpus_golden included) 908 passed, 0 failed; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean.

Review round 1 (send back, two defects):
1. docs/plugin.md 9.5 still said only the PCF reads BlockContext::tick; the 909's note flams read ctx.tick and ctx.tempo in its process. Fixed there ("What reads the tick" and the clock bullet); docs/plugin_usage.md's 909 velocity item now points to the flam slices.
2. Velocity 0 and the 1..7 flam slice. Verified the path: orb-player's parser keeps 0x93 39 0 as a NoteOn, the core sends Command::Note with velocity 0, and Engine::live_note (crates/orb-engine/src/frame/mod.rs, `if velocity == 0 { dev.live_note_off(note) }`) turns it into the note-off before any key_on, so it never armed a flam through the engine. Tr909::key_on (public) now also ignores velocity 0 (velocity_band's doc says velocity 0 is never a band). Tests: tr909/note_tests.rs velocity_0_neither_plays_nor_arms_a_flam (key_on with 0 changes no voice, arms no flam, keeps a pending one) and tests/note_clock.rs a_velocity_0_note_on_plays_no_909_hit_and_no_flam (0x93 39/36/42 with velocity 0 through the core in note mode: silence, no hit). docs/midi.md 5.5 says velocity 0 is the note-off, not the slice's bottom.

Review round 1 checks: cargo test 910 passed, 0 failed (95 binaries, corpus_golden included); clippy -D warnings and nightly fmt check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The 909 clap of the dragged-out pattern A6 was shorter because of the flam: the clap's step value is 3 (low + flam), and the original's flam pass (0x1002AAC0; flamTick 0x1002ABBC, value > 2 at 0x1002B324, accent*1.5) restarts the non-decaying clap 28.6 ms after the step (Flam 54, 120 BPM) 5.5 dB louder, while the drag-out wrote value 3 as value 1 and a note armed no flam. Per the user's decision (task-246, option 1) the 909's velocity bands now carry the flam: a flam slice at the bottom of each band (1..7, 48..55, 96..99, 112..115; the old drag-out velocities 40, 80, 104, 127 keep their meaning), the drag-out writes 4, 52, 98, 114 for flam steps, and a flam note arms a per-instrument flam that Tr909::note_flams fires on the note-mode clock exactly as the flam pass (tick + Flam*tempo/700000 + 1, retrigger at accent*1.5). Docs: midi.md 5.4-5.6, plugin.md 9.2 and 9.5, plugin_usage.md.

Numbers: the user's state, notes in the pattern's blocks: max difference 0.94 before, one LSB (3.05e-5) after. New test tests/note_clock.rs the_dragged_909_flam_pattern_plays_as_the_pattern: one LSB (0.51 with the flam velocities replaced by the plain ones). Engine unit tests: a flam note equals a flam step bit for bit at Flam 0/54/127, AC 0/2, four instrument sets; a velocity-0 note-on neither plays nor arms a flam (review round 1).

Branch worktree-agent-af7892f223f196b4b, one commit on main 4a923a4. Tests: cargo test 910 passed (corpus_golden included, the corpus unchanged: only notes arm a note flam); clippy -D warnings and nightly fmt check clean.
<!-- SECTION:FINAL_SUMMARY:END -->
