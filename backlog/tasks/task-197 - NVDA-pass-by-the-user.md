---
id: TASK-197
title: NVDA pass by the user
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T08:00:00Z'
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
<!-- AC:END -->
