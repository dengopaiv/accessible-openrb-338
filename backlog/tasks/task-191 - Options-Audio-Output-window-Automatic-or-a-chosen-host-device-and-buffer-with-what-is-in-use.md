---
id: TASK-191
title: >-
  Options > Audio Output window: Automatic or a chosen host, device and buffer,
  with what is in use
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-15T12:44:00Z'
updated_date: '2026-11-15T12:45:00Z'
labels:
  - ui
  - player
  - docs
milestone: m-5
dependencies:
  - TASK-190
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/settings.rs
  - crates/orb-player/src/output.rs
  - docs/ui_spec.md
  - task-190
  - https://github.com/pingw33n/OpenRB-338/issues/12
priority: medium
type: feature
ordinal: 205000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
GitHub issue #12 (2026-11-15) asks for choosing the audio driver/output; today OpenRB-338 always plays on the system default device. task-190 gives orb-player an Automatic mode (follows the OS default device, buffer growing on underruns) and the device list. This task adds the UI the user asked for (2026-11-15): a new item **Options > Audio Output...** that opens a window (egui-drawn like the app's other windows, no native dialog) where the user picks:
- **Automatic** (the default), or
- a host (as cpal lists them: CoreAudio; WASAPI, and ASIO once task-193 adds it; ALSA and any other Linux host cpal offers), a device of that host, and a buffer size (Automatic, or a fixed size from the sizes the device allows).

The window also shows what is in use: host, device, sample rate and whether orb resamples (the engine runs at 44.1 kHz), channels and sample format, the buffer requested and the first callback's size, the underruns counted since the stream opened, and in Automatic why the buffer grew. The choice is saved in the settings file (TOML) and applied at once (the stream reopens; the song keeps playing after a short gap). In Automatic the app remembers per device the buffer size that worked (task-190 reports it) and starts there next time. A saved device that is missing at start falls back to Automatic with a line in the status line, not an error box; a chosen device that fails to open shows the error box (task-130) and falls back to Automatic.

`orb play` stays Automatic only (task-190) and reads none of these settings.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Options > Audio Output... opens a window to choose Automatic or a host, a device and a buffer size (Automatic or fixed), and shows the host, device, sample rate and resampling, channels, format, buffer requested and first callback, underrun count and, in Automatic, the reason for any growth
- [ ] #2 A choice applies at once (the stream reopens while the song keeps playing) and is saved in the settings file; the next start uses it
- [ ] #3 In Automatic the buffer that worked is remembered per device and used as the next start's first request
- [ ] #4 A saved device missing at start falls back to Automatic with a status-line note; a chosen device that fails to open shows the error box and falls back to Automatic; neither panics
- [ ] #5 Tests in crates/orb-ui cover the settings round trip (Automatic, a chosen device, a fixed buffer, the per-device buffers), the fallback of a missing device and the window's actions, with no real audio device
- [ ] #6 docs/ui_spec.md section 18 describes the menu item and the window; a manual check on the Mac (switch devices while playing, back to Automatic) is recorded in the notes; fmt, clippy and cargo test pass
<!-- AC:END -->
