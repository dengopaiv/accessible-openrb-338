---
id: TASK-211
title: '303 step editor: an option for pitch keys to turn a rest into a note'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-06T23:17:00Z'
updated_date: '2026-11-07T08:55:00Z'
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
- [x] #1 With the setting on and screen reader mode on, a pitch key on a rest step sets the note and clears the rest (the model's step and the Step303 events for fields 0 and 5 sent to the engine), in and out of pitch mode; a test on a real Link and engine shows the step plays.
- [x] #2 A pitch key on a step that is already a note changes only its note; - / = still toggle the rest; Clear still makes every step a rest.
- [x] #3 With the setting off, or screen reader mode off, a pitch key leaves the rest as upstream does (test).
- [x] #4 The setting is saved and restored with the other settings, is in the Options menu with its state, and is documented in docs/ui_spec.md 20.5, the key help and the README.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Settings::pitch_keys_make_notes (ui.toml) + Options checkbox next to Step Editor Follows (screen reader mode only). 2. Panel::pitch_makes_notes set each frame by App::pin_patterns (mode && setting); press_303 key branch clears the rest and emits Step303 field 5 value 0 before field 0. 3. Key help row, ui_spec 20.5 and menu table, README, plan item 24. 4. Tests: wired.rs on real Link and engine, a11y_step303, settings, menu bar.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a1c12fe70006da6df. User decision 2026-11-07: the Options item is labeled "Pitch Keys Make Notes"; off by default, effective only in screen reader mode.

Done: code in model.rs press_303, app.rs pin_patterns, settings.rs, menu.rs (Cmd::TogglePitchKeysMakeNotes), key_help.rs (informational row in the screen reader group). Event order: field 5 then field 0. Spoken text needed no change (press_effect speaks the step's new state). cargo test --release -p orb-ui --no-fail-fast: all pass except crash.rs a_deliberate_panic_exits_101_and_writes_the_log (known, TASK-200). clippy -D warnings and nightly fmt --check clean. New tests: wired.rs pitch_keys_make_notes_on_the_real_engine (engine pattern bytes show steps 1 and 2 with rest 0), a11y_step303.rs pitch_keys_make_notes_is_spoken_and_only_in_screen_reader_mode, settings round trip, a11y_menu_bar checkbox in SR mode only. Plan: NVDA checklist item 24 added (unchecked); TASK-211 left in the To do table for the supervisor.

Merged by the supervisor 2026-11-07: review merge on the first round. wired (24), a11y_step303 (6), a11y_menu_bar (13) and keyboard (12) pass on main after the squash.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Options > Pitch Keys Make Notes (off by default, screen reader mode only, saved as pitch_keys_make_notes in ui.toml). On, a 303 pitch key on a rest (keyboard or panel key, time or pitch mode) sets the note and clears the rest, sending Step303 field 5=0 then field 0; verified on a real Link and engine (wired.rs). A step already a note only changes its note; -/= and Clear unchanged; setting off or mode off leaves the rest (tests). Documented in ui_spec 20.5 and the Options table, key help, README, plan (NVDA item 24). Tests: orb-ui all pass except known crash.rs (TASK-200); clippy and fmt clean. Branch worktree-agent-a1c12fe70006da6df, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
