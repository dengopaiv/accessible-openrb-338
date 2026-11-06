---
id: TASK-209
title: 'Decide: the 303 step editor during playback, and its arrow keys'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-06T21:22:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - crates/orb-ui/src/a11y/step303.rs
priority: medium
type: spike
ordinal: 227000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Raised by the review of TASK-190 (the 303 step editor, spoken), 2026-11-06.

Context: in screen reader mode a 303's "Step editor" is a focusable node whose value is the edit step and its contents, read from the row's pattern. During playback (song mode, or a pattern change waiting for the bar's end) the engine moves the row to another pattern (Link::follow_values, Panel::adopt_pattern), so the focused editor's value changed with no key pressed and NVDA spoke during playback, against TASK-189's rule that nothing speaks by itself while it plays. The review also asked what Up and Down should do on the editor: the first version made Left/Down the panel's Back and Right/Up its Step, reading decision TASK-182 (A11Y-D6, "Up and Down change the focused value").

Questions put to the user:
1. Should the step editor follow the playing pattern ("move-along"), and should it speak when it does?
2. On the step editor, what do Up/Down and Left/Right do?

Options given: keep the editor on the pattern being edited until the user acts; follow playback; speak the change or not. Recommendation: never speak a change the user did not make.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by @dengopaiv on 2026-11-06, in the user's words:
- "Move per pattern or not should be a thing. And even then, nvda shouldn't announce. It's a good question, but it has been a standard that in accessible programs, movealong doesn't speak. Then the pattern can be checked with a key or something when necessary."
- "If there's nothing for the left and right to do, the concensus has usually been that up and down should move on steps, and left and right do nothing."

Outcome, applied in TASK-190 (docs/ui_spec.md 20.5):
- Move-along is a setting, Options > Step Editor Follows Playback (screen reader mode only; Settings::step_editor_follows, off by default). The editor edits its own pattern (Synth303::pin, kept by App::pin_303): off, it stays on the pattern the user last chose, however playback moves on; on, it moves to the playing pattern at the next input. Either way the move is never spoken. F2 reads the pattern and says "Step editor on pattern A1" when the editor is not on the one playing.
- On the step editor Up is the previous step (Back) and Down the next (Step), Home/End step 1 and the pattern's last step; Left and Right do nothing.
- The same silent move-along rule applies to TASK-191's drum grids.
<!-- SECTION:FINAL_SUMMARY:END -->
