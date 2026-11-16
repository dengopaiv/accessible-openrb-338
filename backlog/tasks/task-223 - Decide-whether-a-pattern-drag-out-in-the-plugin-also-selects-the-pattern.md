---
id: TASK-223
title: 'Decide: whether a pattern drag-out in the plugin also selects the pattern'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-16T16:31:00Z'
updated_date: '2026-11-16T17:08:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies: []
references:
  - backlog task-203
  - docs/plugin.md section 12
priority: medium
type: spike
ordinal: 229000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Context.** task-203 lets the user drag a section's pattern button out of the plugin editor onto a DAW track as a MIDI clip. A press-and-move of 6 points or more (egui's click tolerance) starts the drag, and a click without movement selects the pattern. The plugin's pattern buttons act at mouse down, as the original's do (and as the standalone app does). So a drag-out also selects the pattern it drags: the press has selected it before the movement is seen. While the host plays in pattern mode, the section therefore switches to the dragged pattern at the next bar line. The host also records a parameter change of the section's pattern. task-203 keeps this behavior and documents it (docs/plugin.md 11). The task said only that a click still selects the pattern and that a movement threshold tells the click and the drag apart.

**Options.**
1. Keep it: the press selects at mouse down and a drag-out selects too. This is the original's timing. The lit button then shows which pattern was dragged. No change is needed.
2. In the plugin, select on mouse release when the press did not move (a click), and never on a drag-out. A drag-out then changes nothing in the instance. A click acts about 0.1 s later than now, and later than in the standalone and the original. In pattern mode the change takes effect at the next bar line anyway.
3. Select at mouse down, and on a drag-out put the previous pattern back. The host then sees two parameter changes (an automation glitch while recording), and the section may briefly switch.

**Recommendation.** Option 1: no surprise in the click's timing, and the selection change takes effect only at the bar line. Choose option 2 if dragging patterns out during playback without changing what plays matters more.

**Acceptance criteria for the follow-up, if option 2 or 3:** the plugin's pattern buttons behave as decided, the standalone app is unchanged, and tests/editor.rs covers both the click and the drag.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user (@pingw33n) on 2026-11-16: option 1, keep the current behavior. The plugin's pattern buttons act at mouse down as before, so a drag-out also selects the pattern it drags; a click without movement selects only. No follow-up change. Implemented as is by task-203 (docs/plugin.md section 12; section 11 when this task was written, before task-218's Gate Pattern Mode took that number).
<!-- SECTION:FINAL_SUMMARY:END -->
