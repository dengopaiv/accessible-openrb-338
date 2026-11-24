---
id: TASK-191
title: >-
  Options > Audio Output: driver, device, sample rate and buffer size (absorbs
  task-190)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-15T12:44:00Z'
updated_date: '2026-11-24T16:04:00Z'
labels:
  - ui
  - player
  - docs
milestone: m-5
dependencies: []
references:
  - crates/orb-player/src/output.rs
priority: medium
type: feature
ordinal: 150
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** GitHub issue 12 asks for a choice of audio output and driver. Today orb-player always opens `cpal::default_host()`'s default output device and requests a 2048-frame buffer, falling back to the device's default buffer when refused (task-176; crates/orb-player/src/output.rs `start`, `BUFFER_FRAMES`).

The user redesigned this on 2026-11-24, to be closer to commercial DAWs (Ableton Live's Audio preferences, REAPER's audio device settings):
- no automatic buffer growth;
- a safe default of 2048 frames;
- Driver, Device, Sample rate and Buffer size settings.
This task absorbs task-190, which is archived: its player-side work is here, its automatic buffer growth is dropped. The user is the first reviewer.

**Player side (crates/orb-player):**
- List the hosts cpal offers on this build, as "drivers": CoreAudio on macOS; WASAPI on Windows (cpal has no MME or DirectSound; ASIO comes with task-193); ALSA and any other host cpal offers on Linux. For each, list:
  - the output devices;
  - per device, the supported sample rates and the buffer size range.
- Open a stream with a chosen driver, device ("System default", or a named one), sample rate request ("device default", or a rate the device lists) and buffer size request. The engine runs at 44.1 kHz and orb resamples to any other rate, as today.
- **System default** follows the OS: when the default output changes (e.g. headphones plugged in) or the device disappears, reopen on the new default. The engine keeps running across a reopen (`start` already recovers the `Audio` through `Keeper`), so a reopen costs a short gap.
- A named device that disappears: reopen on System default, and report it to the UI.
- Report what is in use: driver, device, the actual sample rate and whether orb resamples, channels, sample format, the buffer requested and the first callback's size, and an underrun count. The count comes from the stream's xrun errors (`Health`) and from callback gaps clearly longer than the buffer.
- The callback still allocates nothing (tests/no_alloc.rs).
- `orb play` is unchanged: System default, 2048 frames, no options. Its start and exit lines still name the device, the buffer requested and the first callback's size.
- Pure functions get unit tests with no real audio device: the choice of sizes and rates offered from a device's supported configs, and the fallback rules.

**The window: Options > Audio Output...**, egui-drawn like the app's other windows, no native dialog. Four settings:
- **Driver:** the drivers above. The default is the platform's first: CoreAudio, WASAPI or ALSA.
- **Device:** "System default" first, then the driver's devices. Default: System default.
- **Sample rate:** "Device default", or the rates the device lists (a request). Default: Device default.
- **Buffer size:** the sizes the device allows between 256 and 8192 frames (powers of two, plus the device's own if it has a fixed one). Default: **2048**.

Below them, read-only, what is in use: the actual rate and resampling, the buffer the device granted (first callback) and the underrun counter since the stream opened.

For **ASIO** (task-193; leave the hook here), the buffer size row is replaced by the text "Set in the driver's control panel" and a "Driver Settings..." button, when task-193 can open the driver's panel. The sample rate stays selectable.

**Behavior:**
- A change applies at once: the stream reopens and the song keeps playing after a short gap.
- The choice is saved in the settings file (OpenRB-338.toml, task-289) and used at the next start.
- A saved device that is missing at start falls back to System default, with a note in the status line, not an error box.
- A chosen device or config that fails to open shows the error box (task-130) and falls back to System default with the defaults.
- Nothing panics.
- The standalone app only. The plugin's audio is the host's, and the window and menu item are not in the plugin.

**Compatibility:** these settings keys are new. No compatibility code is needed for keys that never shipped in a release.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 orb-player lists the drivers, devices, sample rates and buffer sizes cpal offers, opens a chosen driver, device, rate request and buffer request, follows the system default device and reopens when a device disappears; the callback allocates nothing
- [ ] #2 Options > Audio Output... has Driver, Device (System default first), Sample rate (Device default first) and Buffer size (256..8192, default 2048), and shows the actual rate and resampling, the granted buffer and the underrun count
- [ ] #3 A change applies at once with the song playing on, is saved in OpenRB-338.toml and is used at the next start
- [ ] #4 A missing saved device falls back to System default with a status-line note; a device that fails to open shows the error box and falls back; nothing panics
- [ ] #5 orb play is unchanged (System default, 2048 frames, the same start and exit lines)
- [ ] #6 Unit tests with no real device cover the size and rate lists, the settings round trip and the fallbacks; docs/ui_spec.md section 18 and docs/building.md describe the window and the player
- [ ] #7 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #8 The user has checked it on the Mac (switching devices and buffer sizes while playing) and on Windows
<!-- AC:END -->
