---
id: TASK-269
title: >-
  Plugin Gate Pattern Mode: all-notes-off (CC 123/120) lets go of the held gate
  notes
status: To Do
assignee: []
created_date: '2026-11-21T10:09:00Z'
labels:
  - player
milestone: m-7
dependencies:
  - TASK-258
references:
  - crates/orb-plugin/src/core.rs
  - docs/plugin.md
priority: low
type: bug
ordinal: 256000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Found while fixing task-258 (note mode's 303 stuck sliding after a host lost a note-off). In Gate Pattern Mode (task-218, docs/plugin.md section 11) the plugin keeps the notes 0..31 held per section channel (core.rs `Held`, `gate_note`); a note-off the host loses (a REAPER MIDI send removed while a gate note is held) leaves that section's gate open, so the section keeps playing its pattern, and with the host stopped a held gate note even keeps the song running free. `Held::press` already holds a key once, so the gate heals at that key's next note-off, but the plugin ignores All Notes Off (CC 123) and All Sound Off (CC 120) there; task-258 honours them only in note mode (`Engine::message`, `Command::NotesOff`).

Make CC 123 and CC 120 on channel 1..4 in Gate Pattern Mode forget that section's held gate notes and close its gate (`send_gate`), as a note-off of the last held note does. Whether the host's stop should also close the gates is a product question (by design a held gate note plays with the host stopped, docs/plugin_usage.md 3): leave the stop alone unless the user decides otherwise (a `Decide:` task if needed).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In Gate Pattern Mode, CC 123 and CC 120 on a section's channel close that section's gate as the note-off of its last held note would, bit for bit, in a core test
- [ ] #2 Other channels and the other modes are unchanged; docs/plugin.md section 11 describes it
<!-- AC:END -->
