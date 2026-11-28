---
id: TASK-245
title: Audio Output shows the lost device after the player falls back
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-27T22:48:00Z'
updated_date: '2026-11-28T01:19:00Z'
labels:
  - ui
  - player
milestone: m-7
dependencies:
  - TASK-234
references:
  - crates/orb-ui/src/audio_window.rs
  - crates/orb-player/src/control.rs
priority: low
type: bug
ordinal: 262000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: found in the review of task-234 (2026-11-28). When the audio device goes away while playing, orb-player's controller falls back (control.rs open_chain, the device-missing branch: 'Audio device X is not available; using System default') and App::check_audio passes the note on through App::audio_events and set_status, so it is announced. But Options > Audio Output's own lists and chosen values (crates/orb-ui/src/audio_window.rs, the AudioUi state and its Device combo) are not refreshed from the controller after such a reset: the Device combo still shows, and in screen reader mode reads as its value, the lost device until the user picks again. This predates task-234 (upstream's window behaves the same) and affects both modes; for a screen reader user it is worse, as the value read is wrong.

What to do: after a controller reset that changes what is in use, update the window's chosen driver/device (and rate/buffer if they changed) from the controller's in-use state, without announcing anything beyond the controller's own note (move-along doesn't speak). Keep the window's layout as upstream's.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 After a simulated device-lost fallback the window's Device (and any other changed) choice shows what is now in use (a test through the window's state, without real devices)
- [x] #2 No announcement besides the controller's own note is made by the update (a11y test)
- [x] #3 With screen reader mode off the window draws as upstream's apart from the corrected value
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Controller::for_test (doc(hidden)) simulates a fallback without devices. 2. App::audio_events (called by check_audio and startup) follows the controller: settings.audio := AudioChoice::from_request(c.request()) when it differs, silently, not saved (the preference stays if the device returns); lists refreshed if the window is open. 3. a11y test in both modes: Device value, one announcement (the note). 4. CHANGELOG line.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a06db7177f2535ee1

Evidence: App::audio_events (check_audio and startup path) now follows the controller: settings.audio := AudioChoice::from_request(c.request()) when it differs, silently, not saved (the saved preference stays if the device returns; set_audio is not used), lists re-read if the window is open. Seam: Controller::for_test (doc(hidden)), AudioUi.fixed set by audio_open_for_test. Test audio_output_follows_a_fallback_and_says_only_the_note (both modes): Device value Speakers -> System default, rate kept, exactly one announcement in sr mode (the note), none with the mode off, an empty poll says nothing. Full run: cargo test --release --workspace all ok, clippy and nightly fmt clean. No by-ear check needed.

Review round 1: (1) the fallback no longer touches settings.audio: AudioUi.in_use_choice holds what runs when it differs from the saved choice, the window draws from App::audio_choice(); only audio_choose (a user pick) saves. Test a_fallback_is_shown_and_never_saved saves another setting (theme) after a fallback and reads the file back: the saved device stays. (2) main.rs calls app.audio_follow_controller() after the controller start; tested (start with no events). (3) the seam is AudioUi.lists_fixed, set only by audio_open_for_test; the real path never sets it. (4) a fallback while a list is open sets refresh_pending, the lists are read when no popup is open; test a_fallback_waits_for_the_open_list. (5) the saved device returning: the controller runs it again, in_use_choice is None, the window shows the saved device; nothing is written (tested). The controller's retry only reopens the system default, so while on the default the window shows System default. (6) TASK-244 got the by-ear criterion. Full run: cargo test --release --workspace 0 failed, clippy and nightly fmt clean.

Review round 2: (1) the start order is now App::adopt_audio_and_settings(controller, settings) (audio, then settings, then follow), called by main.rs and by the test a_fallback_is_shown_and_never_saved, which starts from App::new with the saved device missing. (2) The controller is unchanged and never returns to the saved device (retry and poll use the system default only): doc comments corrected; the test now checks the next start finding the device (window shows it, file unchanged). (3) show_audio's refresh branch clears refresh_pending. cargo test --release -p orb-ui, clippy -D warnings and nightly fmt --check pass.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Options > Audio Output now follows the controller after a fallback (App::audio_follow_controller, called from audio_events): the window's choice (device, rate, buffer) becomes the controller's in-use request, silently; only the controller's note is announced. The window's drawing is unchanged. In-memory only, not saved. Tests: new a11y test in both modes; cargo test --release --workspace, clippy -D warnings, nightly fmt check all pass.
<!-- SECTION:FINAL_SUMMARY:END -->
