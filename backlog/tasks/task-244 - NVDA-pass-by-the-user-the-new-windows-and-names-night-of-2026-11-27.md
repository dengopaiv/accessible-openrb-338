---
id: TASK-244
title: 'NVDA pass by the user: the new windows and names (night of 2026-11-27)'
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-27T21:55:00Z'
updated_date: '2026-11-27T22:19:00Z'
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
<!-- AC:END -->
