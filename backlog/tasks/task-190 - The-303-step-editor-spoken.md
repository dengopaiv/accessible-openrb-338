---
id: TASK-190
title: The 303 step editor, spoken
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T21:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-188
  - TASK-189
  - TASK-209
references:
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/keys.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 213000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-6; the analysis is docs/accessibility.md. Read both before starting.

- What: in each 303 section, a "Step editor" node (focusable) whose value is the step being
  edited ("Step 5 of 16: C sharp, up, accent, slide", "Step 6: rest") and the editor's mode
  (pitch or time mode as the original has them). After each programming key, the step's new
  state is spoken (the node's value changes; if NVDA does not speak value changes of the
  focused node reliably, the announcer). Left and Right move the edit step where the original
  allows (`move_edit`), Home and End to the first and last step. F2 on a 303 section reads the
  whole pattern, one step after another, compactly ("1 C accent, 2 rest, 3 D sharp slide,
  ..."), in the pattern's length.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: program a pattern with the keys alone, read it back with F2, and compare with the pattern written by the mouse in the same steps: equal events and equal text.
- [x] #2 The note names follow the original's keyboard (C to high C, with "sharp"), and up/down are spoken as octave up/down.
- [x] #3 A short how-to in docs/ui_spec.md: programming a 303 pattern with a screen reader.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. New module crates/orb-ui/src/a11y/step303.rs: note names (C .. B, high C, "sharp"), the step text ("C sharp, octave up, accent, slide"; a rest "rest, C sharp" with its flags), the editor's value ("Step 5 of 16: ..."), the F2 pattern text ("1 C accent, 2 rest, ..." over the pattern's length), and what a programming press announces.
2. The Step editor node: in screen reader mode the 303's edit step display (tb303_N.edit_step) becomes a focusable Slider named "Step editor" whose value is the edit step and its contents, plus the mode (pitch mode); in the focus order where the display is (after the knobs, before Step / Back). With the mode off the display stays the label it is.
3. Keys on the focused step editor (screen reader mode, dispatched only when the focus is a step editor, before the value keys): Left / Right = Back / Step (Panel::press of tb303_N.back / step, move_edit, wraps modulo 16 as the original), Home / End = the first step / the pattern's last step (by Back / Step presses, no engine event). Up / Down: decided by what D6 says ("change the focused value"); see notes.
4. Speech after each programming key (act, Space on a 303 button): the announcer says the step the key changed ("Step 5: D sharp") unless the step editor has the focus, its step did not move and its value changed (then the value change speaks, as task-189 does for focused controls). Pitch mode advance: the programmed step is announced (the value change then gives the next step). Clear and the pitch mode button are announced.
5. F2 on a 303 section: the row read-out followed by the steps, through a separate function read_303_pattern (minimal touch of readout_key).
6. Tests: unit tests of the texts; tests/a11y_step303.rs: program a pattern with keys only (Program Synth, step editor focused) vs the same steps by mouse clicks: equal Panel events and equal F2 text; Left/Right/Home/End; the spoken texts; the tree (role, name, value). Update a11y_baseline.rs and a11y_sections.rs (focus order 192), uia_tree numbers in docs/accessibility.md 6.3.
7. Docs: ui_spec.md section 10 (Left/Right row of the screen reader table), 20.x (the step editor) with a how-to for programming a 303 with a screen reader; key_help.rs; accessibility.md 3.3/3.4 done notes; accessibility_plan.md NVDA checklist item for by-ear checks (TASK-197).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa4eb387f6da39059

Research and choices (2026-11-06):
- Model: Synth303 { edit, pitch_mode, patterns }, press_303 (model.rs ~760): a pitch key sets the edit step's note and, in pitch mode, moves the edit step on (move_edit(1)); Step/Back = move_edit(+1/-1) modulo 16, no engine event; pitch_button on resets edit to 0. Step303 default = rest on C.
- Node: the edit step display tb303_N.edit_step becomes, in screen reader mode only, a focusable Slider (horizontal) named "Step editor" (same egui id as the display's hit area, so nav::order places it after the 303's knobs). No numeric value: a RangeValue would raise a second value change (UIA_RangeValueValuePropertyId besides ValueValue) per move. Mode off: the label "Edit step N" as before (a11y_names/a11y_drive tests unchanged).
- The value carries the mode as ", pitch mode" at the end while on (the original has only the Pitch mode button, on/off; "time mode" is not a word of ReBirth's panel, so off says nothing).
- Rests: "rest, C" (the note and flags kept), because a pitch key on a rest changes only the note; "Step 6: rest" alone would not confirm it. F2 stays compact: "6 rest".
- Speech channel, which and why: accesskit_windows 0.35.2 raises UIA_ValueValuePropertyId changed for a node whose value text changed (node.rs properties! macro, enqueue_simple_property_changes) and nothing when it is equal. So the focused editor's value change is the channel when the key changed its own step (consistent with task-189: the focused control speaks its own change); the announcer when the value cannot say it: the same note again (no change, no event), a pitch key in pitch mode (the edit step moved on: the value shows the NEXT step, so "Step 5: D sharp" is announced, then the value change), and whenever the focus is not on that 303's editor (a knob, a button by Space or a screen reader's Click). By-ear check that these do not double or cut each other: TASK-197 / NVDA checklist item 8.
- Up/Down on the focused editor: D6 says Up/Down change the focused value; the editor's value is the step, so Down = Back, Up = Step, as on a Windows slider. Page Up/Down do nothing. Home/End = repeated Back/Step presses (Panel::press, the mouse's path; at most 15; nothing sent to the engine). End = the pattern's last step (length-1); Left/Right wrap modulo 16 as the original's buttons.
- Failed first attempt: press_effect suppressed Step/Back announcements only when the edit step did NOT move (copied from the key rule) -> unit test what_a_press_says caught it; Step/Back on the focused editor are spoken by its value change.
- Tap (Ctrl+Tab) announces nothing (not asked; playback silence rule).

Results (2026-11-06):
- First full release run: a11y_announce's two F2 tests expected the row-only text for a 303 row (now followed by the steps): updated, the Trakka Rekka test now pins the steps of Bass Line 1 pattern A6 ("1 A sharp octave down, ..., 16 C"). a11y_step303's event count guess was wrong (14 events, not > 15): fixed.
- tools/uia_tree.ps1 on the release build: -ScreenReader on 207 elements, 88 Slider (was 86), no Text (was 2), 196 focusable (was 194), all named, "Step editor" after each 303's knobs. -ScreenReader off: 198 elements, 194 focusable, as after task-188 (unchanged). Recorded in docs/accessibility.md 6.3.
- Headless baseline: mode off unchanged (203/196/112/194); screen reader mode focusable 196 (BASELINE_FOCUSABLE + 2). Focus order 192 stops (a11y_sections).
- cargo clippy --workspace --all-targets -D warnings clean; nightly-2026-10-18 fmt --check clean.
- By-ear NVDA check (value change vs announcer, double or cut speech) deferred: TASK-197 got an AC, docs/accessibility_plan.md NVDA checklist item 8 extended.
- Reproduce: cargo test --release -p orb-ui --test a11y_step303 --test a11y_announce --test a11y_sections --test a11y_baseline; pwsh tools/uia_tree.ps1 -ScreenReader on|off.

AC evidence:
- AC1: crates/orb-ui/tests/a11y_step303.rs a_pattern_programmed_by_keys_equals_the_mouse_s: a pattern of Bass Line 1 programmed with keys only (screen reader mode, the Step editor focused: C, -, P, Right, G, -, [, ', Enter, /, =, ;, Home, End, K, -, Right, Left, Backspace, Enter, Tab x3 + Space for pitch mode, Shift+Tab x3, B, N) and the same steps clicked by the mouse on a second app (mode off): the 14 Panel events are equal, the 32 patterns are equal, and F2 gives the same text on both ("Bass Line 1, pattern A1, 16 steps, shuffle off. 1 E accent, 2 rest, 3 D sharp octave up slide, 4 high C octave down, 5 rest, ..., 16 G sharp"). The spoken texts after each key are checked in the same test (value of the focused editor or the announcer).
- AC2: step303::NOTES = C, C sharp, D, D sharp, E, F, F sharp, G, G sharp, A, A sharp, B, high C (the keys C F V G B N J M K , L . /); Up/Down flags spoken "octave up"/"octave down" (unit tests the_notes_follow_the_keyboard, steps_read_as_words; F2 on Trakka Rekka in a11y_announce: "3 C octave down octave up").
- AC3: docs/ui_spec.md section 20.4 ends with "Programming a 303 pattern with a screen reader", 7 steps.

Review round 1 (2026-11-06, reviewer verdict: send back), fixed by the supervisor in the foreground (the user asked for one agent at a time; the task agent's session was not resumed):
- Defect: the focused editor's value read the row's pattern, which the engine moves during playback (link.rs follow_values -> Panel::adopt_pattern), so NVDA spoke without input. The user's decision TASK-209 (2026-11-06): move-along is a setting, off by default, never spoken; Up/Down move the step, Left/Right do nothing.
- Fix: Synth303::pin (model.rs), the pattern the programming buttons edit; Panel::edit_pattern(k) = pin or the row's pattern; press_303, the 303 lights (led_on) and step303 (value, F2) use it; press_pattern moves the pin to the pattern the user chose. App::pin_303 keeps it at the start of every frame, before the input: none with the mode off (exactly upstream), else step303::pin_of: only a frame with input sets it (no pin yet, or Options > Step Editor Follows Playback on). Settings::step_editor_follows (ui.toml step_editor_follows, default false), the Options checkbox in screen reader mode only (menu.rs, Cmd::ToggleStepEditorFollows).
- F2 on a 303: the steps of the editor's pattern, "Step editor on pattern A1: ..." first when the row plays another one.
- Keys: Move::of_key Up = Back, Down = Step, Left/Right = Move::Nothing (consumed); the node is a vertical Slider. key_help.rs rows updated.
- Reviewer note 2 applied: press_effect returns None for Pitch mode and Clear too when the focused editor's value changed (no double speech).
- Failed first attempt: the pin was taken on the first frame even without input; the_read_out_keys_on_a_known_song (a11y_announce) caught it: after a song load the pin held the panel's pattern from before the engine's report (A3 instead of Trakka Rekka's A6). Now a new panel has no pin until the next input.
- Rebase on main 988bcd5: docs/ui_spec.md conflict with TASK-193's 20.4, kept both, the step editor is now section 20.5 (references renumbered); TASK-197's file conflicted too and was restored from main, its AC #7 re-added through the CLI (Up/Down, Left/Right, playback silence).
- Tests: new a11y_step303 playback_moving_the_pattern_is_silent (adopt_pattern while focused: value and announcer unchanged, off and on; F2; a programming key edits the pin's pattern; the user's pattern choice moves it; mode off: no pin), unit tests the_pin_moves_only_as_the_setting_says and the Left/Right/Up/Down mapping, settings round trip, a11y_menu_bar for the new checkbox. The AC1 test now uses Down/Up.
- cargo test --release -p orb-ui --no-fail-fast: 388 passed, 1 failed (crash.rs, known, TASK-200), 3 ignored. Clippy (workspace, all targets, -D warnings) and nightly-2026-10-18 fmt clean. tools/uia_tree.ps1 not rerun (only the editor's orientation changed; the headless baseline and section tests pass).

Review round 2 (2026-11-06, same reviewer, send back):
- Fixed: Clear announced the row's pattern but clears the editor's (press_303 uses edit_pattern): press_effect now names panel.edit_pattern(k); unit test what_a_press_says pins A3 while the row plays A2 and expects "Pattern A3 cleared" with A3 emptied.
- Not a defect: "the pin survives a song open". Link::load replaces the panel too (link.rs `*panel = Panel::new()` right after queue_song, as load_new does), so an opened song starts with no pin; the_read_out_keys_on_a_known_song (Trakka Rekka opened, F2 reads A6) depends on it.
- Note 3 taken: the Left/Right check in the_editor_keys_move_the_edit_step_only_on_the_editor also asserts no engine event and an unchanged row focus.
- cargo test --release -p orb-ui --no-fail-fast: 388 passed, 1 failed (crash.rs, known, TASK-200), 3 ignored; clippy and nightly fmt clean.

Merged by the supervisor on 2026-11-06: reviewer verdict merge (round 2, commit bbd85a0); fast-forward of main 988bcd5, this note and the plan's status (docs/accessibility_plan.md "Where it stands") amended in. Checks on the branch before merging: nightly-2026-10-18 fmt --check clean, clippy --release --workspace --all-targets -D warnings clean, cargo test --release -p orb-ui --no-fail-fast 388 passed, 1 failed (crash.rs, known, TASK-200), 3 ignored.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The 303 step editor is spoken in screen reader mode (A11Y-6), and stays silent during playback (decision TASK-209). Branch worktree-agent-aa4eb387f6da39059, one commit on main 988bcd5.

- crates/orb-ui/src/a11y/step303.rs (new): note names, step texts, the "Step editor" node (a vertical Slider replacing the edit step label in screen reader mode, value "Step 5 of 16: C sharp, octave up, accent, slide", ", pitch mode" while on, "rest, C" for a rest), moves (Up = Back, Down = Step, Home/End = step 1 / the pattern's last step through Panel::press; Left/Right nothing), what a programming press announces, F2's read_303_pattern, and pin_of.
- Playback: the editor edits its own pattern (Synth303::pin, Panel::edit_pattern, kept by App::pin_303 before each frame's input; none with the mode off). Options > Step Editor Follows Playback (Settings::step_editor_follows, screen reader mode only, off by default): off, the editor stays on the pattern the user chose; on, it moves to the playing one at the next input. Never spoken. F2 says "Step editor on pattern A1" when the row plays another.
- Speech channel: the focused editor's value change when the key changed its own step; the announcer for the same note again, a pitch key in pitch mode, and with the focus elsewhere. By-ear check: TASK-197 AC #7 and NVDA checklist item 8.
- Also: app.rs (the node, keys, announce_303, pin_303), model.rs (pin, edit_pattern, lights), settings.rs, menu.rs, a11y/nav.rs, a11y/announce.rs (F2), key_help.rs.
- Tests: tests/a11y_step303.rs (5), 7 unit tests, settings, a11y_menu_bar, a11y_announce (F2 texts), a11y_sections (192 stops), a11y_baseline (196 focusable in the mode).
- Docs: ui_spec.md 10, 20.2, 20.3, the Options table, new 20.5 with the how-to; accessibility.md 3.3, 3.4, 6.3; accessibility_plan.md NVDA item 8.
- cargo test --release -p orb-ui: 388 passed, 1 failed (crash.rs, known, TASK-200), 3 ignored. Clippy and nightly fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
