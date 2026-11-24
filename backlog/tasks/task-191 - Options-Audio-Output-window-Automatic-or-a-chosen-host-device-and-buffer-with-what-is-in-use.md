---
id: TASK-191
title: >-
  Options > Audio Output: driver, device, sample rate and buffer size (absorbs
  task-190)
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-15T12:44:00Z'
updated_date: '2026-11-24T17:46:00Z'
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
- **Buffer size:** from the device's own minimum to 8192 frames, clamped into its range (the minimum itself first, then the powers of two above it; one size for a device with a fixed size), each entry with its time at the stream's rate ("2048 frames (46.4 ms)"). Default: **2048**. The lists are read when their combo box opens, never every frame; no Refresh button.

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
- [x] #1 orb-player lists the drivers, devices, sample rates and buffer sizes cpal offers, opens a chosen driver, device, rate request and buffer request, follows the system default device and reopens when a device disappears; the callback allocates nothing
- [x] #2 Options > Audio Output... has Driver, Device (System default first), Sample rate (Device default first) and Buffer size (from the device's minimum to 8192 frames with the time in ms, default 2048), and shows the actual rate and resampling, the granted buffer, the measured output latency and the underrun count
- [x] #3 A change applies at once with the song playing on, is saved in OpenRB-338.toml and is used at the next start
- [x] #4 A missing saved device falls back to System default with a status-line note; a device that fails to open shows the error box and falls back; nothing panics
- [x] #5 orb play is unchanged (System default, 2048 frames, the same start and exit lines)
- [x] #6 Unit tests with no real device cover the size and rate lists, the settings round trip and the fallbacks; docs/ui_spec.md section 18 and docs/building.md describe the window and the player
- [x] #7 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #8 The user has checked it on the Mac (switching devices and buffer sizes while playing) and on Windows
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. orb-player: devices.rs (drivers, devices, caps, pure buffer_sizes/offered_rates, Request), output.rs (open(audio, Request) with the engine handed back on refusal, Stream::stop/in_use, underruns from Xrun errors and is_gap), control.rs (Controller: apply, poll follows the default device and lost devices, fallback chain).
2. orb-ui: settings AudioChoice (audio_* keys), audio_window.rs (Options > Audio Output...), App.audio Controller, main.rs starts from the saved choice, hidden --audio flag for screenshots.
3. Tests: pure lists and fallbacks, settings round trip, open refusals without a device, a real-device reopen test that skips without a device.
4. Docs: ui_spec section 18, building.md.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-abc6e4e8e4f309a1f (rebased on local main 06edcd4).

2026-11-24 first user review round (task still To Do). Evidence: cargo test --workspace 1131 pass, 0 fail; clippy -D warnings clean on macOS and x86_64-pc-windows-msvc (cargo xwin); nightly fmt --check clean. Real-device test crates/orb-player/tests/reopen.rs on this Mac (CoreAudio, MacBook Pro Speakers): opened default request (44100 Hz, not resampled, 2048 requested / 2048 first callback), reopened at 512 frames, the song's tick kept advancing across the reopen, a rate no device lists (12345) fell back to the default request with an Error event. The test skips without a device. Window screenshot: OpenRB-338 --audio --screenshot (hidden flag added for this). Decisions taken, for the user to confirm: (1) 'Device default' sample rate keeps today's rule (44.1 kHz when the device offers it, else the device's own rate, resampled), so orb play and the default are unchanged; a literal device-default rate would resample on a 48 kHz device. (2) Only the all-default request falls back to the device's own buffer when the size is refused (task-176 behaviour, orb play); any other refused request shows the error box and opens the defaults, which are then saved. (3) A saved device that is missing at start opens System default with a status-line note but the saved choice is kept, so the device is used again when it is back. (4) The app asks for one frame a second while it owns a stream, to notice a changed default device or a lost one while idle (ui_spec section 16). (5) No automatic buffer growth; underruns are counted from Xrun errors and from callback gaps over two buffers and 20 ms. (6) Windows: not run here (no Windows machine on the dev side); code is cfg-free cpal API and passes cargo xwin clippy; the zip is for the user's check. Not done yet: the ASIO Driver Settings button (task-193; the buffer row already shows 'Set in the driver's control panel' for a driver named ASIO), AC 8 (the user's check on Mac and Windows).

Review round 1 (user, 2026-11-24): (1) buffer list now runs from the device's minimum (the minimum itself first, then the powers of two above it) to 8192, clamped into the range; a device with no stated range gets 64..8192; task description and criterion 2 updated. (2) Refresh button removed; devices plus the selected device's rates and sizes are re-read when the Device list opens, rates and sizes when the Sample rate or Buffer size list opens (a combo header click), drivers only when the window opens; never per frame. (3) '(follows the system default)' removed. (4) Latency: entries read '2048 frames (46.4 ms)' at the stream's rate; the In use buffer line adds the first callback's ms; cpal 0.18.2 OutputCallbackInfo does give timestamp().playback and .callback, so an 'Output latency' row shows the last callback's difference (CoreAudio here: 49.6 ms at 2048 frames, 44.1 kHz; one atomic store in the callback, no allocation). ui_spec section 18 says a control change can come up to one 1024-frame engine step (about 23 ms) later on top of the buffer. Interpretation note: 'rounded up to a power of two, or the minimum itself' was taken as: the minimum itself is always the first entry. Rerun: workspace tests, clippy macOS and Windows, fmt, app and zip — see the final report of this round.

Review round 2 (user, 2026-11-24): the In use Buffer line shows only the first callback, 'N frames (M ms)' (e.g. 2048 frames (46.4 ms)), no 'requested' wording and no range of callback sizes; the requested size is the Buffer size combo. ui_spec section 18 updated; no test quoted the old line. Everything else, including the minimum first in the list, stays.

2026-11-24 sign-off: the user checked the Mac app and the Windows zip 2026.1124.1722 and answered 'good' (criterion 8). Criteria evidence: 1 devices.rs, control.rs, output.rs, tests/reopen.rs (real CoreAudio reopen), tests/open_refused.rs, tests/no_alloc.rs unchanged and passing; 2 window screenshot audio2.png; 3 tests/audio_settings.rs and reopen.rs; 4 Controller::open_chain fallbacks, tests/open_refused.rs; 5 orb play uses output::start(default Request), start and exit lines untouched; 6 devices.rs and audio_window.rs unit tests, audio_settings.rs, ui_spec section 18, building.md; 7 workspace tests 1133 pass, clippy macOS and Windows clean, fmt clean; 8 the user's check.

Reviewer round (after Done): (1) the 1 s repaint is now only with an open stream; with none open the controller's retry is polled every 2 s, documented in ui_spec section 16 (app.rs check_audio). (2) the callback bookkeeping moved to output::CallbackBook (enter: gap check, underrun, latency; leave: stats, health clock) and tests/no_alloc.rs calls it in the armed region, with a late call that counts an underrun: 0 allocations. (3) tests/fallbacks.rs covers Controller::start with no device needed: an unknown saved device gives a Note and runs on System default (the saved choice is the settings', Controller never writes it), a refused driver or rate gives one Error and the default request with the engine back (apply works, never 'engine is lost'), and the default request refused is one error. (4) nit: the Buffer size combo shows Caps::requested (the saved size clamped into the device's range), with unit tests.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Options > Audio Output... is done: Driver, Device (System default first), Sample rate (Device default first) and Buffer size (from the device's minimum to 8192 frames with ms, default 2048), with an In use block (device, rate and resampling, format, first callback in frames and ms, measured output latency, underruns) and no automatic buffer growth. orb-player has devices.rs, control.rs (Controller: reopen on change, follow the system default, lost device falls back to System default, refused request falls back to the defaults) and output.rs changes; orb-ui has audio_window.rs, the audio_* settings keys and a hidden --audio flag. Branch worktree-agent-abc6e4e8e4f309a1f, one commit (reviewed by the user in two rounds, signed off 2026-11-24). Tests: cargo test --workspace 1133 pass, clippy -D warnings on macOS and x86_64-pc-windows-msvc, nightly fmt clean. The ASIO Driver Settings button stays with task-193 (the buffer row already reads 'Set in the driver's control panel' for a driver named ASIO).
<!-- SECTION:FINAL_SUMMARY:END -->
