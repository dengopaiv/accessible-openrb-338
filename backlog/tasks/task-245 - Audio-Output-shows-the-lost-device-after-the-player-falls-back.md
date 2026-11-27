---
id: TASK-245
title: Audio Output shows the lost device after the player falls back
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-27T22:48:00Z'
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
- [ ] #1 After a simulated device-lost fallback the window's Device (and any other changed) choice shows what is now in use (a test through the window's state, without real devices)
- [ ] #2 No announcement besides the controller's own note is made by the update (a11y test)
- [ ] #3 With screen reader mode off the window draws as upstream's apart from the corrected value
<!-- AC:END -->
