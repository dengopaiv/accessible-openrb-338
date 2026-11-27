---
id: TASK-241
title: Remove the code the app shared with the removed plugin
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-27T20:37:00Z'
labels:
  - ui
  - engine
  - player
  - docs
milestone: m-7
dependencies:
  - TASK-231
references:
  - crates/orb-ui/src/embed.rs
  - crates/orb-ui/src/routing_window.rs
  - crates/orb-player/src
  - crates/orb-engine/src
  - crates/orb-ui-art
  - docs/midi.md
priority: medium
type: chore
ordinal: 240000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user decided on 2026-11-27, after task-231 removed upstream's VST3 plugin: "Remove it. We don't need unnecessary code. If it doesn't work in our program, it shouldn't be there." task-231 left the code the app shared with the plugin in place, to keep upstream merges easy (its notes list it); the user accepts that later upstream merges will conflict more in exchange. The same day the user chose to keep the MIDI layer (orb-player's midi module: the byte parser and StandardMapping, docs/standard_midi_mapping.md, docs/midi.md), so that the standalone app can get a MIDI input port later; it stays even though only the plugin fed it.

What goes: everything that only the removed plugin reached and that does nothing in the standalone OpenRB-338 app or the orb CLI. Starting points (task-231's notes, "Stays"):
- orb-ui: the is_plugin / plugin branches in app.rs, menu.rs and elsewhere; the embedding seam (embed.rs, session.rs) where only a host window used it; routing_window.rs with Options > Output Routing (has_routing, plugin only); the plugin's mode/cursor code and tests/plugin_mode_cursor.rs; portable.rs's knowledge of the .vst3 bundle path (folder_of and its tests).
- orb-ui-art: the plugin-logo parts (plugin_logo_vst3, bg_transport_plugin, PluginFormat) and their source art, if nothing in the app draws them.
- orb-player: the per-device and per-drum outputs (upstream's task-256 and task-300) and the host-event code, where the standalone does not use them.
- orb-engine: the note-mode paths (host notes playing the devices, the note clock, Gate Pattern Mode), only where no standalone or CLI path reaches them. An engine change is checked against the reference corpus (AGENTS.md).
- The standalone/default cargo feature split, if it exists only for the plugin.
- Docs that describe the plugin's behavior as present (docs/midi.md section 5, docs/ui_spec.md, docs/standard_midi_mapping.md, README): reword to say the plugin is not part of the fork, or remove the plugin-only parts.

What stays: the MIDI layer above (the user's choice), and anything the standalone app or orb reaches. When it is unclear whether something is used, find a caller from the app's or the CLI's entry points; if a part's fate is a real choice rather than a fact, make a Decide: task (AGENTS.md) and leave the part in place.

Ground rules: with screen reader mode off the app behaves as before; with it on, nothing in crates/orb-ui/src/a11y changes behavior. Machine load: one cargo build or test at a time (the machine overloads with parallel builds).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No code remains that only the removed plugin reached: every is_plugin branch, embed/session seam used only by a host, Output Routing (routing_window.rs), the per-device outputs and host-event code of orb-player, the note-mode paths of orb-engine and the plugin-logo art are gone or the notes name the standalone or CLI caller that keeps each
- [ ] #2 orb-player's MIDI layer (the parser and StandardMapping) and its tests are kept and still pass
- [ ] #3 The engine renders as before: orb compare --manifest tests/refs/manifest.json --set diag,shipped reports all equal (153 of 153), and corpus_golden passes
- [ ] #4 cargo build --release, cargo test --release --workspace, cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check pass; the a11y tests of orb-ui pass unchanged
- [ ] #5 No doc describes the plugin's behavior as present in the fork; CHANGELOG.md's next section says, in plain language, that unused code from the plugin was removed, if a user can notice it (for example Options > Output Routing gone)
<!-- AC:END -->
