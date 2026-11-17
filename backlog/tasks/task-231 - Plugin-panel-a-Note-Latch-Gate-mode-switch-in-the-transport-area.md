---
id: TASK-231
title: 'Plugin panel: a Note/Latch/Gate mode switch in the transport area'
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-17T08:06:00Z'
labels:
  - ui
  - player
  - docs
milestone: m-7
dependencies:
  - TASK-228
references:
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/src/editor.rs
  - crates/orb-ui/src/embed.rs
  - crates/orb-ui-art
  - docs/plugin.md
  - task-213
  - task-228
  - task-221
priority: high
type: feature
ordinal: 600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's idea (2026-11-17): the plugin panel's transport strip has empty space between Play/Stop, Bar and the logo (Fast forward was removed, task-213). Fill it with a 3-position mode switch: Note Mode, Latch Pattern Mode, Gate Pattern Mode (instance.rs `Mode`, today only in the Options menu as radio items). The mode decides what the plugin does with MIDI and a wrong mode is the likeliest "no sound", so it belongs on the panel.

The user settles the design in their own session in this task's worktree (branch task-231-mode-switch). Starting proposal (the supervisor's): between Play/Stop and Bar; a switch in the panel's style labeled NOTE · LATCH · GATE, Note left (the default); click a position to select it, no keys (the plugin's no-keyboard rule, docs/plugin.md); the Options menu radio items stay and follow it; a one-line tooltip per position; optionally Bar shows "---" in Note Mode. New art parts go in orb-ui-art; the standalone app's panel is unchanged (its --screenshot byte-identical).

Write the settled design into this description or the plan before implementing. task-221 (Help > Plugin Usage) should describe the switch.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The design is recorded in the task (placement, look, labels, behavior).
- [ ] #2 The plugin panel shows the switch; clicking a position sets the mode, the same as the Options menu, and both always agree (test).
- [ ] #3 The mode change from the switch is saved with the plugin state like the menu's.
- [ ] #4 The standalone app's --screenshot output is byte-identical to main's; plugin preview screenshots at 1x and 2x.
- [ ] #5 docs/plugin.md describes the switch; cargo test, clippy -D warnings and nightly fmt pass; the user's Reaper check is recorded.
<!-- AC:END -->
