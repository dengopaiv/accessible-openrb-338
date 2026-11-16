---
id: TASK-198
title: >-
  Plugin: host sync and pattern mode (notes select patterns, step from the host
  position)
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T09:26:00Z'
labels:
  - player
milestone: m-7
dependencies:
  - TASK-185
  - TASK-197
references:
  - backlog task-185 (final summary)
  - docs/ui_spec.md
  - docs/song_load.md
  - docs/initial_investigation.md
priority: medium
type: feature
ordinal: 211000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The plugin (task-197) gets its pattern mode and host sync, as task-185's final summary decides (read it first).

- **Mode switch.** orb's Play/Stop is the mode, an automatable host parameter: playing = pattern mode, stopped = note mode (note mode is its own task). Song mode does not exist in the plugin; File > Open always puts the instance in pattern mode.
- **Pattern mode plays whether or not the host plays.**
  - Tempo: the host's (hosts report it while stopped); the loaded song's when the host reports none.
  - While the host plays, the step comes from the host position: bar 1 beat 1 = step 1. Host start, seek and loop jumps realign it, and an offline bounce is deterministic. Shuffle stays orb's (a parameter).
  - While the host is stopped, orb free-runs from where it is.
- **Pattern selection from notes.** Notes on the device channels 2..5 (303 #1, 303 #2, 808, 909): note n in 0..31 selects pattern n+1 of that section (banks A-D x 8). Latched: it changes at once, at the step the host position gives, and stays until the next note on that channel; note-off is ignored. A section with no note yet plays the plugin state's (the panel's) pattern; a panel click and a host note both change it, the last write wins.
- The Standard Mapping (task-182, docs/standard_midi_mapping.md) keeps working on its own channel (default 1), as in the standalone app; a remote channel equal to a device channel is refused or warned.
- The snapshot gives the editor what it needs: whether orb follows the host position (for the Sync LED: lit in pattern mode while orb and the host both play) and incoming MIDI (MIDI LED).

Where the original's sequencer has no "jump to step" (it advances in ticks per 64-frame block), find the way to place it at a host position (docs/initial_investigation.md, the sequencer and the transport's locate), and record the method in docs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Tests: with a simulated host transport, the step follows the host position after start, seek and loop jump; a host tempo change changes orb's tempo; with the host stopped orb keeps playing
- [x] #2 Tests: notes 0..31 on channels 2..5 select patterns at once and latch; note-off ignored; a panel change after a note wins until the next note
- [x] #3 Two offline renders of the same host timeline are identical
- [x] #4 The Standard Mapping still works on its channel; a device channel as remote channel is refused or warned
- [x] #5 The snapshot exposes the Sync LED state (pattern mode, orb and host playing) and a MIDI activity flag
- [x] #6 How orb places the sequencer at a host position is in docs with the engine addresses it relies on
- [x] #7 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. RE (done, notes): how the original places its sequencer at an external position. TopHatDriveAudioProc (ReWire's precursor) -> sub_1001C0F4: host tempo into the song's tempo, host play/stop -> sub_1001F7C0, a host reposition flag -> sub_1001C3C4 -> locate (sub_1001F980/F984), else sub_1001C2F4: pos = (double)hostTick each call (sub_1001F8B0's fild/fstp), without relocate. MIDI SPP (sub_1001C5C4) also locates.
2. orb-engine: Sequencer::set_pos / Engine::set_position (sub_1001F8B0, the position alone) and a one-device relocate (slot 2 on one device). No change to existing paths; corpus run anyway.
3. orb-player: Command::Position(f64) and Command::Relocate(Device); Snapshot gains host_sync and midi_in (set by the Audio's owner); Audio::last_snapshot for the mapping's state.
4. orb-plugin core: Core::process_with(l, r, values, &HostTransport, &[HostEvent]); a ring of per-call transport records; per block (before render): tempo level (host's, else the song's), follow the host position while it plays (host start: stop+locate+play; jump >= 1 tick: locate; else set the position), free-run while it is stopped. Events split the call at their frames (applied after their frame, as params). Notes 0..31 on channels 2..5 in Play: pattern select (DeviceParam ctrl 1 + relocate that device: at once); note-off ignored. The Standard Mapping on the remote channel (default 1): device/mixer/shuffle actions applied, transport actions ignored (the mode is the host's parameter). Sync LED and MIDI counter into the snapshot.
5. instance: remote channel setting in the state (refused on 2..5), an AtomicU8 shared with the core; OrbParams::snapshot(); select_pattern for the editor (a click on the param's value). lib.rs: MidiConfig::MidiCCs, transport and events from nice-plug into process_with with a preallocated event buffer.
6. Tests (crates/orb-plugin/tests/sync.rs): simulated host transport (start, seek, loop jump, tempo change, host stopped), notes (select at once, latch, note-off, panel wins), deterministic renders (twice, buffer sizes), mapping channel and refusal, snapshot flags; no_alloc covers process_with.
7. docs/plugin.md (host sync section with the method and addresses), docs/initial_investigation.md (TopHat sync facts), cargo test, clippy, fmt, corpus_golden.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a18e52874f5fc9825

RE (2026-11-16, headless copy of ReBirth.dll.i64, read only, closed without saving): how the original places its sequencer at an external position.
- TopHatDriveAudioProc (0x1001B800, export 13, the ReWire precursor) calls sub_1001C0F4 (0x1001B891) with the host's struct each audio call:
  - 0x1001C112..0x1001C121: the host's tempo (+0x950) into the song's tempo (globals +4) and transport+0x144;
  - unless byte_10121ED8 (set when +0x95C is 0, 0x1001B83F): loop start/end/on from +0x960/+0x964/+0x968 into globals +0xC/+0x10/+3;
  - 0x1001C17E: the host's play state +0x954 changed -> sub_1001F7C0 (play sub_1001F7E4 / halt sub_100218A0, no bar locate);
  - +0x958 set (a reposition) -> sub_1001C3C4(+0x94C, the host's tick): unless the countdown transport+0x158 runs, when the bar changed sub_1001FD10 merges every track's recording (sub_1001F520), then sub_1001F980 -> locate sub_1001F984 (0x1001C454);
  - otherwise sub_1001C2F4(tick): host stopped (transport+0x164 == 0) -> countdown 0; host playing with loop on (sub_100215D0, globals +3) and the tick within 192 (0xC0) of loop start (sub_1001F8C0, +0xC) or loop end (sub_1001F8E0, +0x10) -> countdown 10; then countdown 0 -> pos = (double)tick (0x1001C38F: fild; fstp qword [song+8], i.e. sub_1001F8B0's store, no relocate, last tick kept), else countdown - 1.
- MIDI Song Position (EngineHandleMIDI 0x1001B277: (msb<<7 + lsb) * 48 ticks) -> sub_1001C5C4 -> with sync on (transport+0x160) locate (0x1001C5EA). MIDI clock sub_1001C614 works on bars (idiv 0x300).
So: the original follows an external position by setting the position (sub_1001F8B0) every call and locating on a reposition. orb's plugin does the same per 64-frame block (it knows the host position of each block): host start -> stop, locate, play; a jump of >= 1 tick from the expected advance -> locate; always then Position(host tick, fractional). The loop-point hold-off (countdown 10) is not taken: the host's loop is not orb's (pattern mode has no loop).
Engine additions (no existing path changed): Sequencer::set_pos (sub_1001F8B0), Sequencer::relocate_device (slot 2 on one device: a note's pattern at once). orb-player: Command::Position, Command::Relocate, Snapshot.host_sync / midi_in, Audio::output / last_snapshot / set_host_sync / midi_received.
Failed hypothesis: a locate on jumps is needed for the steps to go on after a backward jump. It is not: tick() stores last_tick = t even when t did not pass it (0x1001F44x path), so the steps resume one block later. Mutation (no locate on jumps) left the step test passing; the locate is kept as the original's reposition does it, and a_jump_locates checks its effect (a pending panel pattern is taken at the jump).
Mutation (no Position each block): the_step_follows_the_host_position_and_tempo fails, as it should.

Correction to the note above: the unconditional 'lastTick = t' is the [C] pseudocode of sub_1001EF20 in docs/initial_investigation.md section 5 (task-022), not an address checked here. The IDA worker had exited at its idle timeout before that check; the session was gone (idb_close save=false: not found), so nothing was saved. The scratch copy is deleted.

Acceptance evidence (2026-11-16, this Mac, aarch64-apple-darwin; cargo test --release -p orb-plugin --test sync: 9 pass):
- AC1: sync.rs the_step_follows_the_host_position_and_tempo: host stopped 0..20000 (orb free-runs: position +50 ticks or more, peak > 0.05), start at 20000, seek to beat 37.25 at 60000, tempo 120 -> 133.5 at 80000, loop jump to beat 4 at 100000, stop at 130000 (orb runs on, plays). In every call that follows: engine pos == host tick + one block's advance (|d| < 1e-6), tempo == round(host*1000); more than 200 calls followed, more than 50 checks that the 303 #1's last step (live byte 9) is the position's step; not following only in the block of the start and while the host is stopped. the_host_start_restarts_the_patterns_at_its_position (start at beat 8 -> tick 1536), a_jump_locates (a pending panel pattern is taken at the jump), a_host_tempo_change_changes_orbs_and_the_songs_comes_back (97.25 -> 97250, 151 -> 151000, no tempo -> the song's; snapshot tempo).
- AC2: notes_select_patterns_at_once_and_latch, with the host stopped and playing: note 5 on channel 2 plays pattern 5 in the next block (not at the bar line); note-off, a velocity-0 note-on and note 40 ignored; channels 3..5 notes 31, 9, 17; a panel change (d1c1 = 12 at 40000) waits for the bar line (88200) and wins; the next note (2 at 130000) wins again. notes_on_the_section_channels_do_nothing_in_note_mode.
- AC3: two_renders_of_the_same_host_timeline_are_identical: start, seek, loop jump, stop, 4 pattern notes, a CC and a parameter change; two renders at 512 identical, and buffers 1/63/64/4096 identical (the host tempo 161.4990234375 BPM is 2^-14 beat per frame, so the host's positions do not depend on buffer starts).
- AC4: the_standard_mapping_works_on_its_channel: CC 7 on channel 1 sets the master, on channel 7 not; set_remote_channel(1..=4) refused ("section"), 16 refused; channel 7 then works at once and channel 1 not; transport note 70 does not stop; the channel is saved in the state and restored; a state with a section channel restores channel 1 and last_error.
- AC5: the_snapshot_has_the_sync_led_and_midi_activity: OrbParams::snapshot(): host_sync false while the host is stopped, true in pattern mode with orb and the host playing, false in note mode (mode param 0) with the host playing; midi_in counts 3 for a pattern note, a CC and an unmapped note.
- AC6: docs/plugin.md section 8 (8.1 the original's TopHat sync and MIDI SPP with addresses: TopHatDriveAudioProc 0x1001B800, sub_1001C0F4, sub_1001C3C4/0x1001C454, sub_1001C2F4/0x1001C38F, sub_1001F8B0, sub_1001C5C4/0x1001C5EA; 8.2 the plugin's method), docs/initial_investigation.md section 5 (TopHat sync [C], and the old claim that sub_1001C3C4 is the MIDI song position corrected).
- AC7: cargo test (debug, whole workspace): 83 test binaries, 747 passed, 0 failed. cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean. no_alloc.rs now drives process_with with a playing/stopping/jumping host, pattern notes, note-offs and CCs: 0 allocations at 44100, 48000 and 44101 Hz.
- Corpus: cargo test --release -p orb-engine --test corpus_golden: 4 pass (diag, app, shipped; ORB_CD_ISO unset). The engine change adds two operations (set_pos, relocate_device) and changes no existing path.

Review round 1 (send back), fixed:
1. A saved remote channel of 2..5 made Doc::from_json fail and the restore drop the whole document. Now from_json does not check it; deserialize_fields sets it to channel 1 (0) with last_error and keeps the song, patterns, Vintage and controls (instance.rs). sync.rs the_standard_mapping_works_on_its_channel now edits a pattern and a control before saving, restores the bad state the host's way (parameters, then fields) and asserts the song, the patterns (with the edit), Vintage and every parameter value survive. docs/plugin.md section 5 table and section 8 updated.
2. IDA re-checked (a fresh headless copy of ReBirth.dll.i64, read only, closed with save=false, copy deleted):
   - sub_1001EF20: 0x1001EF94 `cmp edx, eax` (t against [edi+8], lastTick loaded at 0x1001EF4E), 0x1001EF9A `jle loc_1001F475`; loc_1001F475 `mov eax, [esp+54h+var_14]` (t, stored at 0x1001EF96) and 0x1001F479 `mov [edi+8], eax`: lastTick = t whether or not t passed it (the step loop at 0x1001F433..0x1001F473 falls through to the same store). [C]
   - sub_1001C5C4: 0x1001C5EA `call sub_1001F980` (locate, when a current song exists), 0x1001C5F5 `call sub_10007DF0` with ecx = the transport and the tick; sub_10007DF0 is `mov eax, [esp+4]; mov [ecx+18h], eax; retn 4` (stores the tick in transport+0x18). [C]
   docs/initial_investigation.md line rewrapped and given these addresses.
3. task-206 priority Low.
4. core.rs identifies the newest song by orb-player's new load counter (Audio::loads, counted on every Command::Load applied) instead of its address.
Checks after the fixes: cargo test --release -p orb-plugin all pass (sync 9); clippy -D warnings and nightly fmt --check clean; workspace cargo test and corpus_golden below.
Workspace cargo test after the fixes: 83 binaries, 747 passed, 0 failed. corpus_golden: 4 pass.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin has its pattern mode with host sync (branch worktree-agent-a18e52874f5fc9825, one commit).

- Method, from the original: ReBirth.dll's TopHat host sync (TopHatDriveAudioProc 0x1001B800 -> sub_1001C0F4) stores the host's tempo, plays/halts on the host's play state, locates on a host reposition (sub_1001C3C4 -> sub_1001F984) and otherwise sets pos = (double)hostTick each audio call (sub_1001C2F4, 0x1001C38F, sub_1001F8B0's store). The plugin does the same per 64-frame block from per-call transport records: the host's tempo (else the song's), a host start = stop + locate + play, a jump of >= 1 tick = locate, and the position set to the host's fractional tick (192 per quarter note) every block; free-run while the host is stopped. docs/plugin.md section 8; docs/initial_investigation.md section 5 updated (sub_1001C3C4 is the TopHat reposition, not the MIDI song position).
- Notes 0..31 on channels 2..5 in pattern mode select the section's pattern at once (the pattern message + slot 2 on that device), latched; note-off ignored; a panel change wins at the next bar line, the last write wins. Notes never change parameters.
- The Standard Mapping runs on the remote channel (default 1, kept in the state); a section's channel is refused (set_remote_channel) and not restored. Its control actions apply to the engine; its transport actions are dropped (the mode is the host's parameter).
- Snapshot: host_sync (Sync LED) and midi_in (MIDI LED); OrbParams::snapshot(), select_pattern() for the editor. MIDI input is now MidiConfig::MidiCCs.
- Engine/player: Sequencer::set_pos, Sequencer::relocate_device (new, no existing path changed), Command::Position / Command::Relocate.

Tests: cargo test --release -p orb-plugin --test sync 9 pass (AC1-5), no_alloc now with a moving host and MIDI (0 allocations at 44100/48000/44101 Hz); cargo test whole workspace 747 passed, 0 failed (83 binaries); clippy -D warnings and nightly fmt clean; corpus_golden 4 pass (ORB_CD_ISO unset).

Open: task-206 (Decide, @pingw33n): alignment in host time signatures other than 4/4; the plugin uses the quarter-note rule (recommended option A). Not tried in a real host (task-201 AC #1).

Review round 1: a saved section channel as remote channel now restores as channel 1 with last_error and keeps the rest of the state (was: the whole document dropped); the lastTick store (0x1001F479) and sub_1001C5C4 -> sub_10007DF0 (transport+0x18) verified in IDA; task-206 priority Low; the song tempo fallback keys on orb-player load count instead of an address.
<!-- SECTION:FINAL_SUMMARY:END -->
