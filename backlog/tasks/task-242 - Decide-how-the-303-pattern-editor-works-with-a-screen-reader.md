---
id: TASK-242
title: 'Decide: how the 303 pattern editor works with a screen reader'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-27T21:43:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
priority: medium
type: spike
ordinal: 258000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context: task-235. Upstream's 303 pattern editor windows (crates/orb-ui/src/pattern_editor.rs, editor_frame.rs; docs/ui_spec.md section 23) have a header (303-1/2, bank, pattern, Copy, Paste, Clear, Length, Shuffle, Transpose, Shift) and a painted 13-key piano roll over 16 steps with Up, Down, Accent and Slide rows, for the mouse. The panel already has a keyboard step editor in screen reader mode (a11y/step303.rs, task-190).

Options put to the user on 2026-11-27:
A. Whole window (recommended): the header's controls become normal buttons and fields, and the roll becomes a list of the 16 steps with the same keys as the panel's step editor (Up/Down move, the same programming keys, F2 reads the pattern).
B. Header only: the header's controls accessible; notes stay edited in the panel's step editor.
C. Go to the panel: in screen reader mode Ctrl+E and the menu items move the focus to the panel's step editor of that 303 instead of opening the window.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The user chose A, the whole window, on 2026-11-27: the header's controls as buttons and fields, and the roll as a list of 16 steps with the panel step editor's keys.
<!-- SECTION:FINAL_SUMMARY:END -->
