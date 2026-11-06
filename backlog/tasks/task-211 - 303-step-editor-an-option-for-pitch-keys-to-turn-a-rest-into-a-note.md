---
id: TASK-211
title: '303 step editor: an option for pitch keys to turn a rest into a note'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-06T23:17:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/a11y/step303.rs
  - crates/orb-ui/src/settings.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 229000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user (NVDA, 2026-11-07) entered notes on the 303 step editor and the steps still read "rest". As in the original and the real TB-303, a pitch key sets only the step's note (`sub_43F590`, docs/ui_spec.md section 5 "Keys"); the step stays a rest until Note/Pause (`-` or `=`) flips it, and a new or cleared pattern is all rests. By ear this is slow and easy to miss: every note needs a second key, step by step, after the notes are typed. The user asked for an option that makes a pitch key also turn a rest into a note.

What: a new setting under Options, next to "Step Editor Follows Playback", e.g. "Pitch Keys Make Notes" (the exact label to settle in the task, consistent with the other Options items), off by default, effective only in screen reader mode. When it is on and screen reader mode is on, a pitch key on a 303 (in and out of pitch mode, keyboard or the panel's note keys) that lands on a rest also clears the rest: the model sets `rest = false` and emits the Step303 event for field 5 (value 0) besides the note's field 0 event, so the engine plays it. A step that is already a note only changes its note. `-` / `=` still toggle the rest as before, so a rest can still be made. The step editor's spoken answer is the step's new state as now (e.g. "Step 1: D" instead of "Step 1: rest, D"); in pitch mode the step it set is spoken, then the next step, as now. With the setting off, or screen reader mode off, every key behaves exactly as upstream.

Code: crates/orb-ui/src/model.rs `press_303` (the `key*` branch), crates/orb-ui/src/settings.rs and menu.rs (the setting, as `step_editor_follows`), a11y/step303.rs if the spoken text needs it, key_help.rs (the step editor rows), docs/ui_spec.md 20.5 (the bullet and the how-to "Programming a 303 pattern with a screen reader", steps 3 and 6), the README's screen reader section, docs/accessibility_plan.md. The by-ear check goes to TASK-197 (the supervisor adds it at merge).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 With the setting on and screen reader mode on, a pitch key on a rest step sets the note and clears the rest (the model's step and the Step303 events for fields 0 and 5 sent to the engine), in and out of pitch mode; a test on a real Link and engine shows the step plays.
- [ ] #2 A pitch key on a step that is already a note changes only its note; - / = still toggle the rest; Clear still makes every step a rest.
- [ ] #3 With the setting off, or screen reader mode off, a pitch key leaves the rest as upstream does (test).
- [ ] #4 The setting is saved and restored with the other settings, is in the Options menu with its state, and is documented in docs/ui_spec.md 20.5, the key help and the README.
<!-- AC:END -->
