---
id: TASK-226
title: >-
  Plugin: a note at the very start of the host's timeline plays wrong (drums) or
  is skipped in a render (all devices)
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T17:32:00Z'
updated_date: '2026-11-17T07:37:00Z'
labels:
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/lib.rs
  - docs/plugin.md
  - docs/midi.md
priority: high
type: bug
ordinal: 231000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Report (the user, 2026-11-16, Reaper 7.86 on macOS, plugin built from main ff22a15 plus task-202):**
- A MIDI note placed exactly at the song start (position 0) of a Reaper project plays differently from the same note later in the clip when it is a drum: the 808/909 sound is different, for example a wrong velocity/accent, a cut, or a different sample start.
- When the project is rendered to a file, the first note at position 0 is skipped for every device, the 303s included.
- The user thinks this is unrelated to the features merged today.

Assumption to confirm first: this is Note Mode, where host notes play the devices directly. Check also Latch and Gate Pattern Mode with a note at 0 (task-218's gate path).

**Likely areas:**
- The first process block after activation or a transport start: the order in which the core applies the transport change (Locate/Play/Stop, the sync path, preview exit), the parameter resend and the note events at frame 0 of the same block.
- Something there resets the voices after the note at frame 0 has triggered them, or drops events that arrive before the first "playing" block.
- In an offline render Reaper starts processing at sample 0 with the transport just started, possibly after a fresh activate/reset, so a note at frame 0 lands exactly in that first block.
- For the drums, a "different" sound could mean the accent/velocity state, or the per-instrument parameter resend, isn't applied yet when the note fires.
- Check task-214 (the lost parameter change after activation) and task-225 (dropped commands when the queue is full): they may share the cause.

**To do:**
1. Reproduce it in a test: drive the plugin core or the CLAP/VST3 wrapper the way a host renders. Activate, then process from sample 0 with the transport starting and a note-on at frame 0. Compare the output with the same note at a later frame, shifted, for each device: 303 1/2, 808, 909. Cover a realtime start as well as an offline render (CLAP's `process` with steady time 0, VST3's `processContext` with the playing flag set in the first block).
2. Find the cause, with the code path.
3. Fix it, so that a note at frame 0 of the first block sounds the same as the note later, bit for bit after the shift where the model allows.
4. Run the corpus if orb-engine changes. No allocation or lock in process.
5. Rebuild and validate the bundles. The user checks it in Reaper: a note at 0 played live and in a render, for drums and the 303.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The cause is identified with the code path and a failing test before the fix
- [x] #2 A note-on at frame 0 of the first processed block (realtime start and offline render, CLAP and VST3 paths) produces the same output as the same note later (shifted), for 303 1/2, 808 and 909
- [x] #3 Latch and Gate Pattern Mode with a note at position 0 are checked too
- [x] #4 cargo test, clippy -D warnings, nightly fmt pass; corpus 175/175 if the engine changed; validators pass
- [x] #5 The user checks it in Reaper (live and rendered)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reproduce with a scratch test through Core::process_with: activate, note-on at frame 0 of the first call (host playing from 0) against the same note 8192 frames later, 44.1 and 48 kHz, each device, then after a 1 s warm-up (the live case).
2. Find the cause on the code path (activate -> OrbParams::activate queues Command::Load; Core::process_with applies host events with Audio::apply_now; the first engine block drains the queue).
3. Fix in Core::new (main thread, at activation): drain the activation's Load before the first call, then settle the mixer's gain smoothing with silent blocks so that the first block plays at full gain.
4. Committed tests (tests/first_note.rs): the note at 0 vs later for 303 #1/#2, 808, 909 at 44.1 and 48 kHz (CLAP and VST3 share Core::process_with; the wrapper only converts events and transport), the first call of an offline render and of a realtime start; Latch and Gate Pattern Mode with a note at 0.
5. Adapt the_plugin_plays_as_the_player_one_block_later (the reference player settles the same way), docs/plugin.md section 4 (activation), notes on task-214/225.
6. cargo test, clippy, nightly fmt; bundles built and validated.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ada5a6137fb786a47 (worktree .claude/worktrees/agent-ada5a6137fb786a47), based on main 5fb34ce.

Cause (code path, crates/orb-plugin):
1. Lost note (offline render, every device). OrbPlugin::activate -> instance::core -> OrbParams::activate (instance.rs:574) makes Player::with_call_frames, whose current song is an empty placeholder, and QUEUES Command::Load of the state's engine. Core::process_with applies a host event at frame t right after frame t is produced, through Engine::send -> Audio::apply_now, i.e. to Audio::target() at once. The queue is drained only by the first engine block (Engine::next_block -> Audio::drain), which at 44.1 kHz is rendered at host frame 64 (later when resampled). So every event of the first 64 host frames after an activation went to the placeholder song, and the Load then replaced it (Audio::activate: silence_all + make_current, orb-player lib.rs). Reaper's offline render activates and plays from sample 0, so a note at position 0 is such an event: lost for 303 #1/#2, 808, 909.
2. Softer drum (a hypothesis for the live report). A new song's mixer gains start at 1e-6 (ChannelState::default, ctor 0x100113AF) and rise by the per-block smoothing g = f32(g*0.93 + t*0.07) (0x100136B7, frame/mod.rs) after the first mixer update (counter 10 -> 16, block 6): a fade-in of about 50 ms after every activation. A note that lands after the drain but in the fade sounds softer: measured with the drain alone, 808 BD peak 0.0970 against 0.1351 later, 909 BD 0.1084 against 0.2159 (44.1 kHz). Unconfirmed as the live cause: it explains the live report (a drum at 0 sounding different) only if Reaper activated the plugin within about 50 ms before the note; after a 1 s warm-up the drums measure within 1 LSB of a later note, so the live report is not reproduced here. The user's Reaper check (AC #5) settles it. The 303 (a held note) is hardly affected by the fade.
3. Gate Pattern Mode at the host start: a gate note in the call where the host starts (before the first block that sees it play) found the song stopped and started it free-running (Locate(0), Play, Engine::gate_note), then that block's sync (not following) sent Stop/Locate/Play: the Stop's slot 6 cut the first step. Latch was only hit by cause 1 (its DeviceParam went to the placeholder: the section kept the parameter's pattern).

Measurements (scratch tests, removed): note at frame 0 vs the same note 8192 later, Note mode, host playing from 0: before the fix peak0 = 0 for all devices at 44.1 and 48 kHz (fresh activation); after a 1 s warm-up the drums differed by at most 1 LSB (3.05e-5) and the 303 by 0.32, the same as two later notes differ from each other (8192 vs 16384: drums 1 LSB, 303 0.32) -> not a frame-0 effect but free-running engine state (303 waveform depends on when its note starts). Muting the 303 channels (d0c5, d0c11 = 0) did not remove the drums' 1-LSB differences, so their source was not isolated.
Settle length: drain + K silent blocks; K=0: 808 peak 0.0970 (fade); K=64: 0.1346; K=256: 0.1350708 = the later note's, differences 1 LSB as between later notes. Chose 256 (0.93^230 < 2^-24; a multiple of the 16-block mixer update period).
Failed hypotheses: parameter resend or accent/velocity state at the first block (Core::apply sends nothing at the first call when the values are the activation's; the drum accent is the note's velocity, Tr808::key_on); a Reaper reset() (OrbPlugin has no reset; nice-plug calls activate only from setActive/CLAP activate).

task-214: this change addresses its cause (the activation's Load drained before the first call): a parameter changed in the first call after an activation (d1c3 70 -> 110 via common::set) is now in the engine after the call (before: 70 kept, the change applied to the placeholder whose byte was 20). task-214's own AC #1 test is not written here; task-214 stays To Do. Its AC #2 says the timing tests stay unchanged: settle needed one change in tests/timing.rs (the_plugin_plays_as_the_player_one_block_later: the reference player now settles the same way, since the plugin no longer fades in after an activation); the other timing tests are unchanged and pass.
task-225: not the same cause (garbage queue full); not changed. The drain in Core::new throws the placeholder engine into the garbage queue at activation, as the first block did before.

Fix (crates/orb-plugin/src/core.rs):
- core::settle, called by Core::new on the main thread at the activation: Audio::drain (the activation's Load) then SETTLE_BLOCKS = 256 silent render_call blocks, dropped. No change to process: no allocation or lock added (tests/no_alloc.rs passes).
- Engine::gate_note: the free-running start (Locate(0), Play) only when the host is stopped in the note's own call (Engine::host_playing, the newest transport record); with the host playing the next block's sync starts the song at the host's position.
- tests/timing.rs the_plugin_plays_as_the_player_one_block_later: the reference player settles the same way and renders with render_call (render_f32 after render_call would hand out the last settle block first).
- docs/plugin.md section 4 (activation) and section 11 (gate note at the host start).
Tests (new): tests/first_note.rs (Note mode: 303 #1 note 36, 303 #2 note 48, 808 BD/SD, 909 BD/CH at 44.1 and 48 kHz, offline render = first call after activation, realtime start = host playing after 20480 stopped frames; Latch: notes at 0 select the song's patterns, bit-exact with the song's own selections; Gate: notes held from 0, bit-exact with Latch at warm 0, within 1 LSB after a stopped while). tests/exports.rs: a_note_at_frame_0_of_the_first_clap_call_sounds and a_note_at_frame_0_of_the_first_vst3_block_sounds drive the real library through CLAP (activate, start_processing, process at steady time 0, transport playing, note-on time 0) and VST3 (setupProcessing offline, setActive, setProcessing, process with kPlaying in the first block's processContext, note-on at sampleOffset 0), each bit-exact with Core::process_with and peak > 0.1.
Before the fix (core.rs from main, same tests): first_note 3/3 fail (Note: rate 44100 warm 0 303 #1 peak 0; Latch: patterns [31;4] instead of the song's; Gate warm 20480: difference 0.081); exports: both new tests fail 'the note at frame 0 sounds'.
After: cargo test 835 passed 0 failed; clippy -D warnings clean; nightly fmt --check clean; cargo xtask bundle orb-plugin --release + scripts/validate-plugin.sh: validators all passed. orb-engine unchanged (no corpus run needed).

Rebased onto main c91b55f (task-202): cargo test 842 passed 0 failed, clippy clean, fmt clean, bundles rebuilt and validate-plugin.sh all passed.

Review round 1 (send back): (1) task-214 claims reworded: this change addresses task-214's cause, its AC #1 test is open and task-214 stays To Do; why tests/timing.rs changed is recorded above. (2) The live drum cause is reworded as an unconfirmed hypothesis (AC #5 settles it). (3) AC #2 wrapper coverage extended: exports.rs a_note_at_frame_0_of_the_first_playing_clap_call_sounds and a_note_at_frame_0_of_the_first_playing_vst3_block_sounds now run four cases each (offline render: activate then play from the first call; realtime start: 16 stopped calls of 512 frames, VST3 in realtime mode, then the transport plays with the note at frame 0 of that call), for the 808 bass drum (ch 3, key 36) and 303 #1 (ch 1, key 36), each bit-exact with Core::process_with and peak > 0.1. With main's core.rs the offline 808 case fails in both ('the note at frame 0 sounds'). (4) Commit subject is now player,tests,docs; docs/plugin.md section 4 rewrapped to 100 columns and its settle sentence no longer names the 303s as the only moving state. After: cargo test 842 passed 0 failed (the wrapper tests were renamed, not added), clippy and nightly fmt clean, bundles rebuilt, validate-plugin.sh all passed.

Rebased onto main af012b8 (task-203); docs/plugin.md:134 rewrapped to 100 columns (reviewer nit). cargo test 856 passed 0 failed, clippy and nightly fmt clean. Bundles not rebuilt (the user is testing the e739948 bundles; the code is unchanged since).

Round 3 (2026-11-16): the user checked the e739948 bundles in Reaper 7.86, 48 kHz, Note Mode, the 909 bass drum from a MIDI clip at 120 BPM, loop on (two passes of a 3-hit loop). The render is fixed. Live, the very first hit after pressing play sounds muffled; the same hit at the loop wrap (pass 2, position 0) sounds right. Recording: ~/Documents/timeline0_bug.mp4 (AAC, 48 kHz stereo, 5.5 s), extracted by the supervisor to the scratchpad as t0.wav.

Analysis (numpy, scratchpad analyze_t0.py, lags.py, align.py, run with -I):
- Onsets at 1.632, 2.632, 3.132 (a quieter hit), 3.637, 4.637, 5.137 s. The 909 BD hits: peak 0.207 / 0.209 / 0.207 (later ones) against 0.089 for the first; RMS over 150 ms 0.0377 against 0.0344.
- Timing: aligned on the waveform tail (from 600 samples after the onset on), the first hit sits 5 samples from where the 1 s grid of pass 1 puts it (pass 2: -1 and 0 samples). So it is NOT early or late and not cut by a time shift.
- Its waveform equals the later hits (residual 0.0023 of the energy after the attack) times a gain envelope that rises roughly linearly over the first ~10 ms: least-squares gain per 48 samples from the onset 0.153 0.184 0.258 0.384 0.446 0.579 0.628 0.779 0.838 0.960 1.003, then 1.00 +- 0.02. The later hits have gain 1.00 throughout. Spectrum: the first hit's 3.2-6.4 kHz band is 3.4 dB lower and the centroid the same (209 vs 207 Hz); the muffled character is the attack (the click) scaled down to 15-60%.
- Not the plugin's mixer fade (settle removed it; it would be exponential, g = 1 - 0.93^k per 64-frame block, 0.40 after 7 blocks and 0.9 only after ~32 blocks, ~46 ms), nor the gate fade (one block, Gate mode only), nor an early note cut at the start (the hit is on the grid).
- Cause: a host gain ramp of ~10 ms at playback start, which only hits material at the play position; at the loop wrap there is none. REAPER has an option 'Tiny fade in on playback start' (and on stop); its length and default could not be confirmed (a REAPER accessibility wiki says fades default to 10 ms). Hypothesis to confirm by the user: untick that option (REAPER Preferences, Audio > Playback) and play from 0 again; the first hit should sound like the loop-wrap one. Any instrument's hit at the play position is faded the same way.
- So no plugin fix for the live symptom; no code change in this round.

Side finding (not the user's cause), reproduced and prototyped, then reverted: a host event lands in the 64-frame block that holds its frame, so after the latency compensation it sounds up to 63 engine frames early; at a realtime start off the block grid the early part falls before the play start and the host drops it (Core::process_with, 909 BD at frame 0 after 1000/7777/20000 stopped frames at 44.1 kHz: onset 40/33/32 frames early, 0.0103 of the peak 0.216 in the dropped part, the peak kept). A fix (events reach the first block at or after their frame: render ahead, never early) passed a new unaligned-start test and the CLAP/VST3 wrappers at 44.1 and 48 kHz, but changes task-197's documented timing and needed one-block shifts in 4 gate_mode and 2 sync tests: a design decision, filed as task-227 (Decide, @pingw33n, recommendation (b)). The prototype patch is in the scratchpad (round3-defer-events.patch, not durable); task-227 describes it.

Checks after the rebase on af012b8: cargo test 856 passed 0 failed; clippy and nightly fmt clean. Code unchanged since 525f5ed (round 3 changes only the backlog), so the bundles were not rebuilt.

Round 4 (2026-11-16): the user decided task-227: option (b), host events reach the first block at or after their frame (0..63 frames late, never early). Implemented here (the round-3 prototype reapplied):
- crates/orb-plugin/src/core.rs: Core::render_before(host) runs before the parameter values of a call (at its first frame) and before each event; Out::Direct holds a boxed ring of 3 blocks (made on the main thread in Core::new) instead of one; process_with produces frames up to an event's frame (exclusive), renders ahead, then applies the event. Module doc rewritten.
- crates/orb-plugin/src/resample.rs: Resampler::render_before with the exact rule (b + delay) * den <= (host - 1 + latency) * num (the 'host - 1' absorbs the half frame of the rounded-up latency: with '< host' block 0 counted as before frame 0 at 48 kHz and every aligned note came a block late, onset 179 instead of 109); ring capacity 2W + 4 blocks (was 2W + 2).
- Tests: first_note.rs a_note_at_an_unaligned_realtime_start_sounds_whole (44.1 and 48 kHz, 909 BD after 1000/7777/20000/20001/20480 stopped frames: nothing before the play start, the peak heard whole, onset 0..block late against the aligned run; before the fix at 44.1 kHz warm 1000: 0.0103 before the play start, onset 40 frames early). exports.rs FIRST_NOTES now 8 cases with a rate and a stopped length in frames (44.1 kHz: 808/303 offline, 808/303 after 8192, 909 after 7777; 48 kHz: 909 offline, 909 and 303 after 7777) and checks nothing before the play start; both wrapper tests fail on the round-2 code (909 at 7777: 0.0103 before the play start).
- Test shifts (each: an event off the block grid now reaches the next block):
  gate_mode a_note_starts_and_its_note_off_stops_the_section_at_its_block: the note-on and note-off at call frame 1000 of an aligned host: first and last sound in 1088..1152 (was 1024..1088), silent from 1152 (was 1088): frame 1000 is in the block of 960, the first block at or after it starts at 1024 and is heard at 1088.
  gate_mode with_the_host_stopped_a_note_plays_from_step_1_at_the_hosts_tempo: (1) on(63) then the next call grows from 64 to 128 frames: the note's block starts at the next frame and is rendered when due, in the call after the next; (2) the note-off at frame 0 of a call that starts off the grid (52484 = 4 mod 64): silent from 192 (was 128); (3) the later on(63) in a 128-frame call needs one more 64-frame call before the snapshot shows the song playing.
  gate_mode the_last_note_pressed_wins: the None case checks silence from 192 (was 128): an event at frames 1..63 (10, 3) reaches the block of frame 64, which fades out (64..127 compensated, heard 128..191).
  gate_mode the_stop_fades_out_over_one_block: the note-off at frame 0 of calls off the grid (4096 + 1000 + 997k): last sound < 192 (was < 128), the fade block starting 64..127 frames into the call.
  sync notes_select_patterns_at_once_and_latch and with_the_host_stopped_notes_only_select: after(f) is now f.div_ceil(64) * 64 + 128 (was f / 64 * 64 + 128): the snapshot at the end of the call after the event's block; the 'not yet' probe moves from 9984 to 10048.
- docs/plugin.md section 4 (Events, one block ahead) and section 11 (note-on, snap rationale, note-off), docs/midi.md (the caller's block choice, Gate note-on) describe the new rule.
Checks: cargo test 857 passed 0 failed; clippy -D warnings clean; nightly fmt clean; orb-engine unchanged (no corpus run); bundles rebuilt, validate-plugin.sh all passed. task-227 set Done with the decision in this commit.

User check (AC #5), 2026-11-17, on 5f1a7c3 bundles in Reaper: passed ('looks good'): live start from position 0 with REAPER's tiny fade-in on playback start switched off, render from 0, and the timing. This confirms round 3: the muffled live first hit was REAPER's playback-start fade, not the plugin. The user keeps task-227's never-early timing. Review nit fixed: the orphaned wrap in docs/midi.md (Gate Pattern Mode bullet) reflowed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Done: the user's Reaper check passed on 2026-11-17 (5f1a7c3 bundles: live start from 0 with REAPER's tiny fade-in off, render from 0, timing).
Cause: an activation queued the state's song (Command::Load) for the first engine block's drain, 64 host frames later, while host events are applied at once: a note (or parameter change) in the first frames went to the player's empty placeholder song and was lost when the Load replaced it - the skipped note at 0 in an offline render, every device. A new song's mixer gains also fade in from 1e-6 (0.93/0.07 per-block smoothing, 0x100136B7), so a drum in the first ~50 ms after an activation sounded softer (808 BD peak 0.097 vs 0.135). It was a hypothesis for the live report; round 3 found the live symptom has another cause. In Gate Pattern Mode a gate note in the host's start call also started the song free-running and the first synced block's Stop cut it.
Fix: core::settle in Core::new (main thread): drain the Load, render 256 silent blocks; Engine::gate_note leaves the start to sync when the host plays in the note's call. This addresses task-214's cause; its own test is open and task-214 stays To Do. tests/timing.rs changed in one test (the reference player settles the same way). task-225 is unrelated.
Round 3 (the user's Reaper check): the render is fixed; live, the first hit after pressing play is muffled by a ~10 ms gain ramp at playback start (measured: the hit is on time and equals the later ones times a ramp 0.15 -> 1.0 over ~10 ms), REAPER's tiny fade-in on playback start, not the plugin (confirmed: the hit is right with it off). A side finding (events up to 63 engine frames early, their start cut at a realtime start off the block grid) became task-227; the user chose (b) and round 4 implements it: events reach the first block at or after their frame (0..63 frames late, never early); 4 gate_mode and 2 sync tests shift by a block (notes).
Tests: tests/first_note.rs (Note/Latch/Gate, 44.1 and 48 kHz, offline and realtime start); tests/exports.rs CLAP and VST3 wrapper tests, offline render and realtime start, 808 and 303; the offline cases fail before the fix. cargo test 857 pass (on main af012b8), bundles rebuilt and validated, clippy, fmt, validators pass; no engine change.
Branch worktree-agent-ada5a6137fb786a47, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
