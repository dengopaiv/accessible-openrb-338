---
id: TASK-207
title: >-
  Plugin: device channels 1..4 and the Standard Mapping off by default (channel
  16 when on)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T09:43:00Z'
labels:
  - player
  - docs
milestone: m-7
dependencies:
  - TASK-198
references:
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/tests/sync.rs
  - docs/plugin.md
  - docs/midi.md
  - backlog task-185 (final summary)
priority: medium
type: enhancement
ordinal: 211700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user changed task-185's channel layout on 2026-11-16. In the plugin (crates/orb-plugin, task-197/198):

- **Device channels 1..4** (1-based): 303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4, instead of 2..5. They carry the pattern-selection notes (pattern mode, task-198) and later the played notes (note mode, task-200). A keyboard or DAW track on its default channel 1 then reaches 303 #1 without setup.
- **The Standard Mapping (Remote MIDI) is off by default** in the plugin, as the original's Remote MIDI Control defaulted to off. When it is turned on, its channel defaults to 16; any channel outside 1..4 is allowed, a device channel is refused (set) or reset to the default with last_error (restore), as task-198 does now for 2..5. Its transport notes stay ignored (task-198, docs/plugin.md 8.2). The on/off and the channel are plugin state (no host parameter); the UI for them is task-208's decision and task-201's work.
- **Saved states from before this change:** the plugin is not released, so no migration is needed; say so in docs/plugin.md.
- **docs/plugin.md:** the new channels and default; a line that the main way to map a controller is the host's MIDI learn of the plugin's parameters (in Reaper: the FX window's Param > Learn), including the mode parameter as a play/stop button.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Pattern-selection notes act on channels 1..4 (tests updated); channels 5..16 do not select patterns
- [ ] #2 A new instance has the Standard Mapping off; turning it on gives channel 16; channels 1..4 are refused when set and reset with last_error on restore, keeping the song (tests)
- [ ] #3 docs/plugin.md describes the channels, the default and the host MIDI learn; docs/midi.md and docs/standard_midi_mapping.md do not contradict it
- [ ] #4 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
