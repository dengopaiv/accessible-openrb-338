---
id: TASK-244
title: 'NVDA pass by the user: the new windows and names (night of 2026-11-27)'
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-27T21:55:00Z'
updated_date: '2026-11-27T23:05:00Z'
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
- [ ] #2 By ear with NVDA (TASK-239): F6 through the sections says 303-1, 303-2, 808, 909 (not Bass Line 1/2, TR-808, TR-909); F2 on a drum grid starts with 808 or 909; Help > Keyboard Shortcuts in screen reader mode shows the same names
- [ ] #3 Options > Audio Output (task-234), by ear in NVDA: the window is announced as Audio Output and the focus lands on Driver; Driver, Device, Sample rate and Buffer size are each read with their name and the chosen value; Down/Up/Enter in an open list changes it and NVDA says what changed once (e.g. Sample rate: 48000 Hz); Tab reaches the In use lines and reads them as label: value; the numbers refreshing every 500 ms are never spoken; Escape closes an open list first, then the window, and the focus returns
- [ ] #4 By ear with NVDA (TASK-238): with View > Classic in screen reader mode, F6 and Tab go through the sections and controls with the same names as the modern look; the pattern selectors, the drum grids (F2 reads them) and the 303 step editor work; the 808's round instrument knob is not read (Up/Down on the drum grid choose the instrument); View > Classic and View > Modern each say only the menu item and the focus stays on the control it was on
<!-- AC:END -->
