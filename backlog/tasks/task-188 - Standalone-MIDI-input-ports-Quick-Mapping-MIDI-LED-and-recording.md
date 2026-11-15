---
id: TASK-188
title: 'Standalone MIDI input: ports, Quick Mapping, MIDI LED and recording'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-15T12:11:00Z'
labels:
  - re
  - ui
  - player
  - docs
milestone: m-7
dependencies:
  - TASK-182
references:
  - docs/midi.md
  - docs/ui_spec.md
  - docs/initial_investigation.md
  - crates/orb-ui/src/menu.rs
  - /Volumes/downloads/rebirth_2.01_manual.pdf
  - task-182
priority: low
type: feature
ordinal: 198500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Split from task-182 by the user on 2026-11-15. task-182 implements the original's MIDI **Standard Mapping** as a source-independent layer (timed MIDI bytes to player commands), with no UI and no MIDI source; the plugin feeds it the host's events. This task makes MIDI usable in the standalone app (OpenRB-338): reading a MIDI port and the parts of the original's MIDI that need UI.

The original manual (`/Volumes/downloads/rebirth_2.01_manual.pdf`, or the copy on the CD) describes two mappings: Standard Mapping (predefined, task-182) and **Quick Mapping**, a limited mapping the user changes in the UI. This task adds Quick Mapping, as close to the original's as the minimal reverse engineering shows (what can be mapped, how it is set in the UI, how it combines with the Standard Mapping, whether and where it is saved), together with any MIDI setting the Standard Mapping depends on (task-182 used its default). Where the original keeps these in a menu or dialog, orb's follows it; the result goes into docs/midi.md and docs/ui_spec.md.

Also here: the panel's MIDI LED (docs/ui_spec.md section 3, drawn dark today) lights on input as in the original; a control moved by MIDI in song mode with record on is recorded like a mouse move (the original's controller messages all call the record hook `sub_10012BC0`, docs/initial_investigation.md 5.3).

The MIDI output does not have to match the original sample for sample; it should behave as a user of the original would expect. Development and testing happen on a Mac with no MIDI hardware: virtual CoreMIDI ports, the IAC Driver, `sendmidi`/`receivemidi`, MIDI Monitor. The user's IDA databases are read only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The app lists the MIDI inputs (including a virtual port it creates on macOS), lets the user choose one, remembers the choice, and survives a port that disappears or fails to open (an error box, no panic)
- [ ] #2 Quick Mapping works as docs/midi.md describes the original's (what can be mapped, the UI that sets it, how it combines with the Standard Mapping, how it is saved), each fact [C] with addresses or [G]
- [ ] #3 Any setting the Standard Mapping depends on is available in the UI the original uses for it, and docs/ui_spec.md describes the new UI
- [ ] #4 The panel's MIDI LED lights on input as in the original, and docs/ui_spec.md no longer calls it out of scope
- [ ] #5 In song mode with record on, a control moved by MIDI is recorded like a mouse move, checked by a test
- [ ] #6 cargo tests cover Quick Mapping and the port-to-layer path without a real port; a manual check with a virtual port and sendmidi on macOS is recorded in the notes with the commands used
<!-- AC:END -->
