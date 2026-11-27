---
id: TASK-243
title: 'Decide: how the 808 and 909 pattern editors work with a screen reader'
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
ordinal: 259000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context: task-236. Upstream's 808 and 909 pattern editor windows (crates/orb-ui/src/drum_editor.rs, editor_frame.rs; docs/ui_spec.md sections 24 and 25) have a header like the 303 editor's and a painted grid of 11 drum rows and the AC row (the 909's cells: off, low, high, flam, with an entry switch), for the mouse. The panel's drum machines already are a keyboard grid in screen reader mode (a11y/grid.rs, task-191).

Options put to the user on 2026-11-27:
A. Whole window (recommended): the header's controls become buttons and fields, and the grid uses the same keys as the panel's drum grid in screen reader mode (arrows, Home, End, Space with Ctrl and Alt).
B. Header only: the header's controls accessible; steps stay edited in the panel's grid.
C. Go to the panel: in screen reader mode Ctrl+E and the menu items move the focus to the panel's grid of that machine instead.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The user chose A, the whole window, on 2026-11-27: the header's controls as buttons and fields, and the grid with the panel drum grid's keys in screen reader mode.
<!-- SECTION:FINAL_SUMMARY:END -->
