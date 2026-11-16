---
id: TASK-200
title: 'Plugin: note mode, host notes play the 303s, 808 and 909'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T11:12:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies:
  - TASK-185
  - TASK-197
  - TASK-199
  - TASK-207
references:
  - backlog task-185 (final summary)
  - docs/midi.md
  - crates/orb-engine/src/tb303.rs
  - crates/orb-engine/src/tr808.rs
  - crates/orb-engine/src/tr909.rs
  - docs/initial_investigation.md
priority: medium
type: feature
ordinal: 213000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
With orb stopped, the plugin is in note mode (task-185's final summary): host notes on the device channels 1..4 (303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4; task-207) play the devices, by the note mapping in docs/midi.md (task-199). The original never plays a device from a note (task-183), so this is new engine behavior; pattern playback must stay bit-exact.

- **303:** a monophonic voice driven by notes: pitch from the note, accent at velocity >= 100, slide when notes overlap (the next note slides from the held one), tie on same-pitch overlap, last-note priority, legato, gate = note length. Use the engine's own step machinery (note-on, slide, accent, gate) rather than a new synth, so the sound is the 303's.
- **808 and 909:** each note triggers its voice, as a pattern step would, with the accent factor of its velocity band (808 normal/AC; 909 low/high/low+AC/high+AC). A shared-slot note (tom/conga, rim/claves, clap/maracas) plays the voice it names for that hit only; the switch parameter is not changed. One voice per slot as the engine has it (the second note cuts the first). Chokes and the BD's two alternating voices as in pattern playback.
- Works whether or not the host plays. Notes never change host parameters.
- **Per-voice sounding mask:** the engine exposes, per block, which 808/909 voices are sounding (active and gain above about -60 dB) and whether the voice's last hit had AC, for the note-mode visual feedback task.

Notes in pattern mode keep their pattern-select meaning (task-198).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: a 303 note sequence gives the expected pitch, accent, slide, tie and gate (checked against the engine's step path with an equivalent pattern where possible)
- [x] #2 Tests: each GM drum note triggers the right voice; shared-slot notes pick their voice without changing the switch parameter; velocity bands give the documented gains
- [x] #3 The reference corpus is unchanged (corpus_golden passes): pattern playback is bit-exact
- [x] #4 The per-voice sounding mask and the AC flags are in the snapshot, with a test
- [x] #5 No allocation in the audio path (no_alloc test)
- [x] #6 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Engine (orb-engine): ChannelDevice gets live_note_on/live_note_off (default no-op) and sounding() (default empty); Engine::live_note(device, note, velocity) (velocity 0 = off), ignored while the song plays.
2. Tb303: split note_on(step) into the semitone core (same operations, so the step path is unchanged); note mode keeps a fixed held-note stack (no allocation): MIDI m -> semitones m - 36 plus the Tune table at note-on; notes whose table index tuneTab + m - 36 is outside -48..47 ignored; accent = velocity >= 100; a note while another is held sets the slide flag first (no retrigger, glide; same pitch = tie); release of the sounding note slides back to the newest still held (last-note priority); release of the last note = note_off (gate = note length). Stack cleared by slot 6 (silence) and slot 3 (start).
3. Tr808/Tr909: trigger split into trigger_as(i, select override); a note sets the accent (808: AC=1 at velocity >= 100; 909: AC=2 and gain 0.8/2.0 by the four bands) with the step's expressions and triggers the GM note's instrument; shared 808 slots pick their voice for the hit only. Track which sound each shared slot played and the last hit's AC (step path too; no numeric change), and compute the sounding mask (voice active and peak gain >= -60 dB) per sound.
4. orb-player: Command::Note, Snapshot::sounding [Sounding; 4].
5. orb-plugin core: in note mode (mode Stop) note-on/off on channels 1..4 go to Command::Note (hold_mode first so a pending Stop lands before the note); pattern mode unchanged.
6. Tests: tb303 note path vs step path bit-exact (plain, accent, slide, tie, octave), range and priority; 808/909 GM mapping, shared slots, gains vs the step path; sounding mask and AC flags (engine + snapshot); plugin note mode test; no_alloc in player and plugin with note-mode notes.
7. Decide task for the pattern section's on/off switch in note mode (provisional: notes play regardless).
8. Docs: docs/plugin.md note-mode section, docs/midi.md 5.3/5.4 pointers.
9. corpus_golden, cargo test (ORB_CD_ISO state noted), clippy, nightly fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ac9291e1e0583c805 (rebased on main ed7100a).

Research 2026-11-16 (no IDA needed: every engine fact used is already [C] in docs/midi.md 5 and the engine code):
- The sequencer's step runs in Frame::block before the devices' audio (seq.tick then process), so a note applied between blocks equals a step at the next block's start: the basis of the bit-exact comparison.
- 303: note_on 0x10023CD4 takes the step bytes; the semitones are (int16)(note + 12*up - 12*down) (0x10023E61). Split into note_on_semis (same operations, same order) so the step path is unchanged; note mode passes m - 36. The slide flag +0xCA1 is what a sliding step leaves for the next note-on (no retrigger, glide) and what makes note_off skip; note mode sets it before a legato note-on, and every note-mode note-on passes slide byte 0 so the last release closes the gate.
- 808/909: the accent expression of the step (0x10027821 / 0x1002ACAF) moved into set_accent (same expression); the 808's trigger got a switch override (trigger_as, the step passes None). 909 gains 0.8/2.0 as 0x1002B011/0x1002AD02.
- Drum voice end test: flt_1015364C = 3e-8 on the gain with 2^-15 folded in = 9.8e-4 of full scale = -60.2 dB, so the -60 dB sounding threshold coincides with the voices' own end for SIMPLE/FILTER/DUAL (found when a test expecting an active-but-quiet OH failed: the voice had already ended). Documented in docs/plugin.md 9.3.
- First run of the plugin drum test failed with peak 0: a new instance's mixer gains rise from 0 (0.07 a block smoothing) and the int16 output truncates; the test now warms up 4096 frames first.
- Mode turning to Stop + a note in the same block: without care the note lands first and the next block's hold_mode Stop silences it (slot 4 -> slot 6). core.rs now calls hold_mode before sending the note; test a_note_right_after_the_mode_turns_to_stop_is_not_silenced.
- Open question -> task-210 (Decide, @pingw33n): does a section's On/off switch mute note mode? Provisional A: not read (the switch is the pattern sequencer's; the mixer mute applies in both modes). Two 303s are on separate channels, each monophonic: no polyphony question.

Verification 2026-11-16 (ORB_CD_ISO not set in this environment):
- cargo test (optimized test profile): 777 passed, 0 failed, 5 ignored.
- cargo test --release -p orb-engine --test corpus_golden: 4 passed (diag, app at fitted N, shipped, shipped-out-of-corpus): pattern playback bit-exact.
- New tests: orb-engine tb303::note_tests (8; step path vs note mode bit-exact in both sound models: plain/accent/octave/3 Tune settings, slide chain, tie; MIDI 36 = 49152, A4 = 440 Hz; range 0..83 at center, 95 at Tune -12, 71 at +12; priority/legato; play/silence forget), tr808::note_tests (6; all 16 GM notes x velocities 1/99/100/127 equal to a one-instrument step incl. AC, switch unchanged, gains x(1+knob/32), other notes nothing, BD alternation, CH choke, sounding mask/AC incl. pattern steps), tr909::note_tests (5; 11 notes x 8 band-edge velocities equal to a step with value/AC, gains 0.8/2.0/2.4/6.0 at knob 32 and 2.0 at 24, CH darkens OH, BD either voice), orb-player tests/note_mode.rs (3) and no_alloc with Command::Note, orb-plugin tests/note_mode.rs (4) and no_alloc with note-mode notes.
- cargo clippy --all-targets -D warnings: clean; cargo +nightly fmt --all --check: clean.

Review round 1 (2026-11-16, send back, no code change): (1) task-200 was Done while depending on task-210 (To Do); by the supervisor's choice the task-210 dependency is dropped (the dependencies are back to task-185, task-197, task-199, task-207: the earlier --dep task-210 had replaced them), and the final summary records option A as provisional with task-210's AC #2 covering the follow-up. (2) docs/plugin.md 9.3 no longer restates the original's drum end test; it points to docs/initial_investigation.md section 7.1 and keeps only orb's -60.2 dB comparison.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Note mode implemented (branch worktree-agent-ac9291e1e0583c805, one commit).

- Engine: ChannelDevice::live_note_on/live_note_off/sounding and Engine::live_note (ignored while the song plays). 303: note_on split into note_on_semis (step path unchanged); key_on/key_off with a fixed 16-note held stack: semitones m - 36 plus Tune at note-on, notes outside the pitch table ignored, accent at velocity >= 100, overlap sets the slide flag (glide, no attack; same pitch = tie), release of the sounding note slides back to the newest held (last-note priority), last release = note-off; slots 3 and 6 forget the stack. 808/909: key_on plays the GM note's instrument through the step's trigger code with the step's accent expression (808 AC=1 at >= 100; 909 bands 1..47/48..95/96..111/112..127 as value 1/2 with AC 0/2); 808 shared slots via a switch override (trigger_as), switch unchanged. Sounding {mask, ac} per sound (tr808::sound 16 bits, tr909::sound 11), voice active and peak >= 0.001 (-60 dB); last-hit AC tracked in pattern mode too.
- orb-player: Command::Note, Snapshot::sounding. orb-plugin: in note mode notes on channels 1..4 (note-off and velocity 0 too) go to Command::Note, after hold_mode so a pending Stop does not silence them; pattern mode unchanged.
- Docs: docs/plugin.md section 9 (9.3 compares the -60 dB threshold with the original's 3e-8 end test of docs/initial_investigation.md 7.1), docs/midi.md 5.3/5.4 pointers.
- Section On/off in note mode: option A (notes play whatever the section's On/off switch says; the switch is not read) is implemented provisionally. The user decides in task-210; if the choice differs, task-210's AC #2 covers the follow-up task. No dependency edge between the two.
- Tests: cargo test 777 pass / 0 fail / 5 ignored (ORB_CD_ISO not set), corpus_golden 4/4 pass (bit-exact), clippy and nightly fmt clean.
- Review round 1: dependency on task-210 dropped, the drum end-test restatement in docs/plugin.md replaced by a cross-reference.
<!-- SECTION:FINAL_SUMMARY:END -->
