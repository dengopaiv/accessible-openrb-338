---
id: TASK-268
title: >-
  Themes 3/3: the animated pixelating transition between the modern and classic
  skins
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T10:03:00Z'
updated_date: '2026-11-21T10:08:00Z'
labels:
  - ui
milestone: m-8
dependencies:
  - TASK-267
references:
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** Task 3 of the user's themes request (2026-11-21, milestone m-8), after task 2 (View > Themes > Classic / Modern).

**The animation (the user's words, 2026-11-21).**
- **Modern → classic:** the classic panel is in place underneath. The modern panel starts pixelating, down to the classic theme's pixel density (one block per original panel pixel, 624 x 510 across the panel), while fading out by alpha, within about 500 ms.
- **Classic → modern:** the reverse. The modern panel starts fully transparent and fully pixelated, and resolves while fading in.
- The duration is a constant in the code, not a setting; the user may tune it during their review. Use an ease curve and the pixel-size steps the user finds pleasant.

**How (the user's requirement).**
- Pixelate by **scaling**, never by rasterizing the SVG parts again: rasterizing from SVG is expensive.
  - Take the modern panel as an image once at the start of the transition.
  - Make it smaller (box-filter it down to the step's resolution), then scale it back up with **nearest** filtering.
  - Do this on the GPU or with cheap CPU scaling of one image, not per part.
- Getting the modern panel as one image needs an offscreen render or a frame capture. Choose and document one, for example:
  - egui's viewport screenshot of a frame (asynchronous: one frame of latency), or
  - rendering the panel into an offscreen wgpu texture through a paint callback.
  For classic → modern, the modern image must be made before it is shown.
- Whether the modern panel may freeze (a still image) during the 500 ms, or must keep its lights and meters live, is the user's call at review. A still image is acceptable to start.
- The desk and the shadows don't animate. Input is ignored, or goes to the target skin, during the transition; say which.
- Standalone only.

**Testing.**
- The transition's pure parts (the step and alpha schedule, the downscale and upscale) are unit tested.
- A test runs a switch both ways and checks the end states draw the target skin.

The user is the first reviewer: give them a dev app and say where the duration and step constants are, before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Switching modern to classic pixelates the modern panel to the classic pixel density while fading it out over about 500 ms (a code constant), and classic to modern runs in reverse
- [ ] #2 Pixelation uses image downscale plus nearest upscale of one captured panel image, never SVG rasterization per step; the capture method is documented
- [ ] #3 The desk and shadows stay still; input during the transition is defined and documented
- [ ] #4 The schedule and scaling are unit tested and a both-way switch test passes; docs/ui_spec.md describes the transition
- [ ] #5 The user reviewed the build (and tuned the constants if wanted) before the agent review
- [ ] #6 The pixelation's target density is the active classic set's: 624*N blocks across for a set at scale N (task-267), not a fixed 624
<!-- AC:END -->
