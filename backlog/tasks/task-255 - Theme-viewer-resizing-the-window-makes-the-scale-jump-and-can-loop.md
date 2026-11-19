---
id: TASK-255
title: 'Theme viewer: resizing the window makes the scale jump and can loop'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-19T14:32:00Z'
labels:
  - ui
milestone: m-5
dependencies:
  - TASK-253
references:
  - crates/orb-ui/src/theme_view.rs
  - crates/orb-ui/tests/theme_view.rs
  - docs/ui_spec.md
  - task-187
priority: high
type: bug
ordinal: 203700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-19 (macOS, the dev app of task-253): while resizing the Mods > Theme Viewer window, the panel's scale jumps strangely, and the window can get into a weird loop (it keeps changing size by itself). Screen recording: ~/Documents/theme_viewer_resize_bug.mp4 (outside git; ffmpeg is not installed: extract frames with a small Swift/AVFoundation script, AVAssetImageGenerator, into the scratchpad, or ask the supervisor).

Likely cause (the supervisor's reading, verify): `App::theme_viewer` in crates/orb-ui/src/theme_view.rs (~line 930) builds the viewport every frame with `.with_inner_size(v.size...)`, and `v.size` is updated every frame from the window's current `inner_rect` (~line 940, stored at ~964; added in task-187 round 3 so a Linux reopen keeps the size). egui treats a changed `ViewportBuilder` field on an existing viewport as a command and applies it to the window, so each frame the window is told to go back to the size it had one frame earlier: during a live resize this fights the user's drag (the jumps), and with the OS's own resize events it can oscillate (the loop). The fix should give the builder a size only when the window is created (e.g. remember the size at open/reopen time in a separate field that stays constant while the window lives), keep the Linux reopen-at-same-size behavior of task-187, and keep the min size.

Also check the letterboxing math (`draw`, ~line 800) for a jump between fits during the resize, and that the viewer repaints on resize without requesting a size.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Resizing the viewer window by dragging scales the panel smoothly with no jumps and the window never changes size by itself (checked by the user on macOS)
- [ ] #2 The viewport builder sets the inner size only when the window is created; a test checks that live size changes do not produce a resize command/builder change for an open viewer
- [ ] #3 A reopened viewer (Linux path of task-187) still opens at the last size; its test still passes
- [ ] #4 cargo test, clippy -D warnings and nightly fmt pass; docs/ui_spec.md section 21 updated if the behavior description changes
<!-- AC:END -->
