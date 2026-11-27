---
id: TASK-241
title: Remove the code the app shared with the removed plugin
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-27T20:37:00Z'
updated_date: '2026-11-27T21:17:00Z'
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
- [x] #1 No code remains that only the removed plugin reached: every is_plugin branch, embed/session seam used only by a host, Output Routing (routing_window.rs), the per-device outputs and host-event code of orb-player, the note-mode paths of orb-engine and the plugin-logo art are gone or the notes name the standalone or CLI caller that keeps each
- [x] #2 orb-player's MIDI layer (the parser and StandardMapping) and its tests are kept and still pass
- [x] #3 The engine renders as before: orb compare --manifest tests/refs/manifest.json --set diag,shipped reports all equal (153 of 153), and corpus_golden passes
- [x] #4 cargo build --release, cargo test --release --workspace, cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check pass; the a11y tests of orb-ui pass unchanged
- [x] #5 No doc describes the plugin's behavior as present in the fork; CHANGELOG.md's next section says, in plain language, that unused code from the plugin was removed, if a user can notice it (for example Options > Output Routing gone)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Trace from the entry points (orb-ui main.rs, App::new; orb-cli main.rs) what reaches each plugin part; App::embedded (the only constructor of App::plugin) has no caller since the plugin crate went, so every is_plugin branch is dead.
2. orb-ui: delete embed.rs, routing_window.rs, tests/plugin_mode_cursor.rs; drop App::plugin, is_plugin branches (app.rs, menu.rs, controller.rs, drum/pattern editors, skin), Cmd::OutputRouting and Cmd::SetPluginMode, the note-mode lights (Lit) in the skins, the plugin= transport background and the hidden-control list; replace link's Port trait (host seam) by Player itself, drop with_port and reset_view; session's Opened::file if only the plugin read it; portable.rs folder_of's .vst3 bundle path; the app/default feature split (only for the plugin's eframe-less build).
3. orb-ui-art: drop bg_transport_plugin, plugin_logo_vst3, mode selector parts, led_mode_on, lit808/litsel808/led909*_lit parts and the layout's plugin=/lit= params; regenerate assets/ui with ui-art gen and check that only those files change.
4. orb-player: drop Command::Position, NoteClock, Relocate, Note, NotesOff, Gate; Snapshot sounding, gate_closed, host_sync, midi_in; Audio's host API (with_call_frames, drain, apply_now, render_call, taps, set_routing, set_host_sync, midi_received, last_snapshot) where no standalone/CLI caller; tests note_mode.rs and the plugin parts of garbage.rs/no_alloc.rs. Keep midi.rs and its tests.
5. orb-engine: drop live notes (tb303/tr808/tr909 note mode and note_tests), the note clock, the pattern gates and their fade, Sounding/hits/audible/peak, the taps, OutputRouting, render_split/Sink/DrumOutput/OUTPUTS/process_split, set_position/relocate_device if only the plugin called them; keep every store and operand order of the export path (render_split with all outputs to the bus is v.process(bus) in order).
6. Docs: midi.md section 5, ui_spec.md, standard_midi_mapping.md, README, rewire.md and others: plugin behavior not described as present; CHANGELOG next section (Options > Output Routing was never shown in the standalone, so say unused code went).
7. Verify one cargo job at a time: build --release, test --release --workspace, clippy, nightly fmt, corpus_golden, orb compare --set diag,shipped (153/153); ORB_CD_ISO unset, note the skips.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a49ccdcc13870a67b

Evidence (2026-11-27, Windows 11, one cargo job at a time):
- Callers traced from the entry points. orb-ui main.rs builds the app with App::new / App::with_link and Link::new (Player::new); App::embedded was the only place that set App::plugin, and nothing calls it since task-231 removed the plugin crate, so every is_plugin()/plugin branch was dead. Link::with_port, Link::reset_view and the Port trait's other implementor were the plugin's only. orb-cli uses Engine::render (export path) only. The player's Command::Position, NoteClock, Relocate, Note, NotesOff and Gate were sent only by the plugin (grep over crates: no other sender); Audio::with_call_frames, apply_now, render_call as a public step, enable_taps, taps, set_routing, set_host_sync, midi_received, last_snapshot, loads, newest, call_frames, output had no caller outside the plugin and its tests.
- orb-ui removed: embed.rs (App::embedded, Embed trait, PluginMode, PluginFormat, mode selector, note-mode Lights, host position Bar display, resize corner and grip, pattern drag-out, PLUGIN_HIDDEN), routing_window.rs (Options > Output Routing, has_routing), tests/plugin_mode_cursor.rs; App::plugin, routing_open/serial; every is_plugin branch in app.rs, menu.rs (File/Edit/View/Options/Help items, asks_to_save, open, fresh, mod change, command filter, Paste Pattern), controller.rs (hidden controls, drag_out), drum_editor.rs and pattern_editor.rs (saved place, Spec::keep, now always true so the field went), skin Frame's lit/plugin/hidden and the lit selection marks (skin::mark returns the selection only); Cmd::OutputRouting, Cmd::SetPluginMode; link.rs Port trait (Link holds the Player itself now), with_port, reset_view, the gate_closed step-light filter; session::Opened::file (the plugin kept the bytes in its state); portable.rs folder_of's .vst3 bundle path and its two tests; the app/default cargo feature split (eframe now a plain dependency; the cfg(feature = "app") gates in app.rs, lib.rs, system_dialog.rs go); the plugin sentences of the drum editor's help texts (Input::what; not shown in the UI, only listed and tested) and comments.
- orb-ui-art removed: bg_transport_plugin, plugin_logo_vst3, mode_label_0..2_on/off, led_mode_on, led_transport_on (the plugin's Sync/MIDI LEDs), lit808_* and litsel808_* (16 each), led909_lit, led909ac_lit, PLUGIN_BAR/LOGO/MODE_* and the layout's plugin=, lit=, litsel= params. `cargo run -p orb-ui-art -- gen` removed exactly those 46 SVGs and rewrote layout.tsv (42 lines, params only); no other SVG changed. assets/ui/README.md updated.
- orb-player removed: the six plugin commands, Snapshot::sounding, gate_closed, host_sync, midi_in, the Audio host API listed above (drain and render_call are private steps of the call now, CALL_FRAMES a constant), tests/note_mode.rs, the note-mode block of tests/no_alloc.rs; tests/garbage.rs (task-225) keeps its two cases through the queue (run) instead of apply_now. Kept: midi.rs (Parser, StandardMapping) and its tests, the user's choice; its module doc reworded (no source feeds it in the fork yet).
- orb-engine: the plugin's engine changes were reverse-applied commit by commit (script: git show C -- crates/orb-engine | git apply -R, newest first: 7052216 comments, 2ea3e21 per-drum outputs and OutputRouting, 4ad0ac6 303 notes off, 82e8836 taps (one 3-way conflict in Engine's fields, resolved keeping task-306's undo field), 79e0033 909 note flams, 961e4cc note clock, c91b55f sounding hits, f84d164 Gate Pattern Mode, f50af89 note mode and the three note_tests.rs, 8b2e54c Engine::set_position/Sequencer::set_pos and relocate_device). Check: `git diff --cached 8b2e54c^ -- crates/orb-engine/src` now equals the diff of d2c2b4b (Initialize Song/Loop, task-306) plus 4e3d22b's merge_tests line (test profile), line for line: the engine source is the pre-plugin engine with the two non-plugin commits on it. So the export path's stores and operand order are those of the engine before the plugin (the drum voices add with `out[i] = st(out[i] as f64 + x)` again, as before task-300's Out trait). Original facts kept in docs: sub_1001F8B0 / sub_1001C2F4 0x1001C38F (TopHat host sync) in docs/initial_investigation.md, ReWire's taps in docs/rewire.md.
- ORB_CD_ISO is not set on this machine, so the tests that need the CD image skip (with their message); ORB_THIRDPARTY_* not set either.

Checks (one cargo job at a time):
- cargo check --workspace --all-targets clean; cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean (after cargo +nightly fmt on app.rs, menu.rs, portable.rs comment wraps).
- cargo test --release --workspace: exit 0, 105 test binaries, 1157 passed, 0 failed, 7 ignored (task-231's run: 1213 passed; the difference is the removed plugin tests: the three note_tests.rs, the gate/note-clock/taps/routing tests of frame/tests.rs, embed.rs's unit tests, tests/plugin_mode_cursor.rs, tests/note_mode.rs, portable.rs's two .vst3 tests). ORB_CD_ISO not set: the CD-image tests skip.
- orb-player's MIDI layer: crates/orb-player/src/midi.rs unchanged but its module doc; its unit tests and crates/orb-ui/tests/midi.rs ran in the workspace run and passed.
- cargo test -p orb-engine --test corpus_golden: 4 passed, 0 failed.
- cargo run --release -p orb-cli -- compare --manifest tests/refs/manifest.json --set diag,shipped: "153 of 153 equal", exit 0.
- cargo build --release: ok.
- a11y: git diff main -- crates/orb-ui/src/a11y and the eleven tests/a11y_*.rs is empty; they passed in the workspace run.
- assets/ui/preview PNGs not regenerated: `ui-art check` rewrites the panel PNGs too (they were already stale before this task), so they stay as committed.
- CHANGELOG: no line. Nothing a user can notice changed: Options > Output Routing, the mode selector, the note-mode lights and the rest showed only in the plugin's editor (App::embedded), never in the standalone app; the drum editor's help texts that lost their plugin sentence (drum_editor::Input::what) are not shown in the UI (only listed and tested).
- Kept, with reason: orb-player midi.rs and its tests (the user's choice). drums::Out and VoiceMut came with task-300 and went with it. Nothing else that only the plugin reached is left, so no Decide: task was needed.
- Main had not moved (d317061), so no rebase was needed; git diff main --stat lists only this task's files.
- Correction to the evidence above: the drum voices' adds are back in their pre-task-300 forms, each with its own operand order (e.g. SIMPLE's `st(s1 * g + out[j] as f64)` and `st(out[i] as f64 + g * s0)`), not one form for all.

Review round 1 (merge verdict with one fix): tests/garbage.rs a_pending_song_keeps_a_slot_for_its_play had lost its check that Play and Stop apply (the old test applied them with apply_now). Restored through the queue: Play sent after the edits, the UI side collects and the callback runs until it is applied (at most 8 calls); then !has_pending() and the new song plays; a Stop after it stops it. cargo test --release -p orb-player --test garbage: 2 passed; cargo +nightly fmt --all --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The code the app shared with upstream's removed plugin is gone. orb-ui: embed.rs, routing_window.rs (Options > Output Routing), tests/plugin_mode_cursor.rs, every is_plugin branch, the link's Port seam (Link holds its Player), session's file bytes, portable.rs's .vst3 bundle path and the app/default feature split. orb-ui-art: the plugin transport, mode selector, VST3 logo, Sync/MIDI LED and note-mode light parts (46 SVGs and their layout params). orb-player: the six host commands, the snapshot's sounding/gate/host-sync/MIDI fields and the Audio host API. orb-engine: the plugin commits' engine changes reverse-applied, so its source is the pre-plugin engine plus task-306 (checked line for line). Kept: orb-player's MIDI layer (midi.rs and tests), the user's choice. Docs: midi.md section 5 and rewire.md section 6 marked as upstream's, ui_spec.md section 21 (plugin editor) and the plugin rows gone, standard_midi_mapping.md, initial_investigation.md, original_bugs.md Q-6, AGENTS.md. No CHANGELOG line: nothing a user can notice (all of it showed only in the plugin's editor). Results: orb compare --set diag,shipped 153 of 153 equal; corpus_golden 4 passed; cargo test --release --workspace 1157 passed, 0 failed, 7 ignored (ORB_CD_ISO unset, CD tests skip); build --release, clippy -D warnings, nightly fmt --check clean; a11y code and tests unchanged. Branch worktree-agent-a49ccdcc13870a67b, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
