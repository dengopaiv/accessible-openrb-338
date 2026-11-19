---
id: TASK-255
title: 'Theme viewer: resizing the window makes the scale jump and can loop'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-19T14:32:00Z'
updated_date: '2026-11-19T14:51:00Z'
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
- [x] #1 Resizing the viewer window by dragging scales the panel smoothly with no jumps and the window never changes size by itself (checked by the user on macOS)
- [x] #2 The viewport builder sets the inner size only when the window is created; a test checks that live size changes do not produce a resize command/builder change for an open viewer
- [x] #3 A reopened viewer (Linux path of task-187) still opens at the last size; its test still passes
- [x] #4 cargo test, clippy -D warnings and nightly fmt pass; docs/ui_spec.md section 21 updated if the behavior description changes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add open_size (size at window creation) to the viewer state, set in show_theme_viewer on open and on Linux reopen; the builder uses it, never the live size. Test: builder size constant across live resizes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a1e653ca6d1d6651c

Cause confirmed in the cargo registry: egui 0.37 ViewportBuilder::patch (viewport.rs:771-775) pushes ViewportCommand::InnerSize when the new builder inner_size differs from the stored one; eframe glow_integration.rs:1661 / wgpu_integration.rs:1438 call it every frame for an existing immediate viewport and apply the commands. theme_viewer_ui gave with_inner_size(v.size), v.size = live inner_rect one frame earlier, so every frame resized the window back to its previous size (jumps during the drag, oscillation with the OS events). Not verified from the video (no frames extracted). Letterbox math (View::fit, app.rs:46) is a continuous min-scale fit with centre rounding: no jump source. Viewer repaints on resize on its own; no size is requested. Fix: new field open_size, set from size in show_theme_viewer at open and Linux reopen; builder uses it (min size unchanged). Test a_live_resize_does_not_change_the_builders_size (orb-ui tests/theme_view.rs); the task-187 reopen test still passes. cargo test all ok, clippy -D warnings clean, nightly fmt clean. Dev app for the user's check (AC 1): target/OpenRB-338.app in the worktree.

The user checked the dev app on macOS on 2026-11-19: resizing looks good.

Review round 1: send back (amend only): the commit body said the user's check was still open (now the approval of 2026-11-19), and docs/ui_spec.md section 21 had a 120-column line (rewrapped). Done.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The viewer's ViewportBuilder now gets its inner size only at window creation (new open_size field, set at open and at the Linux reopen), so egui's ViewportBuilder::patch no longer sends an InnerSize command each frame; that command fought the drag (cause confirmed in egui 0.37 viewport.rs:771 and eframe glow/wgpu integrations). Branch worktree-agent-a1e653ca6d1d6651c. Test a_live_resize_does_not_change_the_builders_size added; cargo test, clippy -D warnings and nightly fmt pass. The user checked the macOS dev app on 2026-11-19: looks good.
<!-- SECTION:FINAL_SUMMARY:END -->
