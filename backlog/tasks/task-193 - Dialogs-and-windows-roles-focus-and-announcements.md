---
id: TASK-193
title: 'Dialogs and windows: roles, focus and announcements'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T09:50:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-189
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/error_box.rs
  - crates/orb-ui/src/info.rs
  - crates/orb-ui/src/about.rs
  - crates/orb-ui/src/key_help.rs
  - crates/orb-ui/src/mod_info.rs
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 216000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-8; the analysis is docs/accessibility.md. Read both before starting.

- What: each window gets a Dialog role and its title as name (the error box: AlertDialog);
  when it opens, the focus moves into it (the prompt: to Save; the error box: to OK; Info: to
  the first field; About, Keyboard Shortcuts, Mod Info: to the window's text or its close
  button), and when it closes the focus returns where it was. The error box's message is read
  when it opens. The texts of About, Keyboard Shortcuts and Mod Info can be read line by line
  (labels in a scrollable region, reachable with Tab or the arrows).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: for each window (the four windows, the prompt, the error box), the focused node after it opens and after it closes, and its role and name (tests/a11y_windows.rs)
- [x] #2 With NVDA, each window is announced with its title and content when it opens: moved to TASK-197 (NVDA checklist item 22); here verified in the tree: Dialog or AlertDialog role, title as name, the message as the alert's description, the focus inside
- [x] #3 With the mode off, the windows behave as on main (their tests pass unchanged; tests with_the_mode_off_* in a11y_windows.rs)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Probe egui 0.37's tree for the windows (Window role with the title; close button already 'Close window'; Modal Dialog unnamed).
2. a11y/window.rs: window() sets role Dialog and the title on the window's node (id.with('move')); dialog() names a Modal's node (Dialog or AlertDialog) with a description; line() makes a label a focus stop spoken as 'keys: what'.
3. Windows: Keyboard Shortcuts, About, Mod Info lines as focus stops; Info a Dialog. Prompt: Dialog, focus on Save once. Error box: AlertDialog 'Error' with the message as description, focus on OK once, close button named 'Close'.
4. App::window_focus: when the last modal closes, give the focus back to the window under it or the panel control (one frame later: egui treats the panel as under the modal layer in the frame right after).
5. Tests tests/a11y_windows.rs; update the two a11y_sections tests whose first stop changed; docs ui_spec 20.4, accessibility.md, NVDA checklist item 22; AC for TASK-197.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a60ffc4431977dabd

Probe (egui 0.37, mode on): each window is Role::Window named by its title (window.rs:682) at id.with('move'), its close button is egui's 'Close window' (window.rs:1491), a Modal is an unnamed Dialog. The error box's own close button was unnamed (fixed: 'Close').
Failed hypothesis: setting the role on the UiKind::Window frame of Ui::stack made a second, Dialog node under the Window node (two nodes named by the title); the real node is window_id.with('move') (area.rs:558).
Failed hypothesis 2: restoring the panel focus in the frame right after a modal closes: the focus was cleared in the same frame (after the controls drew, instrumented with eprintln), because egui still treats the panel as under the modal layer; the restore now waits one frame (App::modal_was = 2).
Changed behavior in screen reader mode (tests updated): About's first stop is its version line, not the first link; Tab in Keyboard Shortcuts moves to the next line instead of cycling on one.
Tests: cargo test --release -p orb-ui --no-fail-fast: every binary passes except crash.rs a_deliberate_panic_exits_101_and_writes_the_log (TASK-200, known). a11y_windows 9 pass, a11y_sections 21 pass, lib 117 pass. fmt and clippy -D warnings clean.
Not done: the by-ear NVDA check (AC 2) moved to TASK-197 (AC added) and the NVDA checklist item 22. Replace question (TASK-194) left as it is; it now also gets the shared focus return.

Review round 1 (send back, two fixes): (1) App::window_focus now calls request_repaint while modal_was is 1 or 2, so the focus return does not wait for input; the comment says the restore comes two frames after the modal's last open frame; test the_focus_return_asks_for_the_frames_it_needs (ctx.has_requested_repaint after the first frame, then the focus is back after two more frames with no input). (2) error_box.rs: with screen reader mode on, Copy copies exactly the message and the box stays open. Optional check dropped on purpose: with the mode off the focus does go onto OK when the panel's focus is cleared by the modal; that is main's own behavior (error_box.rs 'Escape takes the focus away'), so no assertion. Rebased on 726f5eb. cargo test --release -p orb-ui --no-fail-fast: all pass except crash.rs (TASK-200); clippy and nightly fmt check clean.

Review round 2: the repaint request in App::window_focus is now only at the closing transition (modal_was == 2), not on every frame a modal is open (idle-CPU regression). Test the_focus_return_asks_for_the_frames_it_needs also asserts has_requested_repaint() is false for three frames with the error box simply open; its comment fixed. a11y_windows 10 and error_box 15 pass; clippy -D warnings and nightly fmt check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
In screen reader mode the four windows (Keyboard Shortcuts, Song Information, About, Mod Information) are Dialog nodes named by their title, the unsaved-changes prompt a Dialog named by its question (description: its detail), the error box an AlertDialog named Error whose description is the message; the error box's close button is named Close. The focus goes to Save in the prompt and to OK in the error box on open, and when the last modal closes it returns to the window under it or the panel control. Keyboard Shortcuts, About, Mod Info and the error message are lines that take the focus (spoken as 'keys: what', 'Name: value'). Mode off: unchanged. Branch worktree-agent-a60ffc4431977dabd, one commit. Files: crates/orb-ui/src/a11y/window.rs (new), a11y/mod.rs, app.rs, about.rs, error_box.rs, info.rs, key_help.rs, menu.rs, mod_info.rs, tests/a11y_windows.rs (new, 9 tests), tests/a11y_sections.rs (2 assertions updated), docs/ui_spec.md (20.4), docs/accessibility.md, docs/accessibility_plan.md (NVDA item 22), TASK-197 (AC added). cargo test --release -p orb-ui: all pass except the known crash.rs (TASK-200); fmt and clippy clean. By-ear NVDA check: TASK-197.
<!-- SECTION:FINAL_SUMMARY:END -->
