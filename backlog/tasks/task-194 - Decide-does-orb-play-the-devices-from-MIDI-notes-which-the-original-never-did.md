---
id: TASK-194
title: >-
  Decide: does orb play the devices from MIDI notes, which the original never
  did
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-15T13:03:00Z'
updated_date: '2026-11-15T13:03:00Z'
labels:
  - decision
  - player
  - engine
milestone: m-7
dependencies: []
references:
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - task-183
  - task-185
priority: medium
type: spike
ordinal: 207000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-183 found (docs/midi.md section 4) that ReBirth 2.01 plays no device from MIDI notes: not the 303s, not the 808, not the 909. The manual says so (p. 183, 'About Latency': 'you can not play the sounds in ReBirth RB-338 via MIDI'), and the binary agrees: every note on the remote channel is a panel function of the Standard Mapping (pattern selection, step programming, switches; docs/standard_midi_mapping.md), the 303's note-on 0x10023CD4 is called only by its step function, and the DLL's sub_100261F0, long taken for 'the 303's MIDI input', is an unreachable DLL-side copy of the 303's remote control (pattern selection, step editing, its CCs), not live play. task-183's description says not to invent live play without a decision; this is that decision.

Why it matters: the user's plan is to use MIDI first inside the VST3/CLAP plugin (task-185), where patterns exported to the host as MIDI clips play back through the plugin as notes, and in the standalone app with a keyboard. A note that only selects patterns or programs steps (the original's behavior) does not play a clip's melody.

Constraint whatever is chosen: the same notes already mean something in the Standard Mapping on its channel (12..63 select patterns or program steps, 64..96 switch things). Live play therefore needs its own channel(s) or a mode, so it never collides with the Standard Mapping.

## Options
A. **No live play** (the original). Notes keep their Standard Mapping meaning only. The plugin exchanges patterns as pattern data or Standard Mapping notes, not as notes that sound.
B. **Live play of the 303s only, on their own channels** (an orb addition). Each synth listens to its own MIDI channel (for example synth 1 on channel 2, synth 2 on channel 3; the Standard Mapping stays on channel 1). The clone conventions (task-185's facts): pitch = note number (a range mapped onto the 303's note + octave up/down, with tune applied as for a step), gate = note length, accent = velocity at or above a threshold (for example 100), slide = a new note while the previous one is still held (legato, causal, no lookahead). Played notes override the running pattern's notes on that synth while held (the sequencer's steps are muted during a held note) or only play when the synth's pattern is off: a sub-choice.
C. **B plus the drums**: the 808 and 909 on their own channels, one note per instrument (a map to choose: General MIDI drums, 36 BD, 38 SD, 42 CH, 46 OH ..., or the panel's instrument order from a base note), velocity >= threshold = accent.
D. **B or C, plugin only**: the standalone app keeps the original's behavior.

Sub-choices for B/C: recording played notes into patterns (the original has step programming by MIDI, task-189, but no real-time note recording) — recommended: not recorded; all-notes-off / stop / song load / mod change release every held note.

## Recommendation
B, with the drums (C) as a later step only if the plugin needs them: the 303 is what DAW users play from clips, the conventions are settled by every clone, and separate channels keep the original's Standard Mapping intact. Played notes override the pattern while held. Not recorded. The output does not need to match the original (it has no such output), but it must be a deliberate, documented difference in docs/midi.md section 4.

The implementation follows in the dependent task; task-183's acceptance criteria #3..#5 (note-stream tests, no stuck notes, the manual VMPK/sendmidi check) moved there.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The decision (which devices play from notes, on which channels, the accent, slide and running-pattern rules, recording) is recorded in the final summary with its date
<!-- AC:END -->
