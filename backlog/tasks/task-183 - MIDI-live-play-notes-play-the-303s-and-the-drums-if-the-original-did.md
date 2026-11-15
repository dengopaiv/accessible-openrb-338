---
id: TASK-183
title: 'MIDI live play: notes play the 303s (and the drums if the original did)'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-13T18:20:00Z'
updated_date: '2026-11-15T13:17:00Z'
labels:
  - re
  - engine
  - player
  - docs
milestone: m-7
dependencies:
  - TASK-182
references:
  - docs/initial_investigation.md
  - docs/midi.md
  - crates/orb-engine/src/tb303.rs
priority: medium
type: feature
ordinal: 199000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user decided (2026-11-13) to bring MIDI into scope in milestone m-7. task-182 adds the original's MIDI Standard Mapping (CCs and pattern selection doing what the panel does) as a source-independent layer that turns timed MIDI bytes into player commands. This task adds the other half: playing the devices live from MIDI notes, which the panel cannot do. MIDI input is first used inside the VST3/CLAP plugin (the user, 2026-11-15): patterns exported to the host as MIDI clips play back through it as notes. The standalone app's MIDI port input is task-188's, not this task's.

What is known: the DLL has a MIDI input for the 303, `sub_100261F0`, which also calls the record hook (`docs/initial_investigation.md` section 5.3). Whether the 808 and 909 also played their instruments from MIDI notes is not known.

Reverse engineering is deliberately minimal: find out whether and how live play worked in the original. For the 303: the channel, how a note sets pitch, how velocity or other data sets accent and slide, what happens to a running pattern while notes are played (override, mix, or only when stopped), and whether played notes are recorded. For the 808 and 909: whether notes trigger instruments and with which note map. Addresses are checked against the disassembly. The output does NOT have to match the original sample for sample; it should sound and behave as a user would expect (correct pitch, accent, slide and envelopes, no stuck notes, correct timing). If the original had no live play for a device, record that and do not invent it without a Decide task.

The user's IDA databases are read only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/midi.md gains a live-play section: what the original's 303 MIDI input (sub_100261F0) does with notes, velocity, slide and a running pattern, and whether the 808 and 909 play from notes, each fact [C] with addresses or [G], plus where orb deliberately differs
- [x] #2 MIDI notes play each device the original let play live, through the MIDI layer of task-182; no MIDI port, menu item or setting is added (the standalone app's port input is task-188's)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
The RE shows the original had no live play for any device (notes above). The task says: record that and do not invent it without a Decide task. So:
1. docs/midi.md: a new section 4 'Live play (task-183)': the original plays no device from notes, with the evidence (sub_100261F0 is an unreachable DLL-side copy of the 303's remote control, its full behavior; every other device's slot 12 is nullsub_4; the 303 note-on/off callers; the exe's note dispatch; the manual p. 183); differences from the manual (none on this point; the dead handler's note 31 = Step agrees with p. 202 where the exe does not); what orb does (nothing new) and the pending decision. Section 3's 'Not mapped yet' and the intro line updated.
2. docs/standard_midi_mapping.md: sections 7 and 9 item 4 mention the DLL's unreachable copy (note 31 = Step there). docs/original_bugs.md OB-25: the same remark. docs/initial_investigation.md 5.3: correct 'The 303's MIDI input sub_100261F0 calls it too (out of scope)'.
3. crates/orb-player/src/midi.rs: module doc says the original plays no device from notes (no code change otherwise); a unit test that no note on any channel or in either keyboard mode gives anything but a Standard Mapping action, i.e. no note sounds a device, documenting the original's behavior.
4. Backlog: a Decide task for @pingw33n (add live play the original lacks? which devices, channel, accent/slide conventions, running pattern, recording, drum note map) with options and a recommendation; an implementation task depending on it that takes over AC #3..#5 (tests of pitch/accent/slide/note-off/overlap/all-notes-off/running pattern, no stuck notes, the manual VMPK/sendmidi check), which have nothing to test while no device plays live. AC #3..#5 move there (recorded in the notes), as release-only criteria were moved before.
5. cargo test, clippy, nightly fmt; no engine code changes, so the corpus is untouched (golden test run anyway to state it). Rebase, one commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a9e284d08fcfa319f

RE (2026-11-15, read-only headless copy of ReBirth.dll.i64 in the scratchpad, closed save=false; disassembly checked, Hex-Rays second opinion):
- sub_100261F0 is NOT a live-play input. It is slot 12 of the CTB303Device vtable (base 0x10033FD4, entry at 0x10034004; slot 11 setParam 0x10026180 at 0x10034000, slot 13 panel update 0x10026700 at 0x10034008), args (msg*, on, programMode), retn 0Ch. It reads status at msg+8, d1 at +0xC, d2 at +0xD.
  - 0xB0 (0x10026228): CC base = 7*dev+16 (dev = sub_10010350, 1 or 2: CC 23..29 / 30..36); +0 waveform v>=64 (0x100262B9 cmp 40h), +1..+6 tune, cutoff, reso, envmod, decay, accent -> live bytes +0xC7E..+0xC84, changed flags +0xC8A..+0xC90, record hook sub_10012BC0(dev, ctrl, v). Same as the exe's sub_4405B5.
  - 0x90 with programMode == 0 (0x10026468..): base = 13*dev - 1 (12 / 25); +0 toggles enable +0xC7C (setz 0x100264DD) and records ctrl 0; +1..+4 set bank +0xC86 (and +0xC92); +5..+12 pattern = 8*bank + i at +0xC7D (0x1002654B) and records ctrl 1. The 'on' flag is NOT tested in this branch.
  - 0x90 with programMode != 0 and on (0x10026579): edits the step at cursor +0xC87 of the live pattern (+0x3C + 98*pat + 2 + 6*step): notes 12..24 note = n-12 (0x100265D3); 25 toggles down (+6), 26 up (+5), 27 accent (+4), 28 slide (+3), 29 rest (+7); 30 cursor back with wrap 0->15 (0x1002669E), 31 cursor forward with wrap 15->0 (0x100266B8); 32 not handled (jump table 0x10153D38 default).
  - 0x80 is never tested: note-off does nothing.
  - NO caller: the only reference is the vtable entry (xrefs_to 0x100261F0 = data 0x10034004). The DLL's virtual calls are 'mov reg,[reg+N]; call reg'; every '+30h]' operand in .text was listed (57 hits) and none loads a vtable slot for a call. The DLL has no other MIDI status compare (search for cmp/sub with 80h/90h/0B0h/0Fxh: only 0x1002621C, 0x10026228 are MIDI). So in 2.01 it is unreachable: [C] no reference; [G] a leftover of an earlier DLL-side remote control (its note 31 = Step forward agrees with the manual p. 202, unlike the exe's OB-25).
- Every other device's slot 12 is nullsub_4 0x10010380 (retn 0Ch): 24 vtable references, among them the 808 0x10034184 and the 909 0x100342C4.
- The 303 note-on 0x10023CD4 is called only from the step function 0x100253A0 (0x10025589); note-off 0x10023C74 only from the step function (0x10025442, 0x10025499, 0x1002556B) and slot 6 (0x1002563D). Nothing else can sound a 303 note.
- The exe (task-182, docs/standard_midi_mapping.md 1, 4..7): sub_42EAAF maps every note on the remote channel to a panel function (pattern selection, step programming, switches); none sounds a device.
- Manual p. 183 'About Latency': 'For this reason, you can not play the sounds in ReBirth RB-338 via MIDI. It would feel just awful.'
Conclusion: the original had no live play for any device (303, 808, 909). Per the task description, orb does not invent it without a Decide task.
Failed hypothesis: that sub_100261F0 is the 303's live MIDI note input (docs/initial_investigation.md 5.3 and task-185's facts say 'the 303's MIDI input'). It is a remote-control duplicate with no caller.

Backlog: created task-194 (Decide, @pingw33n, m-7: does orb add live play the original lacks; options A none / B the 303s on their own channels with the clone conventions / C B plus drums / D plugin only; recommendation B) and task-195 (@agent-complex, m-7, depends on task-194: the implementation, if chosen).
AC #3, #4 and #5 moved to task-195 (its AC #2, #3, #6; task-195 also requires the corpus to stay EQUAL and docs/midi.md section 4 to describe the addition): with no device playing live in the original there are no live notes to test for pitch, accent, slide, overlap, all-notes-off or a running pattern, no note that could hang, and nothing to listen to. They are removed here rather than left unchecked under Done (the same practice as moving release-only criteria to task-134). task-183 itself does not depend on task-194: its scope (find what the original did, implement what it had) is complete; task-195 carries the dependency.
Docs: docs/midi.md section 4 (new), intro; docs/standard_midi_mapping.md section 9 item 4 and section 10 (focus note); docs/initial_investigation.md 5.3 (the sub_100261F0 line corrected). No new defect of the original (sub_100261F0 is dead code, not a behavior), so docs/original_bugs.md is unchanged.
Code: crates/orb-player/src/midi.rs module doc; new unit test no_note_sounds_a_device (every note on/off on channels 2..16 and every note-off on channel 1 give nothing in both keyboard modes; a note-on on channel 1 gives at most one panel action and moves no device controller but the on switch (0) and the pattern (1)).
Not changed: task-185 (assigned to @pingw33n) still says 'The DLL has a 303 MIDI input (sub_100261F0)'; reported to the supervisor instead of edited.

Tests: cargo test (workspace, debug) 715 passed, 0 failed, 9 ignored (714 before + no_note_sounds_a_device); cargo test -p orb-player --lib midi 17 passed; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. No engine code changed (crates/orb-engine untouched), so the reference corpus is unaffected; corpus_golden not run.
AC #1: docs/midi.md section 4 (4.1 evidence, 4.2 what sub_100261F0 does with notes, velocity (never read), note-off (never tested), CCs, the running pattern (it only changes the live pattern/enable/step bytes; it plays nothing), 4.3 what orb does and the differences), each fact [C] with addresses, the one guess [G] (why the dead copy exists); the 808 and 909: slot 12 nullsub_4 (0x10034184, 0x100342C4).
AC #2: the original let no device play live, so no device plays from notes in orb either; the layer is task-182's, unchanged but for its doc; the test no_note_sounds_a_device checks it; no MIDI port, menu item or setting added (git diff main --stat: docs, midi.rs, backlog only).

Review round 1 (verdict merge, three doc precision fixes):
1. docs/midi.md section 4 intro: the sources narrowed: the DLL facts come from the DLL copy read here; the exe's note dispatch is task-182's reading, not re-read.
2. 'No caller of sub_100261F0' is now [G] in docs/midi.md 4.2 (heading, paragraph saying what was checked: IDA xrefs = only the vtable entry 0x10034004; the 57 '+30h]' operands; indexed/add/lea dispatch not ruled out), in 4.3, in docs/initial_investigation.md 5.3 and docs/standard_midi_mapping.md 9.4. What it does stays [C]; the no-live-play conclusion rests on the voice's callers (0x10023CD4 only from the step function), which stay [C].
3. docs/standard_midi_mapping.md section 7: note 31 = Back now cross-references section 9 item 4 (the DLL copy treats 31 as Step).
Docs only; commit amended.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: ReBirth 2.01 has no live play. No device (the two 303s, the 808, the 909) sounds from MIDI notes: every note on the remote channel is a Standard Mapping panel function (task-182), the 303's voice note-on 0x10023CD4 is called only by its step function [C], and the manual says so (p. 183, 'you can not play the sounds in ReBirth RB-338 via MIDI'). sub_100261F0, taken for 'the 303's MIDI input', is slot 12 of the CTB303Device vtable (0x10034004), a DLL-side copy of the 303's remote control (CCs 23..36, pattern selection notes, step editing at the cursor, note 31 = Step as the manual says, unlike the exe's OB-25) that plays nothing [C]; no caller of it was found [G]. The other devices' slot 12 is nullsub_4.

Following the task (no live play invented without a decision): task-194 (Decide, @pingw33n: options none / the 303s on their own channels with the clone conventions accent = velocity, slide = overlapping notes / plus drums / plugin only; recommendation: the 303s on their own channels, overriding the pattern while held, not recorded) and task-195 (the implementation, depends on task-194). AC #3..#5 (note-stream tests, no stuck notes, the listening check) moved to task-195, having nothing to test here.

Differences from the manual: none on live play (p. 183 and the binary agree). Since orb plays no device from notes either, there is no deliberate difference yet; task-195 adds one if task-194 chooses.

Docs: docs/midi.md section 4 (new); docs/standard_midi_mapping.md 7, 9.4 and 10; docs/initial_investigation.md 5.3 (the sub_100261F0 line corrected). Code: crates/orb-player/src/midi.rs module doc and the test no_note_sounds_a_device. No engine change; the corpus is unaffected.

Tests: cargo test 715 passed, 0 failed, 9 ignored; clippy and nightly fmt clean. Review round 1: three doc precision fixes (notes).
Branch worktree-agent-a9e284d08fcfa319f, one commit.
For the supervisor: task-185 (the user's) still states 'The DLL has a 303 MIDI input (sub_100261F0)', which this task refutes.
<!-- SECTION:FINAL_SUMMARY:END -->
