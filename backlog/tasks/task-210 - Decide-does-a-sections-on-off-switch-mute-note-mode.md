---
id: TASK-210
title: 'Decide: does a section''s on/off switch mute note mode?'
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-16T10:58:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies: []
references:
  - docs/plugin.md
  - docs/midi.md
  - crates/orb-engine/src/tb303.rs
priority: low
type: spike
ordinal: 221000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context (task-200, 2026-11-16): in the plugin's note mode (orb stopped, task-185) host notes on channels 1..4 play 303 #1, 303 #2, the 808 and the 909. Each of these sections has an On/off LED button (device controller 0, the live byte ENABLED; docs/ui_spec.md: 'switches the section off: the engine plays nothing for it from the next bar line, like an empty pattern'). In the original the switch is read only by the step function at a bar line (the latch +0xC95 / +0x18D1 / +0x18D4, then a disabled 303 step is a note-off and a disabled drum step plays nothing; tb303.rs/tr808.rs/tr909.rs step). Note mode has no bar line and no step, so task-185 and docs/midi.md 5 leave open whether a note sounds on a section that is switched off. The mixer channel's On/off (the mute) applies in both modes either way, since it is in the mixer.

Options:
A. Notes play whatever the section's switch says (the switch governs the pattern sequencer, as the 808's voice switches govern pattern mode only; the mixer mute silences a device). Provisional choice of task-200: no code reads the switch in note mode.
B. Notes on a section that is off are ignored (the switch as a per-device 'notes off'; a held 303 note gets its note-off when the switch turns off).
C. As B, but read at each note-on only (a held note sounds on).

Recommendation: A. The switch's original meaning is 'the pattern plays nothing'; in note mode no pattern plays, and the mixer channel's On/off already mutes a device in both modes. B or C would make a section switched off in a song silently swallow notes, a confusing state to debug in a DAW.

task-200 implements A provisionally (docs/plugin.md section 9).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The user's choice is recorded in the final summary with the date
- [ ] #2 If it differs from the provisional option A, a follow-up task in m-7 changes the note-mode code (Tb303/Tr808/Tr909 key_on) and docs/plugin.md section 9
<!-- AC:END -->
