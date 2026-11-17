---
id: TASK-228
title: >-
  Plugin: File > Open keeps the current mode (no switch to Latch); Note Mode
  default, kept in the state
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T07:48:00Z'
labels:
  - player
  - ui
  - docs
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/editor.rs
  - crates/orb-plugin/src/instance.rs
  - docs/plugin.md
  - task-185
  - task-218
priority: high
type: enhancement
ordinal: 500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's decision (2026-11-17): opening a song in the plugin must not change the mode. Today File > Open switches Note Mode to Latch Pattern Mode (task-185's design, kept by task-218 for a pattern mode): crates/orb-plugin/src/editor.rs, Editor load_song, the `if open && !self.params.mode().patterns() { self.params.set_mode(Mode::Latch) }` block. Remove that switch: File > Open and File > New both keep whatever mode the instance is in.

A new instance starts in Note Mode, and the mode is saved and restored with the plugin state. Both are believed to be true already (instance.rs `Mode` has `#[default] Note`; the state carries the strings latch/gate/note since task-218), so this task pins them with tests rather than re-implementing them. No compatibility work for old states (the plugin is not released).

Update docs/plugin.md where it says File > Open loads the song in Latch Pattern Mode (around line 833 and the File menu bullet near line 664, "sets ... note mode to Latch Pattern Mode (task-185; a pattern mode stays, task-218)"), and any mention in docs/midi.md. Also check the Options menu / mode text in the editor for wording that promises the switch.

Visual/plugin change: the user checks it in Reaper before merge.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 File > Open of a song in Note Mode leaves the instance in Note Mode; in Latch or Gate it stays in that mode (test).
- [ ] #2 File > New keeps the mode (test).
- [ ] #3 A new instance's mode is Note Mode (test).
- [ ] #4 The mode survives a state save and load for each of latch, gate and note, through CLAP and VST3 state (test).
- [ ] #5 docs/plugin.md (and docs/midi.md if it mentions it) no longer say File > Open switches to Latch; cargo test, clippy -D warnings and nightly fmt pass; the user's Reaper check is recorded.
<!-- AC:END -->
