---
id: TASK-207
title: >-
  Plugin: device channels 1..4 and the Standard Mapping off by default (channel
  16 when on)
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-16T09:43:00Z'
updated_date: '2026-11-16T10:29:00Z'
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
- [x] #1 Pattern-selection notes act on channels 1..4 (tests updated); channels 5..16 do not select patterns
- [x] #2 A new instance has the Standard Mapping off; turning it on gives channel 16; channels 1..4 are refused when set and reset with last_error on restore, keeping the song (tests)
- [x] #3 docs/plugin.md describes the channels, the default and the host MIDI learn; docs/midi.md and docs/standard_midi_mapping.md do not contradict it
- [x] #4 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. instance.rs: SECTION_CHANNELS 0..=3 (MIDI 1..4); Doc gets remote_enabled (default off) and remote_channel default 15 (MIDI 16); Shared gets the enabled flag; set_remote_enabled/remote_enabled; restore of a device channel resets to 16 with last_error, keeping the song. 2. core.rs: section notes on channels 0..=3 (ROWS[c]); the Standard Mapping only when enabled. 3. Update sync.rs tests (channels, default off, on -> channel 16, refused/restore). 4. docs/plugin.md (channels, default, no migration, host MIDI learn incl. mode as play/stop), midi.md, standard_midi_mapping.md consistency. 5. test, clippy, fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a395bc852f1b7ba10

Evidence: cargo test (workspace) all ok, 0 failed; cargo clippy --all-targets -D warnings clean; nightly fmt --check clean. sync.rs: section notes on 0-based 0..3, channels 4 and 15 do not select patterns; mapping off by default (CC on ch 16 ignored), on -> ch 16, device channels 1..4 refused, restore of channel 3 -> 16 with last_error, song/patterns/controls kept. A disabled mapping also drops non-section messages. The render determinism test turns the mapping on (CC on ch 16). Docs: plugin.md sections 6/7/8 incl. no-migration line and Reaper Param > Learn incl. mode g1; midi.md and standard_midi_mapping.md note the plugin's default off/ch 16 (library default channel 1 unchanged). Not verified in a real host (no Reaper here).

Review round 1 (send back): fixed commit subject length, test count (cargo test 751 pass), [G] 0x41C293 tag and rewrap in docs/plugin.md, new restore test for absent keys (off, channel 16) and device-channel boundaries 0, 2, 3 (MIDI 1, 3, 4) in sync.rs. Rerun: cargo test 751 pass 0 failed, clippy and nightly fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Plugin device channels are now MIDI 1..4 (303 #1, 303 #2, 808, 909); the Standard Mapping is off by default (Doc/Shared remote_enabled, set_remote_enabled), channel 16 default, device channels refused on set and reset to 16 with last_error on restore keeping the song. Transport notes still ignored. docs/plugin.md documents channels, default, no migration, and host MIDI learn (Reaper Param > Learn, mode as play/stop). Tests updated in crates/orb-plugin/tests/sync.rs; workspace cargo test, clippy, fmt pass. No UI (task-208/201).
<!-- SECTION:FINAL_SUMMARY:END -->
