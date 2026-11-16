---
id: TASK-197
title: >-
  Plugin core: nice-plug VST3/CLAP instrument with state, parameters and
  64-frame timing
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:46:00Z'
labels:
  - player
  - ui
milestone: m-7
dependencies:
  - TASK-185
references:
  - backlog task-185 (final summary)
  - https://codeberg.org/RustAudio/nice-plug
  - https://docs.rs/nice-plug/latest/nice_plug/
  - crates/orb-player/src/lib.rs
  - crates/orb-engine/src/frame/export.rs
  - docs/song_load.md
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 210000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
orb becomes a VST3/CLAP instrument plugin (milestone m-7). task-185's final summary holds every product decision; read it first. This task builds the plugin's skeleton without an editor: the later tasks add host sync and pattern mode, note mode, the editor, drag-out and packaging.

Framework: nice-plug (codeberg.org/RustAudio/nice-plug, crates.io `nice-plug`, ISC license; the community fork of nih-plug): `nice_export_vst3!`, `nice_export_clap!`, `#[derive(Params)]` with `#[id]`, `#[persist]` state, bundling with `cargo-nice-plug` or its xtask. Check its examples (gain, sine, midi_inverter) for the current API.

Scope:
- A new crate (e.g. `crates/orb-plugin`, cdylib) with the identity, permanent after the first release: name `OpenRB-338`, vendor `OpenRB-338`, CLAP id `io.github.pingw33n.openrb-338`, a VST3 class id generated once (random GUID, written down in the crate), category Instrument|Synth, URL https://github.com/pingw33n/OpenRB-338. Stereo out, one note/MIDI input.
- The engine runs inside the plugin with orb-player's lock-free design (no lock, no allocation in `process`; the garbage queue back to the main thread). The standalone player's 1024-frame call (`CALL_FRAMES`) does not apply: render 64-frame engine blocks for any host buffer size, keep leftover frames, apply each event at the block holding its timestamp, render one block ahead and report a constant 64-frame latency.
- The engine runs at 44.1 kHz only: when the host runs at another rate, resample (a deterministic resampler, its latency added to the reported latency). Offline bounce must be deterministic: nothing depends on the wall clock.
- Host parameters: one per control that is visible in the plugin panel (task-185: hidden are the pattern/song switch, loop switch and range, Rewind, Record, the tempo buttons, File > Save), including the five 808 voice switches (ctrl 11, 14, 17, 19, 21), plus the mode switch (orb Play/Stop; its behavior is the next task). Stepped integers mirroring the u8 controllers; stable ids derived from the device and controller numbers; the panel's own message paths (docs/song_load.md 4.10) carry the changes into the engine.
- State: the loaded .rbs bytes (File > Info shows its tempo etc.), the patterns and controls as edited, the mode, the editor size, and a non-default mod embedded (the built-in Standard kit is never embedded). Several instances are independent.
- File > Open of an .rbs into the instance and the Mods menu's mod loading are in the editor task; this task provides the calls they use (load song bytes, set mod).

Out of scope: host transport and tempo sync, pattern selection from notes (next task), note mode, the editor, drag-out, packaging.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A crate builds a VST3 and a CLAP bundle with the identity above; the VST3 class id is a constant in the source with a comment that it never changes
- [ ] #2 The plugin loads in at least one host (Reaper or a validator) and outputs the pattern playback of a loaded song as audio when the mode is playing
- [ ] #3 Rendering works for host buffer sizes 1, 63, 64, 512 and 4096 frames with identical output at 44.1 kHz (a test), and the plugin reports 64 frames of latency
- [ ] #4 At a host rate of 48 kHz the output is resampled and the reported latency includes the resampler's; two offline renders are identical (a test)
- [ ] #5 Every visible control is a host parameter with a stable id; a test lists the ids and fails when one changes
- [ ] #6 Saving and restoring the state restores song, patterns, controls, mode and an embedded third-party mod (a test)
- [ ] #7 No allocation or lock in process (a counting-allocator test, as orb-player's no_alloc)
- [ ] #8 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
