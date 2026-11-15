---
id: TASK-183
title: 'MIDI live play: notes play the 303s (and the drums if the original did)'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-13T18:20:00Z'
updated_date: '2026-11-15T12:11:00Z'
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
- [ ] #1 docs/midi.md gains a live-play section: what the original's 303 MIDI input (sub_100261F0) does with notes, velocity, slide and a running pattern, and whether the 808 and 909 play from notes, each fact [C] with addresses or [G], plus where orb deliberately differs
- [ ] #2 MIDI notes play each device the original let play live, through the MIDI layer of task-182; no MIDI port, menu item or setting is added (the standalone app's port input is task-188's)
- [ ] #3 cargo tests feed timed note streams and check pitch, accent and slide, note-off, overlapping notes, all-notes-off and notes during a running pattern; no test needs a MIDI port
- [ ] #4 No note hangs after its note-off, a stop, a song load or a mod change
- [ ] #5 A listening check of the 303 played from a note stream rendered to a file is recorded in the notes
<!-- AC:END -->
