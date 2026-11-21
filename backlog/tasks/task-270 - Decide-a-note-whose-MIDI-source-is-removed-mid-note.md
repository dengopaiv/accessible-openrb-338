---
id: TASK-270
title: 'Decide: a note whose MIDI source is removed mid-note'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-21T10:36:00Z'
updated_date: '2026-11-21T10:36:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies: []
references:
  - docs/plugin.md
type: spike
ordinal: 257000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context (task-258): in the plugin's note mode, a host MIDI source removed while a 303 note is held (e.g. a REAPER send deleted mid-note) never delivers that note's note-off, and no more MIDI arrives from it. After task-258 the key no longer leaves the 303 sliding: a key is held once, and CC 123/120 and the host's stop let go of every held note. But the held note itself sounds on until the transport stops or an all-notes-off arrives, as in any MIDI instrument.

Options:
- A: keep it so: the note sounds until the host's stop or an all-notes-off / all-sound-off on its channel (standard MIDI behavior; the host's all-notes-off action clears it at once).
- B: also clear note mode's held notes at a host loop wrap or position jump (cuts a key a live player holds across the loop).
- C: a timeout: let go of a key held longer than some time (cuts long notes and drones played on purpose).

Recommendation: A.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-21 (relayed by the supervisor after the user's REAPER check of task-258): A, keep it as is. A note whose MIDI source is removed mid-note sounds until the host's transport stops or an all-notes-off / all-sound-off arrives on its channel; no clearing on loop wrap or position jump, no timeout. Documented in docs/plugin.md 9.1 and docs/plugin_usage.md 4 (task-258).
<!-- SECTION:FINAL_SUMMARY:END -->
