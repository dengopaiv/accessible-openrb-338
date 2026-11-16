---
id: TASK-217
title: >-
  Plugin pattern preview, and a pattern/note mode setting that follows the host
  transport
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T13:33:00Z'
updated_date: '2026-11-16T15:48:00Z'
labels:
  - player
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-216
  - TASK-201
references:
  - backlog task-216 (final summary)
  - backlog task-185 (final summary)
  - docs/plugin.md
  - docs/midi.md
  - crates/orb-plugin/src/params.rs
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/editor.rs
  - crates/orb-ui/src/embed.rs
  - crates/orb-plugin/tests/param_ids.txt
priority: high
type: feature
ordinal: 224000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user decided this on 2026-11-16 (task-216's final summary is the spec). Pattern mode as built, with `g1` as a host parameter and orb free-running while the host is stopped, gave no stable way to edit patterns. Host MIDI and automation could change or stop what the user was editing, and nothing could stop a pattern from MIDI.

**What to build (crates/orb-plugin, orb-ui's plugin presentation in embed.rs):**
1. **Pattern preview on the panel's Play/Stop.**
   - Play enters preview and Stop leaves it. Preview behaves like the standalone app's pattern mode: orb plays its patterns free-running (the step does not follow the host position) at the host's reported tempo, also while the host is stopped, and at the song's tempo only when the host reports none (the user's Reaper check, 2026-11-16); the tempo display shows that tempo.
   - Play and Stop show only whether preview runs: Play is lit only in preview; starting or stopping the host does not change them, and pattern mode following the host does not light Play (the user's Reaper check, 2026-11-16).
   - While preview runs, all host input is ignored: notes, the Standard Mapping, and parameter changes or automation. Ignoring means the preview's sound and panel don't change; the host's parameter values still update in the plugin's model, so leaving preview returns to the latest host values.
2. **The preview set.**
   - Knob and switch changes and pattern selections made during preview go into a preview set in the plugin's state, not to the host parameters. Send no begin/set/end gestures to the host for them.
   - Leaving preview returns the panel and the engine to the host's parameter values. The next entry restores the preview set. The first entry starts from the current values.
   - Pattern step edits (pattern data) keep working as now and persist whether in preview or not.
   - The preview set is saved and restored with the state.
3. **A pattern/note mode setting in the plugin's state,** not a host parameter, saved with the project. It is a panel or Options-menu control; pick the clearer one and show a screenshot. (Built: Options-menu radio items labeled just "Pattern Mode" and "Note Mode", without descriptions; the user's Reaper check, 2026-11-16.)
   - Pattern mode plays only while the host plays, with the step following the host position as now (task-198, task-206 rule).
   - In pattern mode, when the host is stopped there is no sound, and pattern-select notes (0..31 on channels 1..4) only change the panel's selected patterns.
   - Note mode: notes play the devices whatever the host's transport does, exactly as now (task-200, task-210); the no-sound-while-stopped rule is pattern mode's only (the user, 2026-11-16).
   - Preview overrides both modes while it runs.
4. **Remove `g1` (the mode) from the host parameters.**
   - Update crates/orb-plugin/tests/param_ids.txt and the parameter count. Shuffle `g20` and the others keep their ids. No release has shipped yet.
   - No compatibility with plugin states of earlier builds (the user, 2026-11-16): until the plugin is released on GitHub Releases the state format changes freely, with no tests, fallbacks or migrations for old states.
5. **Displays.** Sync and Bar show the host while orb follows it and are dark in preview, where the panel's own position shows if the standalone has one in this layout. The MIDI LED still blinks on input during preview, but the input is ignored.
6. **Docs.** Update docs/plugin.md (parameters section 3, the modes and host sync, the state) and docs/midi.md section 5 (pattern selection when the host is stopped; preview ignores input). Add a change note to task-185's final summary (`--append-final-summary`) pointing to task-216.

**Context:** the plugin's mode and transport live in crates/orb-plugin (params.rs, instance.rs, core.rs; docs/plugin.md). The editor's Play/Stop goes through PluginPort in editor.rs. Mind task-214, an open bug: a parameter change before the first engine block after an activation is lost; it touches the same activation path. Engine code is unchanged, so no corpus run is needed unless engine code changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Panel Play enters preview: the patterns play free-running (step not following the host position) at the host's reported tempo, the song's only when the host reports none, whether the host plays or not; host notes, Standard Mapping messages and parameter changes during preview change neither the sound nor the panel (tests through the plugin's process path)
- [x] #2 Changes made in preview are kept in a preview set: leaving preview returns the panel and engine to the host's current parameter values, the next preview restores the set, no host parameter gesture is sent for them, and the set survives a state save/restore (tests)
- [x] #3 A pattern/note mode setting in the plugin state, not a host parameter: in pattern mode orb sounds only while the host plays (step from the host position); with the host stopped there is no sound and pattern-select notes only change the panel's selected patterns; in note mode notes play the devices whatever the host's transport does, as now (tests)
- [x] #4 g1 is no longer a host parameter (param_ids.txt updated)
- [x] #5 docs/plugin.md and docs/midi.md describe preview, the preview set and the mode setting; task-185's final summary has a change note
- [x] #6 Screenshots of the panel in preview and with the mode control; the user checks it in Reaper before merge
- [x] #7 cargo test, clippy -D warnings and nightly fmt pass
- [x] #8 Play and Stop show only whether preview runs: the host's start and stop do not change them, and pattern mode following the host does not light Play (test)
- [x] #9 The Options menu items read just "Pattern Mode" and "Note Mode"
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. instance.rs: Mode {Pattern, Note} in the Doc (JSON key mode, default note for a new instance), the preview set (JSON key preview, by parameter id), Shared gains the mode, the preview flag and the preview values as atomics (process side reads them per block, no lock). OrbParams: mode/set_mode, previewing/set_preview (first entry copies the host values), set_preview_value, panel_values; load_song resets the preview set (to the loaded values while preview runs).
2. params.rs: remove Target::Mode and g1 (COUNT 119), param_ids.txt updated. No migration of old states (the user, 2026-11-16).
3. core.rs: the mode from Shared; per block either preview (apply the preview set's changes, play free-running at the song's tempo, no host sync, notes/mapping/parameters ignored but host values tracked and the MIDI count kept) or the normal path: pattern mode plays only while the host plays (clean start/locate as now), stops when it stops; note mode stopped, notes play the devices whatever the transport. Leaving preview applies the host values (all, through the holds check). Pattern-select notes while the song is stopped only set the pattern (no relocate).
4. editor.rs: Play/Stop enter/leave preview; in preview a control goes to the preview set (no ParamSetter gestures); view() uses the panel values; File > Open sets pattern mode. orb-ui embed.rs/menu.rs: Embed::mode/set_mode, Options menu radio items for the plugin (standalone unchanged).
5. Tests: preview through process path (AC1), preview set incl. no gestures and state round trip (AC2), mode behaviour (AC3), g1 gone (AC4); update existing tests from g1 to the mode setting.
6. editor_preview example: --preview, --mode, --click for screenshots (preview, Options menu).
7. Docs: plugin.md 3, 4, 5, 6, 8, 10; midi.md 5; task-185 change note. Run cargo test, clippy, nightly fmt; bundle + validate-plugin.sh.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ac54b1aea52298cce (worktree .claude/worktrees/agent-ac54b1aea52298cce), from main 915b800.

Clarifications from the user (2026-11-16, through the supervisor), applied to description items 3 and 4 and the ACs:
1. No sound while the host is stopped applies to pattern mode only; in note mode host notes play the devices whatever the host's transport does, exactly as before.
2. No backward compatibility for now: a state saved with g1 need not map it to the mode; old states load with the default mode (note) and must not crash or lose the rest. No old-state mapping test. The preview set needs no old-state handling beyond loading. (A filter_state migration of g1 was started and removed.)

Correction from the user (2026-11-16, through the supervisor): no compatibility work with old plugin states at all, not even loading without crashing, until the plugin is released on GitHub Releases. The state format changes freely; no tests, fallbacks or migrations for states of earlier builds. Applied: the description's old-state sentences are dropped; AC #4 reads 'g1 is no longer a host parameter (param_ids.txt updated)'; the state's new keys (mode, preview) have no serde defaults.

Implementation (2026-11-16):
- params.rs: Target::Mode and g1 removed, COUNT 120 -> 119; param_ids.txt loses its first line; g20 and the d<dev>c<n> ids unchanged (the_ids_do_not_change passes against the edited list).
- instance.rs: Mode {Pattern, Note} (serde lowercase), Doc.mode and Doc.preview (BTreeMap id -> value, None before the first preview); JSON keys mode (required) and preview (null or object), no serde defaults (no old-state compatibility). Shared gains pattern_mode (AtomicBool), preview (AtomicBool, Release/Acquire) and preview_values ([AtomicI32; 119]). OrbParams: mode/set_mode, previewing/set_preview, set_preview_value (refused outside the preview, clamped), preview_values, panel_values. load_song during the preview makes the loaded values the set; outside it forgets the set. deserialize_fields ends a running preview and writes the restored set to the atomics.
- core.rs: Engine.playing (the g1 level) replaced by pattern_mode, preview, host[119] (host values, moved from Core.last) and shown[119] (preview values applied). Before each block modes(): reads the mode; on preview entry applies the whole set (apply_all, through the old holds check), on exit the host's latest values; during the preview applies changed set values. sync(): preview -> song tempo, no host sync, ensure_playing; pattern mode + host playing -> follow as before (clean start also after a pending load); pattern mode + host playing without position -> free-run; otherwise ensure_stopped (switches to a pending song first, then Stop). message(): MIDI count always; ignored when previewing (engine flag or shared flag); note mode reads the shared mode at the message (so a note right after choosing note mode stops the song first); pattern selects send Relocate only while the song plays. apply(): host values always recorded, sent only outside the preview. No allocation or lock added (no_alloc.rs session extended with mode switches, preview entry/edits/exit).
- Choices made (small, recorded here): default mode note (a new instance makes no sound of its own, as g1's Stop default); File > Open sets pattern mode (task-185 rule kept); preview entry keeps the song playing where it is (a stopped song starts from its bar start), it does not restart; the preview is not saved and a state restore ends it; it survives a re-activation; panel Stop outside the preview does nothing (pattern mode with the host playing keeps following); the mode control is an Options-menu radio pair (Pattern Mode (patterns play with the host) / Note Mode (MIDI notes play the instruments)), chosen over a panel control because the panel has no free switch and the hidden Pattern/Song switch's art is labeled Pattern/Song.
- editor.rs: Play/Stop -> set_preview(true/false); in the preview a control -> set_preview_value (no ParamSetter, no gesture); view() uses panel_values; Embed::mode/set_mode. orb-ui: Embed::mode/set_mode (default None / no-op), PluginMode, MODE_ITEMS, Cmd::SetPluginMode; the radio items are drawn only in the plugin branch of Options (the standalone's menu code path is unchanged).
- Tests: new tests/preview.rs (4 tests: free-running at the song tempo for both modes and host stopped/playing, Stop returns to the mode; host notes on 4 channels + Standard Mapping CC + its Stop note + parameter automation + transport jumps during the preview give bit-identical output and identical snapshot live/mixer/patterns/tick vs an undisturbed preview, midi_in 400; preview set vs host values, re-entry restore, clamp, save/restore of set and mode, restore ends preview; load during preview). sync.rs: free-run assertions turned into stopped/silent, new with_the_host_stopped_notes_only_select. note_mode.rs: new note_mode_plays_notes_whatever_the_host_transport_does. timing.rs: pattern_mode_stops_and_starts_with_the_host replaces the g1 test; CHANGES lost its two g1 entries. editor.rs: play_and_stop_enter_and_leave_the_preview, preview_edits_go_to_the_preview_set_without_host_gestures, the_options_menu_sets_the_mode, LED test extended with the preview. params.rs: the_mode_is_no_host_parameter. exports.rs: the CLAP host now sends a clap_event_transport with CLAP_TRANSPORT_IS_PLAYING (no tempo, no position), since pattern mode is silent with the host stopped. common::host plays the host transport PLAYING (no tempo/position: pattern mode free-runs at the song tempo, the old behavior of these tests).
- Results: cargo test (workspace, ORB_CD_ISO set) 809 passed, 0 failed, 5 ignored; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. Engine and orb-player code unchanged: no corpus run needed.
- Bundles: cargo xtask bundle orb-plugin --release -> target/bundled/OpenRB-338.clap and .vst3 in the worktree; scripts/validate-plugin.sh: clap-validator 44 tests, 36 passed, 0 failed, 8 skipped; pluginval SUCCESS; 'validators: all passed'.
- Screenshots (editor_preview, full_mix.rbs, --ppp 2): /private/tmp/claude-501/-Users-pingw33n-dev-openrb-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/agent217/pattern_host_playing.png (pattern mode, host playing: Sync lit, Bar 001, tempo 120 the host's), /private/tmp/claude-501/-Users-pingw33n-dev-openrb-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/agent217/preview_host_playing.png (--preview with the host playing: Sync and Bar dark, tempo 130 the song's, Play lit, MIDI lit), /private/tmp/claude-501/-Users-pingw33n-dev-openrb-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/agent217/options_menu.png (--click 104,11: Options with the two mode radio items, Pattern Mode selected). Example gained --mode, --preview, --click X,Y (raw_input_hook).

Review round 1 (2026-11-16, reviewer verdict on 17cce05: merge, pending the user's Reaper check). One cleanup: params.rs build() filled the table with the shuffle spec and then assigned s[0] again. Fixed: the fill is the shuffle at index 0 and the counter starts at 1 (one assignment; the counter is still needed for the entries after it). Reran: cargo +nightly fmt --all --check clean, cargo clippy --all-targets -D warnings clean, cargo test -p orb-plugin 64 passed, 0 failed (param_ids.txt unchanged, so the table is the same). target/bundled not rebuilt (the user is testing those bundles); the change does not alter the parameter table.

Round 2, the user's Reaper check (2026-11-16, through the supervisor): three changes, applied to the description (item 1, item 3) and the ACs (#1 reworded, #8 and #9 added).
1. The preview plays at the host's reported tempo, also while the host is stopped; the song's tempo only when the host reports none. Still free-running (no position following). core.rs sync(): the host-tempo filter no longer excludes the preview. preview.rs: the_preview_plays_free_running_at_the_hosts_tempo checks 97.25 (host stopped), 151 (host playing at beat 17.5) and a host without tempo (the song's), the tick being the free-run's at that tempo; the_preview_ignores_host_input now keeps the host's tempo constant (tempo changes are no longer ignored, by design).
2. Play/Stop show only the preview: orb-ui Embed::previewing() (default None), set over panel.transport.playing after each link.sync in App::show (plugin only; the standalone path is unchanged). New editor test the_play_light_is_the_preview_not_the_hosts_transport (pattern mode following a playing host: engine plays, Play dark; Play click lights it and the host's stop keeps it; Stop with the host playing: dark).
3. Options items read just 'Pattern Mode' and 'Note Mode' (embed::MODE_ITEMS).
File > Open during the preview left as it is (writes the loaded values to the host parameters), as instructed.
Results: cargo test 810 passed, 0 failed, 5 ignored; clippy -D warnings and nightly fmt clean. Bundles rebuilt with CARGO_TARGET_DIR=target/r2 into target/r2/bundled (target/bundled untouched, 16:00); scripts/validate-plugin.sh target/r2/bundled: clap-validator 44 run, 36 passed, 0 failed, 8 skipped; pluginval SUCCESS; validators: all passed.
Screenshots: /private/tmp/claude-501/-Users-pingw33n-dev-openrb-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/agent217/r2_pattern_host_playing.png (pattern mode, host playing: Sync lit, Bar 001, Play dark), /private/tmp/claude-501/-Users-pingw33n-dev-openrb-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/agent217/r2_preview_host_playing.png (preview, host playing: Play lit, tempo 120 = the host's, Sync and Bar dark), /private/tmp/claude-501/-Users-pingw33n-dev-openrb-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/agent217/r2_preview_host_stopped.png (preview, host stopped), /private/tmp/claude-501/-Users-pingw33n-dev-openrb-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/agent217/r2_options_menu.png (Options: Pattern Mode / Note Mode).

Supervisor, 2026-11-16: the user's Reaper check of the round 2 bundles (target/r2/bundled, b2715c6, installed in ~/Library/Audio/Plug-Ins) passed: preview at the host's tempo, Play/Stop independent of the host's transport, the menu labels, and the round 1 points (input ignored in preview, the preview set, pattern mode silent with the host stopped, note mode either way, state across a project reopen). Reviewer verdict: merge (2 rounds). Follow-ups: task-218 (Gate Pattern Mode; renames Pattern Mode to Latch Pattern Mode), task-219 (logo animation only for a new instance).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Done after the user's Reaper checks of 2026-11-16 (macOS, Reaper 7.86, VST3 and CLAP; round 2 bundles from b2715c6).

Built the user's design of 2026-11-16 (task-216) with the Reaper-check changes of the same day: the panel's Play/Stop enter and leave a pattern preview (free-running at the host's reported tempo, the song's only when the host reports none; all host input ignored; edits kept in a preview set in the plugin state and never sent to the host), and Play/Stop show only whether the preview runs (the host's transport never lights Play). A pattern/note mode setting in the plugin state (Options menu: Pattern Mode / Note Mode; default note mode; File > Open sets pattern mode) replaces the host parameter g1 (removed, 119 parameters). Pattern mode plays only while the host plays (step from the host position) and is silent with the host stopped, where pattern-select notes only change the selection; note mode plays the devices from notes whatever the transport does. No compatibility with plugin states of earlier builds (the user's rule).

Branch worktree-agent-ac54b1aea52298cce, one commit on main 915b800. Tests: cargo test 810 passed / 0 failed / 5 ignored; clippy -D warnings and nightly fmt clean; validators all passed on target/r2/bundled. Engine code unchanged (no corpus run). Docs: plugin.md 3, 4, 4.2, 5, 6, 7, 8.2, 9, 10; midi.md 5.2, 5.3; task-185 change note.

For the user in Reaper: preview tempo follows Reaper's tempo (also stopped); starting/stopping Reaper never changes Play/Stop; the Options items read Pattern Mode / Note Mode.
<!-- SECTION:FINAL_SUMMARY:END -->
