---
id: TASK-244
title: 'NVDA pass by the user: the new windows and names (night of 2026-11-27)'
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-27T21:55:00Z'
updated_date: '2026-11-28T07:03:00Z'
labels:
  - ui
  - tests
milestone: m-7
dependencies: []
priority: medium
type: task
ordinal: 258000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: TASK-197, the user's first NVDA pass, is Done. The tasks of the night of 2026-11-27 (task-237, task-239, task-234, task-238, task-235, task-236) make upstream's new windows and the classic skin usable with a screen reader. Agents cannot hear NVDA (the user's rule, see TASK-197), so each of those tasks verifies by machine (the AccessKit tree, the a11y tests) and adds here, as one acceptance criterion, what only listening with NVDA can confirm.

How: the user runs OpenRB-338 with NVDA in screen reader mode and goes through the criteria below; anything that sounds wrong becomes a bug task in m-7.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Task-237: Edit > Initialize Song from Pattern Mode in screen reader mode: NVDA reads the title and the original's text with Initialize when the question opens; Tab reaches Cancel; after either answer the focus is back where it was.
- [x] #2 By ear with NVDA (TASK-239): F6 through the sections says 303-1, 303-2, 808, 909 (not Bass Line 1/2, TR-808, TR-909); F2 on a drum grid starts with 808 or 909; Help > Keyboard Shortcuts in screen reader mode shows the same names
- [ ] #3 Options > Audio Output (task-234), by ear in NVDA: the window is announced as Audio Output and the focus lands on Driver; Driver, Device, Sample rate and Buffer size are each read with their name and the chosen value; Down/Up/Enter in an open list changes it and NVDA says what changed once (e.g. Sample rate: 48000 Hz); Tab reaches the In use lines and reads them as label: value; the numbers refreshing every 500 ms are never spoken; Escape closes an open list first, then the window, and the focus returns
- [x] #4 By ear with NVDA (TASK-238): with View > Classic in screen reader mode, F6 and Tab go through the sections and controls with the same names as the modern look; the pattern selectors, the drum grids (F2 reads them) and the 303 step editor work; the 808's round instrument knob is not read (Up/Down on the drum grid choose the instrument); View > Classic and View > Modern each say only the menu item and the focus stays on the control it was on
- [x] #5 By ear with NVDA (TASK-235): Edit > 303-1 Pattern Editor... (and Ctrl+E on a 303) in screen reader mode: NVDA says the window's title and lands on Pattern (A1); Tab reads Step editor with its step ('Step 1 of 16: rest, C'), Length, Shuffle, Transpose up and down, Shift left and right, Copy, Paste and Clear by name; on the Step editor Up/Down/Home/End are read once each and Left/Right say nothing; C on a rest says 'Step 1: rest, C' once, - and P are read once as the new value; Pattern and Length changes are read once; Transpose up says 'Pattern A1 transposed up'; F2 reads the pattern; while the song plays nothing is spoken; Escape closes the window and NVDA is back on the panel control.
- [x] #6 By ear with NVDA (TASK-236): Edit > 909 Pattern Editor... (and Ctrl+E on the 909's grid) in screen reader mode: NVDA says the window's title and lands on Pattern (A1); Tab reads Steps with its cell ('Bass drum, step 1, off'), Entry (Cycle), Length, Shuffle, Shift left and right, Copy, Paste and Clear by name; on Steps the arrows, Home and End read the new cell once; Space, Ctrl+Space, Shift+Space and Ctrl+Alt+Space read the cell's new state once (on, accent, flam); Entry Up/Down are read once ('High (accent)'); Shift right says 'Pattern A1 shifted right' (on an empty pattern 'Pattern A1 unchanged'); F2 reads the pattern; while the song plays nothing is spoken; Escape closes the window and NVDA is back on the panel control. The same in the 808 Pattern Editor (no Entry; Space toggles).
- [ ] #7 After a device-lost fallback in Options > Audio Output, NVDA reads the Device combo box as the device in use (System default), not the lost one (task-245)
- [ ] #8 By ear with NVDA (TASK-247): in screen reader mode, F10 then Right and Left on the menu names: NVDA says each name once, Right from Help says File and Left from File says Help, never a panel control; with a menu open, Right and Left close it and open the next or previous menu and NVDA says its first item once; in the Mods menu Right on a name that lists files opens the list and says its first file, Left closes it and says the name again (checklist item 36)
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The user's NVDA pass, reported 2026-11-28 (checklist numbers of docs/accessibility_plan.md Part 3):
- Heard working: items 23, 24, 27 and 28 (the older follow-ups, whose tasks are Done; no open task); 30 (criterion 2), 32 (criterion 4), 33 (criterion 5), 34 (criterion 6).
- Item 25 (Mods menu marks): not tried, the user found no mods installed (the Standard ReBirth entry is the one to hear then).
- Item 26 (Export format) and item 35 (criterion 7): not reported. In item 26's place the user reported a new fault of the menus (below).
- Item 29 (criterion 1, Initialize Song's question): not confirmed; the user's note was about File > New, which from the menu opens Save As at once with unsaved changes (Ctrl+N asks): task-249, which also checks Initialize Song chosen from the menu.
- Item 31 (criterion 3, Audio Output): the lists open only with Space, which the user wants said, and Escape out of a list loses the focus: task-248.
- New finding, reported in item 26's place: Left in an open menu returns to the bar and the other menus cannot then be reached; Right from Help leaves for the Transport section: task-247 (the user chose Windows' way).
Criteria 1, 3 and 7 stay open until their checks pass.
<!-- SECTION:NOTES:END -->
