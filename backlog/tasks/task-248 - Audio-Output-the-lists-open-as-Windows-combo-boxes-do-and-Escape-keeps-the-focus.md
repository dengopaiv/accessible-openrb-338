---
id: TASK-248
title: >-
  Audio Output: the lists open as Windows combo boxes do, and Escape keeps the
  focus
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-28T06:51:00Z'
updated_date: '2026-11-28T09:17:00Z'
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
- [x] #1 In screen reader mode Escape on an open list closes it and the focus stays on its combo box (the tree's focused node); a second Escape closes the window and the focus returns; a11y tests
- [x] #2 Alt+Down and Enter open a box's list, and Up / Down on a closed box change its choice once each, announced once, through the same path as a choice in the list; a11y tests (the window's test seam, App::audio_open_for_test)
- [x] #3 Each box carries a description of how to open it, and Help > Keyboard Shortcuts lists the keys in screen reader mode; tests
- [x] #4 With the mode off the window is as before (its tests pass unchanged); docs/ui_spec.md, CHANGELOG.md's next section, docs/accessibility_plan.md and docs/accessibility.md updated; one by-ear criterion added to TASK-244
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Tests first (crates/orb-ui/tests/a11y_windows.rs, AppTree gets key_with for modifiers): Escape keeps the focus on the box, Alt+Down/Enter/Space open, list navigation and choosing, closed-box keys with one announcement each, the open-hint description, mode off unchanged. Confirm they fail on main.
2. audio_window.rs: one helper audio_combo (ComboSpec, ListNav) replaces the four ComboBox closures. Screen reader mode only: Alt+Down opens; in the open list the highlight is the focus on an entry (Up/Down/Home/End, Enter/Alt+Up choose, Escape keeps); on a closed box Up/Down/Home/End choose; EventFilter (vertical arrows, escape) on the focused box and entry keeps egui from moving or clearing the focus; the focus is returned to the box when a list closes. The description "Alt+Down opens the list" on each box.
3. Test seam: with lists_fixed (audio_open_for_test) audio_choose does not open a stream or ask a driver, the refreshes are skipped.
4. Help > Keyboard Shortcuts line; docs (ui_spec, CHANGELOG, accessibility_plan, accessibility); TASK-244 criterion 9.
5. cargo test --release -p orb-ui, workspace, clippy, nightly fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a2b2f45e0ce216771

Evidence and what was learned (all checked by running the a11y tests; cargo test --release -p orb-ui --test a11y_windows audio_output):
- Before the change the four new tests fail (the description is missing, Alt+Down does not open, Down on a closed box chooses nothing, the focus is lost after Escape in a list); the old Audio Output tests pass.
- egui 0.37 facts the helper rests on: the combo's response id is not the id make_persistent_id gives (the list's popup id is button_id.with("popup") where button_id = ui.make_persistent_id(IdSalt::new(salt)); hashing the string directly gives another id, my first attempt); a list can only be opened before the box is drawn (memory.open_popup is dropped at the end of a frame the popup was not shown in), so the box that had the focus at the end of the last frame is remembered (ListNav::focused); egui closes a popup on a pointer click only, a key's choice needs ui.close(); consume_key(NONE, ArrowUp) also matches Alt+Up, so Alt+Up is consumed first; Escape in Focus::begin_pass clears the focus of any widget whose EventFilter lacks escape: that was the lost focus. The boxes and the entries in the open list set EventFilter {vertical_arrows, escape}.
- A first version of the entry highlight reset the cursor to the entry that still had the focus while the loop ran; the loop now only requests the focus on the moved frame.
- Failed hypothesis: the lists with the real device rates (8000..192000 Hz) showed in the test after Alt+Down, because opening a Sample rate list reads the device again (refresh_caps); the test seam (lists_fixed) now skips these reads and audio_choose's controller apply, so no test opens a stream or asks a driver.
- Not decided, as the task settles it: Home and End on a closed box and in the open list (the task names Home and End for the closed box only; the open list got them too, as a Windows list box has them). A box with no chosen entry (a device that is gone): Down goes to the first entry, Up to the last.
- With the mode off the code path is the old one (no key is consumed, no description, no focus request); with_the_mode_off_audio_output_is_as_before passes unchanged, and a new test checks Alt+Down does nothing with the mode off.
- Commands: cargo test --release -p orb-ui (all pass), cargo test --release --workspace, cargo clippy --release --all-targets -- -D warnings, cargo +nightly fmt --all --check.
- Agents cannot hear NVDA: TASK-244 criterion 9 (checklist item 37) added; criterion 3 (item 31) stays open for the user.

Review round 1 (send back), fixed in the amended commit:
1. Blocking: egui's consume_key matches an extra Alt or Shift, so Alt+Up and Shift+arrows acted as the plain keys. All keys of the lists now go through consume_exact (matches_exact on the event's modifiers). New test audio_output_modified_keys_are_not_the_plain_ones: Alt+Up, Alt+End, Shift+Down, Shift+Up, Shift+Home and Shift+End on a closed Sample rate box change nothing and announce nothing; Alt+Down right after Tab opens without choosing; Shift+Down in the open list does not move the highlight.
2. New test audio_output_device_and_driver_follow_the_arrows: Driver (Down to ASIO, End and Down say nothing, Home back to WASAPI stored as none, Up no wrap) and Device through after_device (three devices: Down, Down, End, no wrap at the end, Home to System default, Up no wrap; the first choice also says the buffer size the device moves to).
3. ui_spec.md: the focus is on the box "after Escape or a choice by key"; a click elsewhere takes it where it clicks.
4. set_focus_lock_filter takes effect only for a widget that had the focus last frame, so the first frame after Tab is unguarded. Tried to show a problem: Tab and Down in consecutive frames (no release between, in the Device test), also with the cancelling move_focus(None) removed: no focus move, the choice is made once and the focus stays. No problem shown, so no fix; the comment is in audio_window.rs at the entries' filter.
5. Space on an entry chooses it (egui's selectable_label); added to ui_spec.md, key_help.rs, CHANGELOG.md, accessibility.md, accessibility_plan.md item 37 and the helper's doc comment; test audio_output_space_chooses_the_highlighted_entry.
6. The state paragraph of accessibility_plan.md is reflowed.

Review round 2 (agent-reviewer): send back for one line, key_help.rs:395 at 104 columns (a string literal rustfmt does not wrap). The supervisor split it by hand (the same text), checked key_help's 13 unit tests and nightly fmt, and amended; then merge.

The user, 2026-11-28, on the two calls left for them: keep both. Home and End also move the highlight in an open list; on a box whose chosen entry is not in the list, Down goes to the first entry and Up to the last.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: in screen reader mode Options > Audio Output's four lists work as a Windows dialog's combo box (branch worktree-agent-a2b2f45e0ce216771, one commit, "ui: Audio Output lists as Windows combo boxes, Escape keeps the focus"). Each box's description is "Alt+Down opens the list". Space, Enter and Alt+Down open the list, the highlight (the focus on an entry, so NVDA reads it) starts on the chosen entry; Up, Down, Home and End move it with no wrap; Enter or Alt+Up choose it; Escape closes the list and keeps the choice. On a closed box Up, Down, Home and End choose the previous, next, first and last entry at once, through audio_choose and change_text, one announcement each, nothing at the ends. The focus is on the box after every close, so a second Escape closes the window and the focus returns to the panel. Help > Keyboard Shortcuts has the line "In Options > Audio Output". With the mode off nothing of this runs (the old test passes unchanged; a new test checks Alt+Down does nothing).
Code: audio_combo, ComboSpec and ListNav in crates/orb-ui/src/audio_window.rs replace the four ComboBox closures; the Escape focus loss was egui's Focus::begin_pass clearing the focus of any widget whose EventFilter lacks escape. The test seam (audio_open_for_test) now also skips the controller apply and the device reads, so no test opens a stream or asks a driver. AppTree::key_with (tests/common/a11y.rs) sends a key with modifiers.
Tests: 4 new a11y tests in crates/orb-ui/tests/a11y_windows.rs (failed before the change: description missing, Alt+Down and Down on a closed box did nothing, the focus was lost after Escape in a list) and 1 in key_help.rs; cargo test --release --workspace 1211 pass (gated CD tests skip) plus the new key_help test; cargo clippy --all-targets -- -D warnings clean (run with --release); cargo +nightly fmt --all --check clean.
By ear: TASK-244 criterion 9 (checklist item 37) added; item 31 (criterion 3) stays for the user to hear again. Docs: ui_spec.md, CHANGELOG.md, accessibility_plan.md, accessibility.md.
Open points for the user, not decided here: Home and End also work in the open list (the task names them for the closed box only); on a box whose chosen entry is not in the list (a device that is gone) Down goes to the first entry and Up to the last.
<!-- SECTION:FINAL_SUMMARY:END -->
