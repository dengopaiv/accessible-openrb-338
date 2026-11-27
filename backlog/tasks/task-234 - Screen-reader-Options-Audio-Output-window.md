---
id: TASK-234
title: 'Screen reader: Options > Audio Output window'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:08:00Z'
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
- [ ] #1 In screen reader mode, Options > Audio Output... moves the focus into the window and closing it returns the focus to where it was (an a11y test)
- [ ] #2 Every interactive widget of the window has a name in the AccessKit tree that says what it chooses (an a11y test lists them)
- [ ] #3 A change of driver, device, rate or buffer is announced once, and the periodic In use refresh is never announced (a test)
- [ ] #4 With screen reader mode off the window draws and behaves as upstream's (no change to its layout or input)
<!-- AC:END -->
