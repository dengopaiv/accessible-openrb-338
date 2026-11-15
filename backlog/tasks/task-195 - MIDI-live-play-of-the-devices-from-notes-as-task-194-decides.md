---
id: TASK-195
title: MIDI live play of the devices from notes, as task-194 decides
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-15T13:03:00Z'
updated_date: '2026-11-15T13:03:00Z'
labels:
  - player
  - engine
  - docs
milestone: m-7
dependencies:
  - TASK-194
references:
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - crates/orb-player/src/midi.rs
  - crates/orb-engine/src/tb303.rs
  - task-183
priority: medium
type: feature
ordinal: 208000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-183 established that ReBirth 2.01 plays no device from MIDI notes (docs/midi.md section 4), so live play is an orb addition, decided in task-194. This task implements what task-194 decides; if it decides option A (no live play), close this task as not needed, with a final summary saying so.

Build on task-182's source-independent layer, crates/orb-player/src/midi.rs (Parser, StandardMapping, Timed<Action>): live-play notes must not collide with the Standard Mapping's notes (they are on the remote channel), so they come on the channel(s) task-194 sets. Add the engine entry points the voices need (for the 303: a held note that sets the pitch target, the gate, accent and slide the way the step function's note-on 0x10023CD4 and note-off 0x10023C74 do, docs/initial_investigation.md 6.1 and 6.4) without changing any existing engine path: the reference corpus must stay EQUAL (cargo test --release -p orb-engine --test corpus_golden). Output need not match the original sample for sample (it has no such output); it must sound as a user expects: correct pitch, accent, slide, envelopes and timing (apply at the event's 64-frame block).

These acceptance criteria moved here from task-183 (#3..#5), where there was nothing to test. The standalone app's MIDI port is task-188's; until it lands, the manual check can use a small dev harness or wait for task-188 (say which in the notes). Testing is on a Mac with no MIDI hardware (a virtual CoreMIDI port, VMPK or sendmidi). docs/midi.md section 4 gains orb's live play, as a deliberate difference from the original.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 MIDI notes play each device task-194 chose, through the MIDI layer of task-182, on the channels task-194 sets, without changing what the Standard Mapping does on its channel
- [ ] #2 cargo tests feed timed note streams and check pitch, accent and slide, note-off, overlapping notes, all-notes-off and notes during a running pattern; no test needs a MIDI port (from task-183 AC #3)
- [ ] #3 No note hangs after its note-off, a stop, a song load or a mod change (from task-183 AC #4)
- [ ] #4 The reference corpus stays EQUAL (cargo test --release -p orb-engine --test corpus_golden), stated in the commit message
- [ ] #5 docs/midi.md section 4 describes orb's live play as a deliberate difference from the original
- [ ] #6 A listening check of the 303 played from a note stream rendered to a file is recorded in the notes (from task-183 AC #5)
<!-- AC:END -->
