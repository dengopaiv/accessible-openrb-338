---
id: TASK-228
title: >-
  Plugin: File > Open keeps the current mode (no switch to Latch); Note Mode
  default, kept in the state
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-17T07:48:00Z'
updated_date: '2026-11-17T09:28:00Z'
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
- [x] #1 File > Open of a song in Note Mode leaves the instance in Note Mode; in Latch or Gate it stays in that mode (test).
- [x] #2 File > New keeps the mode (test).
- [x] #3 A new instance's mode is Note Mode (test).
- [x] #4 The mode survives a state save and load for each of latch, gate and note, through CLAP and VST3 state (test).
- [x] #5 docs/plugin.md (and docs/midi.md if it mentions it) no longer say File > Open switches to Latch; cargo test, clippy -D warnings and nightly fmt pass; the user's Reaper check is recorded.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Remove the Latch switch in Editor::load_song. 2. Tests: editor.rs open/new keep each mode; state.rs default Note + each mode through save/restore (the persisted fields both CLAP and VST3 state use). 3. Docs plugin.md (3 places); midi.md has no mention.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aab0f7e37444cba80

Removed the switch (editor.rs load_song). Tests: tests/editor.rs (open and new keep Latch/Gate/Note), tests/state.rs the_mode_defaults_to_note_and_is_kept_in_the_state. CLAP and VST3 state share nice-plug's persisted fields (serialize_fields/deserialize_fields), so one test covers both. cargo test 858 pass, clippy -D warnings and nightly fmt clean; bundles built, scripts/validate-plugin.sh all passed. Bundles: target/bundled/OpenRB-338.clap and .vst3 in this worktree. Reaper check pending.

Review round 1 (stale wording only): fixed the Embed::load_song doc (orb-ui embed.rs), editor_preview.rs header, tests/common with_song doc, and renamed the editor test to file_open_and_new_keep_the_mode; rebased onto main f1a9aa0; bundles rebuilt.

User's Reaper check, 2026-11-17, on the bundles of the previous commit: passed ("looks fine"): a new instance starts in Note Mode; File > Open and File > New keep Note, Latch and Gate; the mode survives a project save and reload. Correction to the round 1 note: the base after that round was main eeeebdb, and the final rebase is onto main c3b41d6 (the note above says f1a9aa0).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
File > Open and File > New keep the instance's mode (the Latch switch is removed from Editor::load_song); a new instance starts in Note Mode and the mode is kept in the state, pinned by tests (editor.rs open/new for each mode, state.rs default and save/restore). docs/plugin.md updated; docs/midi.md had no mention. Stale docs in orb-ui embed.rs, editor_preview.rs and tests/common fixed in review round 1. cargo test 858 pass, clippy -D warnings and nightly fmt clean, validate-plugin.sh passed. The user's Reaper check passed 2026-11-17. Branch worktree-agent-aab0f7e37444cba80, one commit, rebased onto main c3b41d6.
<!-- SECTION:FINAL_SUMMARY:END -->
