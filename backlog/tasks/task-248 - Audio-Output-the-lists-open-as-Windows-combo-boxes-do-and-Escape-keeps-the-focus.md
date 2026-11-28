---
id: TASK-248
title: >-
  Audio Output: the lists open as Windows combo boxes do, and Escape keeps the
  focus
status: To Do
assignee: []
created_date: '2026-11-28T06:51:00Z'
updated_date: '2026-11-28T06:55:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-234
references:
  - crates/orb-ui/src/audio_window.rs
  - docs/ui_spec.md
priority: medium
type: bug
ordinal: 265000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: found by the user in the NVDA pass of 2026-11-28 (TASK-244, checklist item 31, task-234's check): "Comboboxes can be only opened with a space, needs to be marked. Pressing esc out of one loses focus." Asked what should change, the user chose all three (2026-11-28): Windows' combo keys, a spoken hint, and a line in Help > Keyboard Shortcuts. The lost focus is a bug either way.

Today: Options > Audio Output (crates/orb-ui/src/audio_window.rs, task-234 and task-245) has four egui::ComboBox lists, Driver, Device, Sample rate and Buffer size, named in screen reader mode by name_combo. egui opens a combo box with Space (its button's click); in screen reader mode Escape closes an open list first and the window on the next press (audio_window.rs around line 468), but after Escape closes a list the focus is lost instead of staying on that combo box.

Wanted, in screen reader mode (with the mode off the window stays as upstream's):
- Escape on an open list closes it and leaves the focus on its combo box, so NVDA reads the box again; a second Escape closes the window, and the focus returns to the panel control as now.
- Alt+Down and Enter open the focused box's list too (Space still does). The rest follows a Windows dialog's combo box (the user chose "Windows' combo keys", 2026-11-28): in an open list Up / Down move the highlight, Enter or Alt+Up close the list and choose the highlighted entry, and Escape closes it and keeps the choice as it was.
- Up / Down on a closed box choose the previous / next entry at once, and Home / End the first / last, as in a Windows dialog; each press applies the choice exactly as choosing it in the list does (a device or rate change restarts the stream then, as Windows applies each change), and is announced once, as a change through the list is (task-234's change text). No wrap at the ends.
- Each box's description says how to open it, read by NVDA after the value (for example "Alt+Down opens the list"; keep it short).
- Help > Keyboard Shortcuts in screen reader mode has a line for these keys.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In screen reader mode Escape on an open list closes it and the focus stays on its combo box (the tree's focused node); a second Escape closes the window and the focus returns; a11y tests
- [ ] #2 Alt+Down and Enter open a box's list, and Up / Down on a closed box change its choice once each, announced once, through the same path as a choice in the list; a11y tests (the window's test seam, App::audio_open_for_test)
- [ ] #3 Each box carries a description of how to open it, and Help > Keyboard Shortcuts lists the keys in screen reader mode; tests
- [ ] #4 With the mode off the window is as before (its tests pass unchanged); docs/ui_spec.md, CHANGELOG.md's next section, docs/accessibility_plan.md and docs/accessibility.md updated; one by-ear criterion added to TASK-244
<!-- AC:END -->
