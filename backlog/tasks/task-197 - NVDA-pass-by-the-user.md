---
id: TASK-197
title: NVDA pass by the user
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T22:00:00Z'
labels:
  - ui
  - tests
milestone: m-7
dependencies:
  - TASK-196
references:
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: task
ordinal: 220000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-13; the analysis is docs/accessibility.md. Read both before starting.

- What: the user goes through the checklist with NVDA on Windows and the release build, and
  files each problem as a task (labels ui, milestone m-7). The milestone is done when this task
  is done with no task left open.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each item of the checklist is marked as working or has a task filed for it.
- [ ] #2 With NVDA closed (no screen reader running), Options > Screen Reader Mode set to Automatic leaves the mode off (TASK-185's deferred check)
- [ ] #3 In screen reader mode, Save As, Export Loop as Audio File, Mods > Choose file and Options > Set Songs Folder / Set Mods Folder open the system dialog, NVDA reads it, and a file saved or exported through it is written as through the browser (TASK-194's deferred hand check)
- [ ] #4 In screen reader mode NVDA reads the section's name when Tab or F6 enters a section (the group name, or TASK-189's announcement), and reads the first line of Keyboard Shortcuts when F1 opens it (TASK-188's deferred by-ear check; the tree is tested headless)
- [ ] #5 NVDA speaks TASK-189's announcements (the status messages, a song opened, a mod chosen, the keyboard mode, the transport keys) and the F2/F3/F4 read-outs, a message repeated twice included (checklist item 21 of docs/accessibility_plan.md)
- [ ] #6 NVDA checklist item 22 (TASK-193's dialogs and windows): each window is announced as a dialog with its title and its first line is read; Tab reads the lines of About, Keyboard Shortcuts and Mod Info; the error box is announced as an alert and its message is read when the focus lands on OK (if not, the message needs App::announce: a follow-up task); the prompt is read with Save focused; Escape returns the focus.
- [ ] #7 TASK-190's step editor (NVDA checklist item 8, docs/ui_spec.md 20.5): on the focused Step editor each programming key's new state is spoken once (value change), the same note again is spoken (announcer), Up/Down/Home/End speak the new step and Left/Right nothing, in pitch mode a pitch key speaks the step it set and then the next step, neither cut off; and while a song plays in song mode nothing is spoken when it moves to another pattern, with Options > Step Editor Follows Playback off and on.
- [ ] #8 TASK-191's drum grids (NVDA checklist item 9, docs/ui_spec.md 20.6): NVDA speaks the cell on entering a grid and on each Up, Down, Left, Right, Home and End, and the cell's new state after each Space (its name changes while it keeps the focus); Ctrl+Space, Alt+Space and Ctrl+Alt+Space on the TR-909 give a note, an accent and a flam, and Shift+Space an accent without the system menu (TASK-210); and while a song plays in song mode nothing is spoken when the machine moves to another pattern, with Options > Step Editor Follows Playback off and on.
<!-- AC:END -->
