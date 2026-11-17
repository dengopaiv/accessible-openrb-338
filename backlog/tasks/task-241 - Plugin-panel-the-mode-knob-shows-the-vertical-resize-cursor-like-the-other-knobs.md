---
id: TASK-241
title: >-
  Plugin panel: the mode knob shows the vertical-resize cursor like the other
  knobs
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T17:35:00Z'
labels:
  - ui
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/embed.rs
  - crates/orb-ui/src/app.rs
  - task-231
priority: medium
type: bug
ordinal: 500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user found (2026-11-17, Ableton Live): every knob on the panel changes the mouse cursor (crates/orb-ui/src/app.rs:1281-1283, `knob`: CursorIcon::ResizeVertical while hovered or held), but the plugin's Note/Latch/Gate mode knob (task-231; crates/orb-ui/src/embed.rs `mode_selector`, the drag area `hit`, `ui.interact(hit, Id::unique("orb mode knob"), Sense::drag())`) leaves the cursor unchanged.

Fix: in `mode_selector`, set `CursorIcon::ResizeVertical` when the knob's response is hovered, pressed or being dragged, the same condition as `knob` in app.rs (keep the cursor during a drag that leaves the knob). Only the knob's drag area, not the row labels (the panel's buttons set no cursor). Not when the selector is inactive (file browser, prompt, error box open: the function returns before the interaction already).

Add a test (crates/orb-ui, where the embed/mode selector tests are, e.g. embed.rs tests or the plugin editor tests) that hovers the knob's center and checks the frame's output cursor is ResizeVertical, and that hovering a mode row is not.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Hovering or dragging the mode knob shows the vertical-resize cursor, as the other knobs do; the mode rows don't change the cursor.
- [ ] #2 A test checks the cursor for the knob and a row.
- [ ] #3 cargo test, cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check pass.
- [ ] #4 The user's check in a DAW (Ableton Live or REAPER): the cursor changes over the mode knob.
<!-- AC:END -->
