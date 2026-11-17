---
id: TASK-234
title: 'Plugin: the window''s scale jumps around while resizing in Ableton Live'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-17T11:09:00Z'
labels:
  - player
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/editor.rs
  - docs/plugin.md
  - task-224
  - ~/Documents/ableton_resize_weirdness.mp4
priority: low
type: bug
ordinal: 240000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's report (2026-11-17), Ableton Live 12 Lite on macOS, VST3: when the plugin window is resized up or down, the panel's scale jumps around instead of following the window smoothly. Recording: ~/Documents/ableton_resize_weirdness.mp4 on the user's machine (extract frames with ffmpeg into a scratch folder to see it). REAPER does not show it (task-224 fixed REAPER's resize recursion: crates/orb-plugin/src/editor.rs HostProxy, docs/plugin.md section 10).

Not a release blocker (the user): task-215 and task-230 do not depend on it.

Investigate how Live drives the VST3 view's size (checkSizeConstraint, onSize, canResize, the editor's request_resize and the aspect/scale logic in editor.rs and baseview 0.4.2) and why the scale oscillates: e.g. a round trip between the host's size and the editor's own resize request, rounding of the scale to steps, or a stale size answered to Live. Record the sequence of host calls in the notes (a debug log behind an env var is fine, removed or off by default after).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The cause is found and recorded in the notes and docs/plugin.md section 10, with the host call sequence that produces it.
- [ ] #2 Resizing in Live scales the panel smoothly, up and down, with no jumps (the user's Live check), and REAPER's resizing is unchanged (tests/clap_gui.rs and the editor tests still pass; the user's REAPER check).
- [ ] #3 A test reproduces the host's call sequence where feasible.
- [ ] #4 cargo test, clippy -D warnings and nightly fmt pass; validate-plugin.sh passes.
<!-- AC:END -->
