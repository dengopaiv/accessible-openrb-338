---
id: TASK-201
title: >-
  Plugin editor: the orb panel in the plugin window, with resizing and the
  plugin's panel changes
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-197
  - TASK-198
references:
  - backlog task-185 (final summary)
  - https://codeberg.org/RustAudio/nice-plug
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/layout.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 214000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The plugin (task-197, task-198) gets its editor: orb-ui's panel inside the host's plugin window, as task-185's final summary decides (read it first). The user reviews the visual result before merge.

- **Embedding:** nice-plug's `nice-plug-egui` adapter (egui 0.37, as orb-ui; egui-baseview with the `wgpu` feature). Share the panel code with orb-ui (factor out what is eframe-specific) rather than copy it.
- **The whole window is the panel:** no metallic filler background around it.
- **Panel changes in the plugin:** hidden: the pattern/song switch, the loop switch and range, Rewind, Record, File > Save. The tempo display stays, read-only (its up/down buttons hidden), showing the tempo orb plays at (the host's, or the song's when the host reports none). orb's Play/Stop is the mode switch (task-198). Everything else works as in the standalone app (knobs, patterns, mixer, effects, File > New/Open/Info, Mods menu, Help).
- **Menus:** no native menu bar in a plugin window: the menus are drawn in the panel. File dialogs (Open, mod loading) must work inside hosts (check rfd from a plugin window; fall back to something safe if it does not).
- **LEDs:** MIDI LED on incoming host MIDI; Sync LED lit while orb follows the host position (task-198's snapshot flags).
- **Sizing,** aspect-locked to the panel (layout.rs PANEL_W/H), saved in the plugin state:
  1. host-driven resizing: `Editor::resize_hint()` returns a `ResizeHint` with the panel's aspect ratio, and `EditorHandle::set_size()` applies the host's size;
  2. the plugin's own resize corner, invisible: only the mouse cursor changes over it; dragging asks the host to resize (VST3 `IPlugFrame::resizeView`, CLAP `gui.request_resize`, through nice-plug).
  No zoom presets.
- Keyboard shortcuts: hosts may take keys; best effort.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The editor opens in Reaper (VST3 and CLAP) on macOS and shows the panel filling the window, with the plugin's hidden controls absent and the tempo read-only
- [ ] #2 Host-driven resizing keeps the panel's aspect; the invisible corner changes the cursor and resizes the window; the size is restored with the project
- [ ] #3 In-panel menus work, including File > Open of an .rbs and loading a mod, inside the host
- [ ] #4 MIDI and Sync LEDs follow the snapshot flags
- [ ] #5 The standalone app is unchanged (its tests pass; a screenshot of it is unchanged)
- [ ] #6 The user reviewed the plugin's look and behavior (screenshots or a build) and approved it before merge
- [ ] #7 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
