---
id: TASK-203
title: 'Plugin: drag a pattern button out to the host as a MIDI clip'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T09:43:00Z'
labels:
  - ui
  - player
milestone: m-7
dependencies:
  - TASK-185
  - TASK-199
  - TASK-201
references:
  - backlog task-185 (final summary)
  - docs/midi.md
  - docs/ui_spec.md
  - docs/song_load.md
priority: medium
type: feature
ordinal: 216000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Dragging a section's pattern button from the plugin editor (task-201) onto the DAW drops that section's pattern as a MIDI clip, as task-185's final summary decides. Drag only: no "Save MIDI file" fallback, no song or all-patterns export, no import. The user reviews the result before merge.

- **The clip** (a Standard MIDI File, type 0 or 1 with one track): in the note mapping of docs/midi.md (task-199): the section's channel (1..4: 303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4; task-207); 16th-note steps at the pattern's length; 303 pitch, slide as overlap, tie as one long note, accent as velocity; drums as GM notes, the 808 shared slots by the switch's current position, accent and the 909 levels as the documented export velocities (e.g. 40/80/104/127). Shuffle: decide straight or baked and document it. A test plays the clip back through note mode (task-200, if merged) or compares against the pattern's step data.
- **The drag:** a press-and-move on a pattern button (a click still selects the pattern; a movement threshold tells them apart) starts an OS drag of a temporary .mid file. egui/baseview cannot start an OS drag, so this is native code per platform: macOS (an AppKit dragging source with a file URL), Windows (OLE `DoDragDrop` with CF_HDROP), Linux (XDND). Clean up the temporary files.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The MIDI writer has tests for a 303 pattern (pitch, slide, tie, accent, gate), an 808 pattern (shared slots, AC) and a 909 pattern (four levels, flam)
- [ ] #2 Dragging a pattern button into Reaper on macOS creates a MIDI item with the expected notes; Windows and Linux drags are implemented and tested where a machine is available (else stated)
- [ ] #3 A click without movement still selects the pattern
- [ ] #4 The user reviewed the drag in a host and approved it before merge
- [ ] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
