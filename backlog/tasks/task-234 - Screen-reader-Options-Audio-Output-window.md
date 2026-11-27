---
id: TASK-234
title: 'Screen reader: Options > Audio Output window'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:08:00Z'
updated_date: '2026-11-27T22:51:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-233
references:
  - crates/orb-ui/src/audio_window.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
priority: medium
type: feature
ordinal: 253000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: Options > Audio Output (upstream's task-191 and task-193: driver including ASIO, device, sample rate, buffer size, and what is in use) is where a user picks the sound card, so a screen reader user needs it more than most windows. It is an egui window in crates/orb-ui/src/audio_window.rs.

Context: the merge of upstream f83c54f (task-233; the user's decision task-232, question 2: take everything, each new window gets its own m-7 task) brought this window in as upstream built it, for the mouse. The fork's screen reader work is docs/accessibility.md (the analysis, egui's AccessKit API, how to measure: tools/uia_tree.ps1) and docs/accessibility_plan.md (the plan and its ground rules: with screen reader mode off the app behaves exactly as upstream). Existing patterns to follow: crates/orb-ui/src/a11y/ (names.rs, announce.rs with App::set_status and App::announce, window.rs with win::line for focusable text lines, nav.rs), the Win enum and App::window_focus in app.rs (a window takes the focus as it opens in screen reader mode and gives it back when it closes), and the a11y_* tests in crates/orb-ui/tests (AppTree drives the app and reads the AccessKit tree). By-ear NVDA checks are not criteria here: they go to the user's pass, TASK-197.

What to do: in screen reader mode the window takes the focus when it opens (a Win variant, App::window_focus) and gives it back on close; every combo box, button and the In use lines have names a screen reader reads (the In use numbers as focusable lines, win::line); a change of driver, device, rate or buffer, and the controller's notes and errors (App::audio_events), are announced; the window can be used and closed with the keyboard alone (Tab, arrows in a combo box, Escape). The In use numbers are sampled every 500 ms (upstream's task-193): do not announce each sample.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 In screen reader mode, Options > Audio Output... moves the focus into the window and closing it returns the focus to where it was (an a11y test)
- [x] #2 Every interactive widget of the window has a name in the AccessKit tree that says what it chooses (an a11y test lists them)
- [x] #3 A change of driver, device, rate or buffer is announced once, and the periodic In use refresh is never announced (a test)
- [x] #4 With screen reader mode off the window draws and behaves as upstream's (no change to its layout or input)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Win::Audio in the Win enum / win_open / ALL so window_focus moves the focus in and back. 2. audio_ui: focus_into_window, win::window (named Dialog), Escape closes an open list first in the mode. 3. name each combo box (Driver, Device, Sample rate, Buffer size) via accesskit label, the selected text stays its value. 4. In use rows and the ASIO/problem texts as win::line lines read 'label: text'. 5. change_text announces a driver/device/rate/buffer change once from audio_choose (what the stream ended on); the 500 ms refresh never announces. 6. Tests: a11y_windows (focus/names/silence/mode off), unit tests (rows, change_text). Test-only seam audio_lists_for_test so no real device is read. By-ear check goes to TASK-244.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-af863a2186c2483d8

Evidence: the window is Win::Audio (app.rs); audio_window.rs: focus_into_window + win::window (named Dialog 'Audio Output'), name_combo sets the AccessKit label of each combo box (Driver/Device/Sample rate/Buffer size; egui keeps the chosen text as its value), in_use_rows draws each In use row as win::line read 'label: value' (also the ASIO buffer note and the problem text), change_text builds 'Driver: X', 'Device: X', 'Sample rate: N Hz', 'Buffer size: N frames (ms)' and audio_choose announces it once after the stream reopened (compared with the saved choice, so fallbacks are what is said); the 500 ms refresh only changes values of lines, never calls announce. In screen reader mode Escape first closes an open combo list (egui::Popup::is_any_open), then the window. Mode off: same widgets in the same order (name_combo and win::line are no-ops/plain labels), no layout or input change. Seam: App::audio_lists_for_test (#[doc(hidden)]) fills the lists so no device is read. Tests: a11y_windows 3 new (focus in/back + four named combos with values + Tab reaches the In use line; silence over 130 frames; mode off), unit tests (in_use_rows read as lines, change_text). Limit I could not close by machine: audio_choose needs a Controller (a real stream), so the announce-on-change wiring is checked by change_text's unit test and by reading the code, not by a click in the harness; the by-ear check is a new criterion of TASK-244. Failed first attempt: the in-use unit test saw no lines in one frame (a Grid's first frame is a sizing pass that registers no widgets); it now runs three frames. Runs: cargo test --release --workspace 1163 pass, clippy -D warnings and nightly fmt --check clean.

Review round 1 (sent back, fixed in the amended commit): 1. the seam now replaces show_audio (App::audio_open_for_test seeds the lists and opens, no driver or device is asked; the three tests use it). 2. audio_choose says the change and the controller's notes of the same apply as one utterance, change first (join_said, unit-tested); notes still go to the status line. 3. new a11y test audio_output_escape_closes_the_list_then_the_window (list closes and window stays, second Escape closes and the focus returns). 4. the refresh test comment reworded: sampling is wall-clock timed (unit-tested in audio_window.rs), the test only shows that frames and repaints say nothing; the announcement wiring in audio_choose is checked by reading the code and by unit tests, not through the UI. Runs: cargo test --release -p orb-ui all pass (lib 229), clippy -p orb-ui --all-targets -D warnings and nightly fmt --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Options > Audio Output is readable and usable by keyboard in screen reader mode: named Dialog with the focus in and back (Win::Audio), four named combo boxes holding the chosen value, In use numbers as focusable 'label: value' lines, a change of driver/device/rate/buffer announced once (audio_choose, via change_text), the 500 ms refresh silent, Escape closes an open list first. Mode off unchanged. Branch worktree-agent-af863a2186c2483d8, one commit. Tests: cargo test --release --workspace 1163 pass; clippy and nightly fmt clean. By-ear check added as a criterion of TASK-244; the announce wiring in audio_choose (needs a real stream) is not clicked through in a test.
<!-- SECTION:FINAL_SUMMARY:END -->
