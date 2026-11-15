---
id: TASK-182
title: >-
  MIDI Standard Mapping: a source-independent layer from MIDI to the panel's
  controls
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-13T18:19:00Z'
updated_date: '2026-11-15T12:11:00Z'
labels:
  - re
  - player
  - docs
milestone: m-7
dependencies: []
references:
  - docs/initial_investigation.md
  - docs/song_load.md
  - docs/ui_spec.md
  - crates/orb-player/src/lib.rs
  - /Volumes/downloads/rebirth_2.01_manual.pdf
  - task-185
  - task-188
priority: medium
type: feature
ordinal: 198000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user decided (2026-11-13) to bring MIDI into scope, until now excluded by AGENTS.md ("MIDI and ReWire are out of scope for now"), in the new milestone m-7, which also holds the VST3/CLAP plugin (task-185). This task is the core of MIDI remote control: MIDI messages do what the mouse and keyboard do on the panel (turn knobs, flip switches, select patterns, switch devices on and off).

Why the core comes first (the user, 2026-11-15): MIDI input will first be used inside the plugin, as the way songs and patterns exported to the host as MIDI play back through it. The exported files use the mapping this task implements. Opening MIDI ports in the standalone app comes later, in task-188.

The original manual (`/Volumes/downloads/rebirth_2.01_manual.pdf`, or the copy on the CD) describes two MIDI mappings:
- **Standard Mapping**, predefined and unchangeable. This task implements it, and only it.
- **Quick Mapping**, a limited mapping the user changes in the UI. That is task-188's, not this task's.

What is known: the parameters `docs/song_load.md` marks as probably MIDI remote control (around line 413); the 303 waveform's CCs 23 and 30 (`sub_4405B5`, stores `v >= 64`, docs/ui_spec.md section 2); the controller table of the ReBirth 1.x MIDI-file import (`docs/song_load.md` section 12). In the DLL every panel controller message calls the record hook `sub_10012BC0` after its store (`docs/initial_investigation.md` section 5.3).

Reverse engineering is deliberately minimal: confirm the Standard Mapping (which channel addresses which device, which CC drives which controller and how its value is scaled, how patterns are selected, what other messages it reacts to), from the manual and the exe, with addresses checked against the disassembly. If the Standard Mapping depends on a setting of the original (a base channel, for example), use the original's default and add no UI for it; the setting belongs to task-188. MIDI-driven output does NOT have to match the original sample for sample; it should behave as a user of the original would expect. Write down any choice where the original is unclear.

The layer takes timed MIDI bytes and produces the player's commands, independent of where the bytes come from: the plugin will feed it the host's events with sample offsets, and task-188 will feed it a MIDI port. This task has no live MIDI source yet: the tests drive it with byte streams. No new UI: no menu items, no settings, no port handling, no MIDI LED. The user's IDA databases are read only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A new docs/midi.md documents the original's Standard Mapping (channels per device, the CC-to-controller map with value scaling, pattern selection, the other messages it handles, and any setting it depends on with its default), each fact marked [C] with addresses or [G]; it names Quick Mapping as task-188's and lists where orb deliberately differs
- [ ] #2 The MIDI-to-command layer is independent of the MIDI source, and cargo tests drive it with timed byte streams (no MIDI port) covering every mapped controller kind, pattern selection, channel filtering and out-of-range or unmapped messages
- [ ] #3 A control changed through the layer shows its new value on the panel, as a mouse change does, checked by a test in crates/orb-ui
- [ ] #4 No menu item, setting, MIDI port handling or MIDI LED behavior is added to the app
- [ ] #5 AGENTS.md's scope sentence is updated to say MIDI is in scope in m-7
<!-- AC:END -->
