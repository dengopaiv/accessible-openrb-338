---
id: TASK-190
title: >-
  Audio output: Automatic device choice and a buffer that grows on underruns;
  orb play always Automatic
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-15T12:44:00Z'
updated_date: '2026-11-15T12:45:00Z'
labels:
  - player
  - cli
milestone: m-5
dependencies: []
references:
  - crates/orb-player/src/output.rs
  - crates/orb-cli/src/main.rs
  - docs/export_state.md
  - docs/building.md
  - task-176
  - https://github.com/pingw33n/OpenRB-338/issues/7
  - https://github.com/pingw33n/OpenRB-338/issues/12
priority: medium
type: feature
ordinal: 204000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Today orb-player opens `cpal::default_host()`'s default output device with a fixed 2048-frame buffer request, falling back to the device's default buffer when refused (task-176, `output.rs` `start`, `BUFFER_FRAMES`). Nothing reacts when that buffer underruns: on Linux behind pipewire-alsa (GitHub issue #7) the default ring of two periods underran 23-24 times a second, and the 2048-frame fix was never confirmed by the reporter. GitHub issue #12 asks for choosing the audio output; the selection UI is task-191 and builds on this task's Automatic mode and device list.

The user's plan (2026-11-15): orb-player gets an **Automatic** mode that picks the device and adapts the buffer itself, and `orb play` always uses it (no device listing or choice in the CLI, no flags).

**Device in Automatic.** The device the user chose in the OS is where they listen, so Automatic follows it and never overrides it for a "better" one:
- host per platform: CoreAudio on macOS; WASAPI on Windows (never ASIO, which is often exclusive; ASIO is task-193 and only an explicit choice); on Linux cpal's ALSA host with `default` (cpal's native PipeWire host is out of scope here: if the growing buffer does not cure PipeWire underruns, write a Decide task for it with the evidence);
- that host's default output device; only when it fails to open, the other devices, those offering 44.1 kHz stereo first; on ALSA skip raw `hw:` devices, `null`, and the surround/HDMI aliases unless one of them is the default;
- when the default device changes (headphones plugged in) or the stream reports the device gone, reopen on the new default. The engine keeps running across a reopen (`start` already recovers the `Audio` from a refused build through `Keeper`), so a reopen costs a short gap, not a restart.

**Buffer in Automatic.** Start at 2048 frames (task-176's value). Detect underruns from the stream's Xrun errors (ALSA reports them; `Health` counts errors) and from callback timing (a gap between callbacks clearly longer than the buffer, since CoreAudio and WASAPI rarely report xruns). More than about 2 underruns in 5 s reopens the stream with twice the buffer, up to 8192 frames, with one line on stderr saying so. Never shrink automatically. The size that worked is returned to the caller so the UI can remember it per device (task-191); `orb play` remembers nothing between runs.

**Latency, for reference.** Commands apply once per 1024-frame engine call (`CALL_FRAMES`), as the original's MME path does (docs/export_state.md 2.1, 2.3), so control latency is about 23 ms of grain plus the device buffer. The original queued 2 to 16 such buffers ahead (46 to 371 ms; 16 by default, the test machine's 371 ms); its DirectSound path rendered in multiples of 512 frames (its write-ahead is not traced). So 2048 to 8192 frames is still at or below the original's latency, and smaller buffers gain little until a low-latency mode exists (not in this task).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 orb-player has an Automatic mode that picks the host and device as the description says, falls back to the other devices only when the default fails to open, and reopens on the new default when the default changes or the device disappears
- [ ] #2 In Automatic the buffer starts at 2048 frames and doubles up to 8192 after repeated underruns (Xrun errors or callback gaps), never shrinks, prints a line when it grows, and reports the size in use to the caller
- [ ] #3 Pure functions for the device ranking (from a list of device descriptions) and the growth rule (from a series of underrun times) are unit-tested, including the cap, no shrinking, the ALSA skips and the fallback order; the callback still allocates nothing (tests/no_alloc.rs)
- [ ] #4 orb play always uses Automatic, has no device or buffer options, and its start and exit lines still name the device, the buffer requested and the first callback's size, plus any growth
- [ ] #5 A manual check on the Mac (play, unplug and replug headphones or switch the default output) is recorded in the notes; the Linux path is described for the issue #7 reporter in a reply text in the notes (not posted)
- [ ] #6 docs/building.md describes Automatic and orb play's lines; fmt, clippy and cargo test pass
<!-- AC:END -->
