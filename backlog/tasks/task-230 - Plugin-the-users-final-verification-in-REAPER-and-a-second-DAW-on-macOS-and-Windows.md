---
id: TASK-230
title: >-
  Plugin: the user's final verification in REAPER and a second DAW on macOS and
  Windows
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-17T07:48:00Z'
updated_date: '2026-11-17T11:09:00Z'
labels:
  - player
  - ui
  - tests
milestone: m-7
dependencies:
  - TASK-225
  - TASK-214
  - TASK-221
  - TASK-228
  - TASK-229
  - TASK-235
priority: high
type: task
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's own check of the finished plugin before the first release run (task-215), on macOS and Windows, in REAPER (VST3 and CLAP) and one other popular DAW. Done after the remaining plugin tasks (225, 214, 228, 229, 221) are merged. Any defect found becomes its own bug task, which task-215 then waits for too.

Areas to cover: insert and remove, the editor and its resizing, the three modes (Note, Latch Pattern, Gate Pattern) with host play/stop/loop/seek, MIDI clips and live input, the note at the timeline start, rendering, saving and reloading a project (state, mode, mod), automation of the parameters, drag-out of a pattern as a MIDI clip, File > Open/New and the Mods menu, Help > Plugin Usage, several instances at once.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 REAPER on macOS: VST3 and CLAP checked.
- [ ] #2 REAPER on Windows: VST3 and CLAP checked.
- [ ] #3 The second DAW on macOS checked.
- [ ] #4 The second DAW on Windows checked.
- [ ] #5 Defects found are recorded as their own tasks and task-215 depends on them.
<!-- AC:END -->
