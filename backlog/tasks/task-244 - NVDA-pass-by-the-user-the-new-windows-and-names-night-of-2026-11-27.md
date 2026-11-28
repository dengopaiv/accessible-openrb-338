---
id: TASK-244
title: 'NVDA pass by the user: the new windows and names (night of 2026-11-27)'
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-27T21:55:00Z'
updated_date: '2026-11-28T11:05:00Z'
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
- [x] #8 By ear with NVDA (TASK-247): in screen reader mode, F10 then Right and Left on the menu names: NVDA says each name once, Right from Help says File and Left from File says Help, never a panel control; with a menu open, Right and Left close it and open the next or previous menu and NVDA says its first item once; in the Mods menu Right on a name that lists files opens the list and says its first file, Left closes it and says the name again (checklist item 36)
- [ ] #9 By ear with NVDA (TASK-248): in Options > Audio Output in screen reader mode, NVDA reads each of Driver, Device, Sample rate and Buffer size with its value and then 'Alt+Down opens the list'; Alt+Down, Enter and Space open the list; Up and Down in the open list say each entry once and choose nothing; Enter or Alt+Up choose the highlighted entry, said once ('Sample rate: 48000 Hz'), and NVDA reads the box again; Escape closes the list, keeps the choice and NVDA reads the box again, a second Escape closes the window and the focus returns; on a closed box Up, Down, Home and End choose the previous, next, first and last entry at once, each said once; Help > Keyboard Shortcuts in screen reader mode has the line for these keys (checklist item 37)
- [ ] #10 By ear with NVDA (TASK-249): in screen reader mode, File > New chosen from the menu with unsaved changes (F10, Down, Enter) opens the question 'Save changes to ...?' and NVDA reads it, with the focus on Save: nothing is saved and no Save As opens until you answer (S or Enter on Save, D, or Escape); the same with Edit > Initialize Song from Pattern Mode (in song mode): NVDA reads the question and nothing is initialized until you press Enter on Initialize or Escape; Ctrl+N asks as before (checklist item 38)
- [ ] #11 By ear with NVDA (TASK-250): in screen reader mode, in pattern mode, Up and Down in File and Edit reach Export Loop as Audio File, Initialize Song and Initialize Loop from Pattern Mode and Undo; NVDA reads each as unavailable; Enter or Space says why once ('Needs song mode; F12 switches', 'Nothing to undo'), runs nothing and the menu stays open on the item; after F12 the items are read without unavailable and run (checklist item 39)
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

TASK-248 (this task's agent): criterion 9 added, checklist item 37; item 31 (criterion 3) stays open to be heard again.

The user's second report, 2026-11-28 (checklist numbers of docs/accessibility_plan.md Part 3):
- Item 25 (Mods menu marks): works, heard with no mods installed (the Standard ReBirth entry).
- Item 26 (Export format): the item could not be found. File > Export is off in pattern mode, and egui skips an item that is off, so NVDA never reached it: task-250. Item 26 waits, to be heard in song mode (F12).
- Item 29 (criterion 1): the Initialize Song item could not be found, for the same reason: task-250. Waits, in song mode.
- Item 31 (criterion 3): Driver set to ASIO said "Audio output: the output stream: sample clock or rate cannot be determined or is not present; using the System default with the default settings". ASIO's System default is the first installed driver, the MiniFuse ASIO Driver, whose device is not connected (ASE_NoClock): task-251. The rest of item 31 was not reported and waits.
- Item 36 (criterion 8): works.
- Item 38 (criterion 10): File > New works; the Initialize Song half could not be found (task-250). Criterion 10 stays open for that half.
- Items 35 and 37 (criteria 7 and 9): not reported.
The user chose Windows' way for items that are off (reached by Up and Down, read as unavailable, Enter says why), recorded in task-250. The checklist's items 26, 29 and 38 now say to press F12 for song mode first.
<!-- SECTION:NOTES:END -->
