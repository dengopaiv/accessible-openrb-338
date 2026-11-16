---
id: TASK-209
title: 'Plugin editor: the Standard Mapping (Remote MIDI) setting in the panel'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T10:40:00Z'
updated_date: '2026-11-16T16:01:00Z'
labels:
  - ui
dependencies:
  - TASK-201
  - TASK-207
  - TASK-208
references:
  - backlog task-208
  - backlog task-207
  - docs/plugin.md
  - crates/orb-plugin/src/instance.rs
priority: medium
type: feature
ordinal: 214500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The plugin's Standard Mapping (Remote MIDI) has an on/off (`remote_enabled`, off by default) and a channel (16 by default, any but the device channels 1..4), both plugin state with no host parameter (task-207, docs/plugin.md). Split off from task-201 on 2026-11-16 so the editor does not wait for the UI decision: build the control for them in the plugin editor's panel (task-201) as task-208 decides (its final summary), using `set_remote_enabled` and the channel setter; show the current value; a refused channel cannot be chosen. The user reviews the visual result before merge.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The setting appears in the plugin panel as task-208 decides, shows the current on/off and channel, and changes the plugin state (a test of the state change)
- [ ] #2 Channels 1..4 cannot be chosen
- [ ] #3 The user reviewed it (screenshots or a build) and approved it before merge
- [ ] #4 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Deferred with task-208 (the user, 2026-11-16): no remote MIDI channel in the first plugin release. Taken out of m-7.
<!-- SECTION:NOTES:END -->
