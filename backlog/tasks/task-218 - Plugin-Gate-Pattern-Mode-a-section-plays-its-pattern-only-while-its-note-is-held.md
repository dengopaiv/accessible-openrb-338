---
id: TASK-218
title: >-
  Plugin Gate Pattern Mode: a section plays its pattern only while its note is
  held
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T15:20:00Z'
updated_date: '2026-11-16T17:04:00Z'
labels:
  - player
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-217
references:
  - backlog task-217 (final summary)
  - backlog task-216 (final summary)
  - docs/plugin.md
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - docs/initial_investigation.md
  - crates/orb-plugin/src/core.rs
  - crates/orb-ui/src/embed.rs
priority: high
type: feature
ordinal: 225000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user decided this on 2026-11-16. The existing pattern mode (task-198, task-217) is a latch: notes 0..31 on a section's channel select a pattern that keeps playing until the next note. There is no way to stop a section from a MIDI clip, and a mid-song start plays whatever was selected last. A gate mode fixes both, the way Phoscyon's Host Note mode and ABL3's MIDI Trigger mode do: a pattern plays only while its note is held. In a DAW, the clip's note lengths then are the arrangement, and a gap in the clip is silence.

**Modes after this task (the Options menu radio items, plugin state, not a host parameter, as task-217 built):**
- "Latch Pattern Mode": the existing pattern mode, renamed. Its behavior is unchanged.
- "Gate Pattern Mode": new.
- "Note Mode": unchanged.
- Preview (the panel's Play) overrides all three, as task-217 built.
- The default mode for a new instance stays as task-217 set it.

**Gate Pattern Mode (the user's choices of 2026-11-16):**
1. **Note-off stops the section at once.** The pattern stops sounding at the note-off's frame; no bar-line wait. A note-on also starts at once, at its frame (the supervisor's reading of "stop at once" together with 3; the user can object at the Reaper check).
2. **It plays with the host stopped.** A held note plays its pattern at the host's tempo (the song's tempo if the host reports none), starting from step 1. This allows live triggering from a keyboard.
3. **It follows the host while the host plays.** A held note plays from the step at the host's position, like Latch mode, so a note on beat 2 starts mid-pattern in step with the song. It also follows host loops and jumps continuously.
4. **The last note pressed wins.** Several held notes on one channel: the last note-on selects the pattern. Releasing it falls back to the most recent note still held on that channel, or stops the section if none is held. A switch between held notes takes effect at once, at the host-derived step.
5. **Each section gates on its own channel** (1..4) independently.

**Standard Mapping (remote channel):** its pattern notes only select, in both pattern modes (the user chose two modes over a remote latch layer). In Gate mode a remote selection doesn't open the gate. The remote's other actions are unchanged.

**Engine constraint:** the engine, like the original, applies pattern changes and section on/off only at absolute bar lines (docs/initial_investigation.md, around line 268, [C]). Gate's at-once start, stop and switch must not change that path for songs or for Latch mode.
- Build it in the plugin or orb-player layer: for example, gate the section's output and set the pattern and its position directly when the gate opens, or add an explicit immediate path that only the plugin's gate uses.
- Any orb-engine change needs the full corpus run (`ORB_CD_ISO="/Users/pingw33n/Downloads/Rebirth 2.01 (2001 version).iso" cargo test --release -p orb-engine --test corpus_golden`), with zero differences.
- An abrupt stop may click. Note it, and propose a de-click ramp if one is audible, but don't add one without saying so.

**Docs:** docs/plugin.md (the modes) and docs/midi.md section 5 (gate semantics). Add a change note to task-185's final summary (`--append-final-summary`).

**No compatibility work for old plugin states** (the user's rule until the plugin is released).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The Options menu offers Latch Pattern Mode, Gate Pattern Mode and Note Mode; Latch is the former pattern mode unchanged (its tests still pass)
- [x] #2 Gate: a section sounds only while a note 0..31 on its channel is held; note-on starts and note-off stops at the event's frame; with the host playing the pattern plays from the host-position step and follows loops and jumps; with the host stopped it plays from step 1 at the host's tempo, or the song's if none (tests through the process path)
- [x] #3 Gate: with several notes held, the last note-on wins; releasing it falls back to the most recent still-held note or stops the section; sections on channels 1..4 gate independently (tests)
- [x] #4 Remote (Standard Mapping) pattern notes select but do not open the gate in Gate mode (test)
- [x] #5 No change to song rendering or Latch mode: the engine's bar-line path is untouched, or the full corpus run passes with zero differences if orb-engine changed
- [x] #6 docs/plugin.md and docs/midi.md describe Gate mode; task-185's final summary has a change note
- [x] #7 The user checks Gate mode in Reaper before merge (screenshots of the menu included)
- [x] #8 cargo test, clippy -D warnings and nightly fmt pass
- [x] #9 Gate never changes the section's On/off switch (no parameter change or host gesture); a section sounds only when its On/off is on and its gate is open; the panel's On button does not follow the gate (tests)
- [x] #10 Gate: a held note shows its pattern on the panel without changing the pattern parameter; after note-off the panel keeps the last gated pattern (test)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. instance.rs: Mode {Latch, Gate, Note} (state strings latch/gate/note; no old-state compatibility), Shared keeps the mode as an AtomicU8; File > Open sets Latch.
2. orb-ui embed: PluginMode {Latch, Gate, Note}, MODE_ITEMS: Latch Pattern Mode, Gate Pattern Mode, Note Mode; editor maps them.
3. orb-engine (not in the original, a path only the plugin's gate uses, default off): frame::Gate {on, open, pattern}; ChannelDevice::gate/set_gate on the 303s, 808, 909. Gate on and closed: the device does not step, its voices are silenced at the close (slot 6) and its bus is zeroed in Frame::block right after slot 14 (before the inserts), so it stops at once. Gate on and open: the bar-line latch and slot 2 take the gate's pattern instead of controller 1 (what plays is the held note's pattern; a panel or automation change only shows); opening or switching sets the pattern, pat_start and the enable latch at once and forces the current step to trigger (wrapped; the 303's pending gate-off first). Gate off: the original's code path unchanged. Engine::set_gate.
4. orb-player: Command::Gate {device, gate}; Snapshot::gate_closed [bool; 4]; orb-ui link hides the step light of a closed section.
5. core.rs: per-channel held-note stacks (fixed arrays, no allocation); gating = Gate mode and no preview; entering gating closes the four gates, leaving it turns them off and forgets held notes. Note-on 0..31: DeviceParam ctrl 1 (the panel shows it, the parameter does not change) + Gate open; note-off: fall back to the most recent held note or close. Host stopped: the first open gate locates to 0 and plays (step 1), the song free-runs at the host's tempo (the song's if none) while any gate is open, stops when none is; host playing: following as Latch. Remote pattern notes only select (controller 1), as now.
6. Tests: orb-plugin tests/gate_mode.rs through process_with (start/stop at the frame, host position, loops, stopped host step 1 and tempo, last-note-wins, channels independent, remote, On/off untouched, panel pattern), engine unit tests for the gate, no_alloc with gate events; Latch tests renamed only.
7. Full corpus run (orb-engine changed): zero differences required.
8. Docs: plugin.md (modes, state, section 8/10, new Gate section), midi.md 5 (gate semantics), task-185 final summary change note; measure the click at a gate stop and propose a de-click ramp as a Decide task.
9. cargo test, clippy, nightly fmt, bundle + validate-plugin.sh, menu screenshots with editor_preview.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Clarification (supervisor, 2026-11-16, per task-185's rule that notes never change parameters): the gate never changes the section's On/off switch (the host parameter, the panel's On button, the remote's On/Off note). A section sounds in Gate mode only when its On/off switch is on AND its gate is open; the On/off switch stays the user's mute, independent of the clip. The panel's On button does not follow the gate; the gate state shows only through the playing pattern's step lights. No gate gesture or parameter change reaches the host.

The panel display in Gate mode (the user, 2026-11-16, option A):
- A held gate note shows its pattern on the panel without changing the host's pattern parameter (as in Latch mode).
- After the note-off, the panel keeps showing the last gated pattern; it does not revert to the parameter's pattern.
- In Gate mode what plays is always the held note's pattern. A panel click or automation of the pattern parameter sets the parameter and the shown pattern as now, but sounds only through a gate note.
- The preview (task-217) stays the way to edit without MIDI interfering.

Branch: worktree-agent-aa89c0844b3b748ed

Design (2026-11-16). The gate is an explicit engine path that only the plugin uses (not in the original), default off, so songs and Latch keep the original's code path:
- orb-engine frame::Gate {on, open, pattern}; ChannelDevice::gate/set_gate on Tb303, Tr808, Tr909 (field pattern_gate). Closed: slot 1 returns at once (no step), slot 6 silences the voices when it closes, Frame::block zeroes the channel bus right after slot 14 and before the inserts (the cut is at the block). Open: the bar-line latch in slot 1 and slot 2 take Gate::latch(controller 1) = the gate's pattern, so a panel click, automation or a remote selection only changes what the panel shows while a gate note is held (the user's option A: what plays is always the held note's pattern). Opening or switching runs slot 2 with the gate's pattern at the current tick (pattern, patStart, enable latch), ends a sounding 303 note, and arms the step: Engine::set_gate then runs slot 1 at once with (tick_pos, shuffled, tempo) while the song plays.
- Failed first version: arming with wrapped=true at every open and stepping at tick_pos. A clip note on the beat arrives when tick_pos is the previous step's last tick (e.g. 287 before 288), so it played the old step and the 303's gate (v+24) swallowed the next step's start (step test: LAST_STEP 5 at tick 292). Second failure: without the immediate step, the start waited for the next tick change (first sound at frame 1091 for an event at 1000, two blocks late at 120 BPM: 0.557 ticks per block). Final rule (frame::gate_arm): the step at the position triggers at once when the position is in its first half (v % 48 < 24, while a 303 note of it would still sound), else the device waits for the next step, which triggers on time (LAST_STEP set to the current step, wrapped false, the 909's flam disarmed).
- orb-player: Command::Gate {device, gate}; Snapshot::gate_closed [bool; 4]; orb-ui link.rs hides the step light of a closed section (a closed section's LAST_STEP is frozen).
- orb-plugin core.rs: Mode {Latch, Gate, Note} in Shared (AtomicU8); per-channel held-note stacks ([u8; 32] + len, no allocation); gating = Gate mode outside the preview; entering or leaving gating forgets the held notes. Every block (and before each section note) gates() compares each engine gate (Audio::newest) with the wanted one and resends a differing one: this covers a newly loaded song (its gates are off) and a Gate command the full garbage queue refused (apply_now refuses below GARBAGE_PER_COMMAND free slots; seen in the test fixture: 128 pattern edits filled it and the first four Gate commands were dropped). Note-on 0..31: press, DeviceParam ctrl 1 (the panel shows it; the parameter does not change) then Gate open. Note-off (or velocity 0): if it was the top, fall back to the newest still held or close. With the song stopped (host stopped) the first open gate sends Locate(0)+Play (step 1); sync() keeps the song free-running at the host's tempo (the song's if none) while any gate is open and stops it when none is.
- Smaller choices (notes, not decisions): File > Open turns Note Mode into Latch and keeps a pattern mode (Gate stays Gate). Held notes are a set: a note pressed twice and released once is released. The host stopping with notes held: the song plays on free-running from where it is (no jump to step 1). A section's On/off switched on while its gate is open sounds from the next bar line (the original's enable latch); opening a gate takes the switch at once (slot 2 reads it). Preview entry forgets the held notes (the preview ignores note-offs); after it, a held note sounds only when pressed again. Placement of state: no Saved field changes besides the mode strings latch/gate/note.

Evidence:
- cargo test -p orb-plugin --test gate_mode: 11 tests pass (start/stop at the block for buffers 64, 1000, 8192: first sound at host frame 1024..1087 for a note-on at 1000, last sound at 1023 for a note-off at 1000; host position/loop/seek; host stopped: tick 0 and step 1 after the note, tempo 97250 or the song's, tick after 1 s within 2 of 44100*tempo*TEMPO_STEP/64, stop with the last gate; last-note-wins sequence; channels independent and closed sections do not step; remote note 19 on ch 16 selects pattern 3 without opening, and does not change the held pattern past 3 bars; On/off and parameters untouched, off+open silent, on again sounds; panel shows the held pattern and keeps it after note-off, parameter 21 shows but 13 plays; mode/preview/File > New; the second-half rule at 60 BPM).
- Engine unit tests: frame::tests::the_gate_helpers, a_closed_gate_silences_its_section_and_does_not_step.
- no_alloc: a 40000-frame Gate session (host stopped 60000..70000, then playing and jumping) allocates nothing and takes no lock.
- Full corpus (orb-engine changed): ORB_CD_ISO=... cargo test --release -p orb-engine --test corpus_golden: 4 passed (diag 145, app 11, shipped 8 equal). orb compare --manifest tests/refs/manifest.json --mod-dir <recrypted CD mods>: 175 of 175 equal (the two pcf wave127 warnings as before).
- The click: the stop is a hard cut at a block boundary. Over 40 note-off positions at the default levels the largest step from the last sample to 0 is 0.036 full scale for a 303 (its peak 0.125), 0.047 for the 808 (peak 0.077), 0.037 for the 909 (peak 0.069): 30..60 % of the section's peak, about -27 dBFS, an audible click on a sustained 303 note (test the_stop_is_a_hard_cut prints them with --nocapture). No ramp added.

Created task-222 (Decide: a de-click ramp for the Gate stop; @pingw33n, depends on task-218; recommendation: a one-block linear ramp). Not added here.

task-185's final summary has the change note (--append-final-summary). Docs: docs/plugin.md (intro, 4 mode bullet, 5 state mode strings, 8 terminology note and host-stopped/notes paragraphs, 10 Options menu and File > Open, new section 11 Gate Pattern Mode with the engine path and addresses), docs/midi.md (5 intro, 5.2 renamed Latch, new 5.2.1 Gate).

Checks: cargo test 823 passed, 0 failed, 5 ignored; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. Bundles: cargo xtask bundle orb-plugin --release -> target/bundled/OpenRB-338.clap and OpenRB-338.vst3; scripts/validate-plugin.sh: validators all passed (pluginval SUCCESS). Screenshots (editor_preview, full_mix.rbs, --ppp 2, --host-playing): /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/worktree-agent-aa89c0844b3b748ed/options_menu_gate.png (Options: Latch Pattern Mode, Gate Pattern Mode selected, Note Mode), /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/worktree-agent-aa89c0844b3b748ed/options_menu_latch.png (Latch selected), /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/worktree-agent-aa89c0844b3b748ed/gate_host_playing.png (Gate, host playing, no gate note: Sync lit, no step lights).

Rebased on main 3d5227b (task-219): clean. After the rebase: cargo test 826 passed, 0 failed; clippy and nightly fmt clean; bundles rebuilt and validated (all passed); git diff main --stat lists only this task's files (crates orb-engine, orb-player, orb-plugin, orb-ui; docs/plugin.md, docs/midi.md; backlog tasks 185, 218, 222).

Review round 1 (2026-11-16), supervisor's defects and the user's decisions:
- Fixed: the stop test no longer pins the hard cut (now the_stop_fades_out_over_one_block, below); LAST_STEP mask 0xFF in all three devices' set_gate (the value is stored as a byte either way); commit subject names engine; follow-up bug task-225 created (host-event commands refused by a full garbage queue are lost; labels player, engine; m-7; 5 ACs); the draft summary's counts and pending items rewritten.
- User decision 1, gate start: snap near the boundary, replacing the second-half rule. A note-on plays the step at the position at once wherever in the step it comes, except within 5 ms (GATE_SNAP_MS) before the next step boundary, at most 12 ticks (GATE_SNAP_MAX_TICKS, a quarter step), which takes the next step on time. Why 5 ms: an event is applied between blocks, so an on-grid clip note finds the engine up to one block (1.45 ms) before its step; 5 ms covers that with room for host jitter (3.4 blocks) and stays under ~10 ms, where an early onset starts to be heard as a separate event. The cap keeps it a boundary snap at fast tempos (5 ms = 16 ticks at 999 BPM, a step 15 ms there); in time units it is tempo independent below 750 BPM. Computed in Engine::set_gate: lead = (pos - trunc(pos)) + min(5 ms * 44100 * tempo * TEMPO_STEP / 64, 12) ticks; frame::gate_arm waits when 48 - v % 48 <= lead. A 303 step entered late now gets its gate-off capped at the step's last tick so the next step is not swallowed. Each device runs slot 1 itself after arming (GateClock carries tick, shuffled, tempo, playing, lead). Tests: engine the_snap_window_is_5_ms_and_at_most_a_quarter_step (120 BPM: 1.5 ticks before waits, 2.5 plays; 999 BPM: 11.5 waits, 13.5 plays); plugin an_on_beat_clip_note_lands_on_its_step (16 block offsets: step 7 lit, the 303 still on step 7 ten ticks on), a_mid_step_note_starts_at_once (10 % and 80 % into a step: sound within the next block, the next step on time).
- User decision 2, a second gate with the host stopped joins in step (kept); Locate(0) only when nothing plays. Test with_the_host_stopped_a_second_gate_joins_in_step; documented.
- User decision 3 (task-222): a one-block linear fade on the Gate stop. Frame::gate_fade[c] is set by Engine::set_gate on a close (cleared by any other gate change); the next Frame::block multiplies the bus by (63 - i)/64 (f64, stored f32) after slot 14, then runs slot 6; later blocks zero it. The devices' set_gate no longer silence on a close. With the host stopped, sync keeps the song playing while a fade is pending, so the Stop's slot 6 does not cut it. Measured over 40 stop positions: 0.0016 (303, peak 0.125), 0.0007 (808), 0.0008 (909), below the 0.002 bound; silent from the second block after the note-off.
- AC #2's 'note-off stops at the event's frame' now means: the fade runs in the block after the event and the section is silent from the block after that; 'note-on starts at its frame' holds except for the 5 ms snap before a boundary.

Review round 2: merge verdict after the Reaper check. Nits fixed: task-222 has a correction note naming the_stop_fades_out_over_one_block; task-222's final summary and docs/plugin.md section 11 state the fade bound's level dependence (step about peak/64: below 0.002 only for a peak below about 0.128, the 303's default 0.125 with little margin; a full-scale section about 0.016). Docs and backlog only; bundles not rebuilt.

AC #7: the user checked the round-2 build (snap, fade, join) in Reaper on 2026-11-16: "gate looks fine" (relayed by the supervisor). Menu screenshots: scratchpad options_menu_gate.png, options_menu_latch.png (round 1).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: the plugin's Options menu has Latch Pattern Mode (task-217's pattern mode, unchanged; its tests pass renamed), Gate Pattern Mode (new) and Note Mode; state strings latch/gate/note. In Gate a section sounds only while a note 0..31 on its channel is held. The note-on starts it at the block after the event, from the step at the position, except within 5 ms (at most 12 ticks) before a step boundary, where it takes that step on time (the user's snap decision, 2026-11-16; on-grid clip notes land on their steps). The note-off fades it out linearly over the next 64-frame block, silent after (task-222, decided 2026-11-16, done in this commit). With the host playing it follows the host position, loops and seeks; with the host stopped the first note plays from step 1, free-running at the host's (or the song's) tempo, and a second section joins in step (the user's decision). Last note wins with fallback; channels independent; the Standard Mapping's pattern notes only select; On/off and parameters never touched; the panel shows the held pattern and keeps the last one; a panel or automation pattern change only shows while a note plays its own pattern. File > Open turns Note into Latch and keeps a pattern mode.

How: a plugin-only gate path in orb-engine (frame::Gate, GateClock, gate_arm, Frame::gate_fade; off by default, so songs, Latch and Note keep the original's path): closed = no step, one-block fade then slot 6 and silence; open = the bar latch and slot 2 take the gate's pattern, opening arms the step and runs slot 1 at once. orb-player Command::Gate and Snapshot::gate_closed (orb-ui hides a closed section's step light). The plugin resends any engine gate that differs from its held notes before every block. No allocation or lock in process.

Numbers: full corpus 175 of 175 equal (corpus_golden diag 145, app 11, shipped 8; orb compare with the recrypted CD mods). Step to silence at a stop: 0.0016 (303), 0.0007 (808), 0.0008 (909) of full scale at the default levels, from 0.036/0.047/0.037 with the first build's hard cut; about peak/64, so a full-scale section gives about 0.016.

Tests: cargo test 829 passed, 0 failed (gate_mode 13 tests, 3 engine unit tests, no_alloc with a Gate session); clippy -D warnings and nightly fmt clean; bundles built and validated (validate-plugin.sh all passed).

User check: the user checked the round-2 build (snap, fade, join) in Reaper on 2026-11-16: "gate looks fine".

Review: round 1 send back (fixed, plus the user's three decisions), round 2 merge.

Follow-up: task-225 (bug: other host-event commands refused by a full garbage queue are lost).

Branch worktree-agent-aa89c0844b3b748ed, one commit (player,engine,ui,docs: plugin Gate Pattern Mode, Latch rename).
<!-- SECTION:FINAL_SUMMARY:END -->
