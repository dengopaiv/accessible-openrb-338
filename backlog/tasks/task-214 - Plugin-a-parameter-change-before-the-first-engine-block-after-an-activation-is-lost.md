---
id: TASK-214
title: >-
  Plugin: a parameter change before the first engine block after an activation
  is lost
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T11:41:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/tests/editor.rs
priority: low
type: bug
ordinal: 223000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Found in task-201 (tests/editor.rs, a_knob_sets_its_parameter_through_the_host). An activation (instance::core -> OrbParams::activate) queues Command::Load of the engine built with the parameter values of that moment. Core::process_with applies a changed parameter right after the call's first frame through Audio::apply_now, but the first engine block is rendered only when its first frame is due (64 frames later, docs/plugin.md section 4); its Audio::drain then applies the queued Load, which replaces the engine with the activation's values. Core::apply has already recorded the new value in its last array, so it is never sent again: the engine plays the old value while the host parameter holds the new one, until the parameter changes again or the next activation.

Trigger: a host or editor parameter change between an activation and the end of the first 64 host frames. Rare in practice, but an automation lane starting at the transport start of an offline bounce could hit it.

Fix to evaluate: drain the activation's Load before the first call (in Core::new, on the main thread, since nothing else can be queued yet), or re-send every parameter whose value the engine does not hold after a Load is drained. Check tests/timing.rs and the bit-exact tests stay as they are.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A parameter changed in the first host buffer after an activation is played by the engine (a test in crates/orb-plugin/tests)
- [ ] #2 cargo test -p orb-plugin passes; the timing and sync tests are unchanged
<!-- AC:END -->
