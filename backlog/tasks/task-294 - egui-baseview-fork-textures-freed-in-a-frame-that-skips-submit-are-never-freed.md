---
id: TASK-294
title: >-
  egui-baseview fork: textures freed in a frame that skips submit are never
  freed
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-24T15:55:00Z'
labels:
  - player
  - ui
dependencies: []
references:
  - Cargo.toml
  - docs/plugin.md
priority: low
type: bug
ordinal: 268700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** found during task-292 (2026-11-24); the user asked to record it, unscheduled. In egui-baseview (0.9.0 and the fork github.com/pingw33n/egui-baseview, branch OpenRB-338, commit e3db196, which moved the free loop after queue.submit), src/renderer/wgpu/renderer.rs returns early when `surface.get_current_texture()` gives no frame:
- Occluded (e.g. the window minimized);
- Timeout;
- Outdated or Suboptimal (the window's size changed, e.g. mid-resize; it reconfigures and returns);
- Lost (it recreates the surface and returns).
That return comes before the drawing, the submit and the free loop, so the frame's `textures_delta.free` is dropped. egui lists each texture to free only once, so such a texture stays in GPU memory until the plugin window closes.

**Effect:** a small leak, never a crash. It needs a texture drop (a mod's splash or panel changing) in the same frame as a minimized window, a resize or a lost surface. Each leak is one image, a few MB at most, freed when the window closes.

**Fix sketch:** a second commit on the fork's OpenRB-338 branch keeps the skipped frame's free list (append it to a pending list on the renderer) and frees those ids after the next frame's submit. Then move the [patch.crates-io] rev to the new commit and update docs/plugin.md's fork paragraph. A unit test of the pending list if the code allows. No upstream pull request unless the user asks.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Textures listed for freeing in a frame that returns before submit are freed after the next submitted frame; the fork's branch has the commit and the workspace patch points at it
- [ ] #2 docs/plugin.md records the second fork commit
- [ ] #3 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
<!-- AC:END -->
