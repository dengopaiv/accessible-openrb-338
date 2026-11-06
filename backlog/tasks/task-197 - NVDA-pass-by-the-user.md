---
id: TASK-197
title: NVDA pass by the user
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T23:22:00Z'
labels:
  - ui
  - tests
milestone: m-7
dependencies:
  - TASK-196
  - TASK-195
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
- [x] #7 TASK-190's step editor (NVDA checklist item 8, docs/ui_spec.md 20.5): on the focused Step editor each programming key's new state is spoken once (value change), the same note again is spoken (announcer), Up/Down/Home/End speak the new step and Left/Right nothing, in pitch mode a pitch key speaks the step it set and then the next step, neither cut off; and while a song plays in song mode nothing is spoken when it moves to another pattern, with Options > Step Editor Follows Playback off and on.
- [x] #8 TASK-191's drum grids (NVDA checklist item 9, docs/ui_spec.md 20.6): NVDA speaks the cell on entering a grid and on each Up, Down, Left, Right, Home and End, and the cell's new state after each Space (its name changes while it keeps the focus); Ctrl+Space, Alt+Space and Ctrl+Alt+Space on the TR-909 give a note, an accent and a flam, and Shift+Space an accent without the system menu (TASK-210); and while a song plays in song mode nothing is spoken when the machine moves to another pattern, with Options > Step Editor Follows Playback off and on.
- [ ] #9 TASK-195's mod chooser (NVDA checklist item 12, docs/ui_spec.md 20.7): Ctrl+M speaks the dialog Choose Mod and the filter Filter mods as an edit field; typing narrows the list and the N of M mods line is readable; Down from the filter speaks Mods, tree view and the selected item with its level and position; Right speaks expanded and then the first file, Left goes back; Up/Down speak each item with has demo and current; Enter chooses and Mod X is spoken; Ctrl+I opens Mod Info and Escape returns to the chooser; Tab / Shift+Tab move between the two stops; Escape closes and the focus returns where it was.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The user's marks so far (2026-11-07, read from their uncommitted edit of docs/accessibility_plan.md "The NVDA checklist"; (x) works, ( ) not working or not yet passed, no mark not yet tried):
- (x) 1 start: title spoken, focus on the Transport's first control.
- (x) 2 F6 / Shift+F6 through the sections.
- (x) 3 Tab through the Transport: name, role, value.
- ( ) 4 tempo with Up / Down and Page Up / Page Down: the new tempo spoken. Open; what failed is to be asked.
- no mark on 5 (F5, Space on a toggle, F9 / F11 / F12 spoken); the user's note on it: "Loop is F11. F12 is song and pattern."
- (x) 6 F3 while playing and stopped.
- (x) 7 Bass Line 1: cutoff by arrows and NVDA's slider commands, waveform, mute, pattern B3.
- (x) 8 the 303 step editor (AC #7). Found along the way: a pitch key leaves a rest a rest, as in the original; the user asked for an option, TASK-211.
- (x) 9 the drum grids (AC #8).
- 10 to 22 not yet tried.

Item 4 (tempo with Up / Down, Page Up / Page Down): works (the user, 2026-11-07, "Tempo works."); the empty mark meant not yet marked.

Item 5 (F5, Space on a toggle, F9 / F11 / F12 spoken): works (the user, 2026-11-07: "It works as expected"); the note on it was the user's reminder of which mode records, not a defect. Items 1 to 9 pass by ear; 10 to 22 not yet tried.
<!-- SECTION:NOTES:END -->
