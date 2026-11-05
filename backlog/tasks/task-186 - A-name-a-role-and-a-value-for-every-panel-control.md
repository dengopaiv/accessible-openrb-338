---
id: TASK-186
title: A name, a role and a value for every panel control
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:31:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-183
  - TASK-181
references:
  - assets/ui/layout.tsv
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/model.rs
  - docs/ui_spec.md
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: feature
ordinal: 209000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-2; the analysis is docs/accessibility.md. Read both before starting.

- Why: today 250 of the 254 focusable elements have no name and no role
  (docs/accessibility.md 1.1).
- What: `crates/orb-ui/src/a11y/names.rs`: for each layout id its spoken name ("Cutoff",
  "Bass drum level", "Channel 3 mute") and its section's name ("Bass Line 1", "TR-808"), as a
  table checked against `layout.tsv` by a test; value text per kind (the 0..127 number for
  knobs and faders until A11Y-14 gives their units, words for switches and pan, the displays'
  units); roles: Slider (knob,
  fader), CheckBox or ToggleButton (toggle), RadioButton (radio), Button (trigger), the switch
  as a two-state control with its two words. Non-interactive parts (LEDs, meters, the step
  parts, the click areas, the mod picture) get no focus: the panel's `ui.interact` for them
  uses a sense without focus, or they get no node. The node data is written with
  `Context::accesskit_node_builder` or `Response::widget_info` in the place each control is
  drawn. Note that egui 0.37 deprecates `WidgetType` (now `egui::Role`) and `Id::new`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A test walks `layout.tsv`: every interactive part (every kind except led, meter, steps, area, modview, display) has a node with a non-empty name, the role of its kind and, for a value control, a value; no two nodes of one section share a name.
- [ ] #2 No LED or meter node can take the focus; the number of focusable nodes is recorded in the baseline test (A11Y-0) and is lower than today's 254.
- [ ] #3 `tools/uia_tree.ps1` on the release build shows the names (recorded in the notes).
- [ ] #4 `crates/orb-ui/tests/frame_cost.rs` also measures a frame with AccessKit on (it enables it on its context; today it does not) and prints the time of building the tree; the shape, mesh and vertex bounds it asserts still hold, and the times before and after are in the notes.
- [ ] #5 With screen reader mode off, the keyboard and interaction tests pass unchanged.
<!-- AC:END -->
