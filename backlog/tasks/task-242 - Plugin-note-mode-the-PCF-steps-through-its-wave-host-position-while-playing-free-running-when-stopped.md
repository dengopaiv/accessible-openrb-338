---
id: TASK-242
title: >-
  Plugin note mode: the PCF steps through its wave (host position while playing,
  free-running when stopped)
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-18T05:29:00Z'
updated_date: '2026-11-18T07:26:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-engine/src/pcf.rs
  - crates/orb-engine/src/frame/mod.rs
  - crates/orb-plugin/src/core.rs
  - docs/plugin.md
priority: high
type: bug
ordinal: 400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user (2026-11-18) dragged an 808 pattern out of the plugin (task-203) into a REAPER track in note mode, and it sounds different from the same pattern in the preview: the PCF does not work the same way.

**Cause.** The PCF picks its wave step from the song's tick: `crates/orb-engine/src/pcf.rs:318` calls `quotient(ctx.tick, res)`, the original's 0x100166D0. In note mode (docs/plugin.md section 9) the song is stopped, so `ctx.tick` stays at the bar start. The PCF stays on one step, with no new-step triggers and no rhythmic sweep. In the preview (and in pattern mode) the song plays, the tick advances, and the PCF triggers on every step. The original behaves like note mode does now when its song is stopped. Note mode is new engine behavior, though, and a dragged pattern should sound as its preview.

**The user's decision (2026-11-18).** In note mode, outside the preview, the PCF's tick comes from:
- the host position while the host plays: `beats * 192`, as pattern mode's position in docs/plugin.md 8.2, so a dragged pattern placed on a bar lines up with the preview's PCF steps;
- a free-running clock while the host is stopped (or reports no position): it advances at the playing tempo (the snapshot tempo, `tempo * TEMPO_STEP` per block, as the song clock) from where it was.

**Scope.**
- This only changes the tick the PCF (and any other device that reads `ctx.tick` for a free-running modulation; check the delay and others and record what you find) sees while the song is stopped in note mode.
- Pattern playback, the preview, Latch and Gate Pattern Mode, and the corpus renders do not change. The song stays stopped; the sequencer does not step devices.
- Likely shape: an engine-side "note-mode clock" tick passed in the BlockContext when the song is stopped and the frame is in note mode, set from orb-player commands (`Command::Position`-like) by `src/core.rs` `Engine::sync`. Choose the design and record it in docs/plugin.md section 9 (a new 9.x on the PCF clock).
- Look for anything else in the drag-out vs preview comparison that differs for the 808 or 909 (accent mapping, the delay's tempo, shuffle: the drag-out's note timing with shuffle) and record findings; fix only the PCF here, and create follow-up tasks for others.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 In note mode with the host playing, the PCF's step follows the host position (beats*192): a test plays an 808 pattern as notes (the drag-out's notes) through the plugin core with a host position starting on a bar, and the PCF's step sequence (and its triggers) matches the same pattern played in the preview / pattern mode at the same tempo; the output is compared and the remaining difference is recorded (bit-exact where possible)
- [x] #2 In note mode with the host stopped, the PCF free-runs at the tempo from its last position; a test shows its step advances across blocks and continues from the last host position when the host stops
- [x] #3 Pattern mode (Latch, Gate), the preview and the standalone player are unchanged: `cargo test -p orb-engine --test corpus_golden` and the full `cargo test` pass, and the existing plugin tests (timing, sync, first_note, exports) pass unchanged
- [x] #4 docs/plugin.md section 9 describes the note-mode PCF clock (host position or free-running), and what else was checked for drag-out vs preview differences
- [x] #5 cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check are clean
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Engine (crates/orb-engine/src/frame/mod.rs): a note-mode clock, Frame::note_clock: Option<f64> (None by default, so the corpus, the app and pattern playback never see it). While it is Some and the song is stopped, Frame::block advances it by tempo*TEMPO_STEP (the song clock's advance, 0x1001EF51 order) before the devices render and gives trunc(clock) as BlockContext::tick; otherwise ctx.tick stays the song's tick_pos. Engine::set_note_clock(NoteClock::{Off, Run, At(tick)}): Off clears it, Run starts it from the song's position if off (keeps it if on), At sets it (Command::Position's twin).
2. orb-player: Command::NoteClock(NoteClock), applied to the newest song.
3. orb-plugin src/core.rs Engine::sync: in note mode outside the preview, after the Stop the mode owes, send NoteClock::At(host tick) (beats*192 extrapolated as pattern mode's) while the host plays with a position, NoteClock::Run otherwise (when the engine's clock is off); anywhere else NoteClock::Off when it is on. The clock is set before the block and the block adds its advance, as pattern mode's Position, so the PCF sees the same tick in both modes.
4. Check the other ctx.tick readers (only pcf.rs:318) and the delay (ctx.tempo, which sync sets in every mode); compare the drag-out vs preview output for an 808 pattern with the PCF on its channel; record the remaining difference.
5. Tests: tests/note_mode.rs (AC 1: 808 clip notes in note mode vs Latch, PCF steps per block equal, output compared; AC 2: host stopped free-run and continuation), engine unit test of the clock; full cargo test, corpus_golden, clippy, nightly fmt.
6. docs/plugin.md 9.5 (the note-mode PCF clock and the drag-out vs preview findings); follow-up tasks for other differences.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ac12181c464339de4 (the task file was created uncommitted in the main checkout; copied verbatim into the worktree so it lands in this commit).

Research:
- Only the PCF reads BlockContext::tick (grep of crates/orb-engine/src: pcf.rs:318 control(), quotient 0x10016792..0x100167AE). The delay reads ctx.tempo (delay.rs:136 length()), which the plugin's sync sets to the host tempo in every mode, so the delay's time in note mode is the preview's. The mixer update's ctx (Frame::mixer_update) passes the song's tick, but the PCF's slot 16 (update, 0x10016C30) does not read it.
- Pattern mode's sync (src/core.rs) sets Command::Position(host tick) before each block and the clock (seq.rs tick, 0x1001EF51) adds tempo*TEMPO_STEP, so ctx.tick = trunc(host tick + inc). The note-mode clock does the same: set before the block, the block adds the advance, so both modes see the same tick.

Design (done):
- Engine: Frame::note_clock: Option<f64> (None in every new song: the corpus, the app and pattern mode never see it). Frame::block_tick: with the clock on and the song stopped (globals.play == 0) add tempo as i32 as f64 * TEMPO_STEP (the song clock's operand order), ctx.tick = seq::trunc(clock); else seq.tick_pos(). Engine::set_note_clock(NoteClock::{Off, Run, At(f64)}): Run starts an off clock at seq.pos and keeps a running one.
- orb-player: Command::NoteClock(NoteClock), applied to target() (the newest song).
- Plugin core.rs sync: host_tick (beats*192 extrapolated, the old pattern-mode formula moved up) computed when the host plays with a finite position; Engine::note_clock(host_tick) after ensure_stopped/ensure_playing: note mode outside the preview → At(host tick) or Run (when off); otherwise Off when on. Pattern path unchanged (Off when on).

Failed hypotheses / test fixes:
- First test version expected tick/24 for wave 0: wave 0 has resolution 16 (tick/48); the test now uses pcf::quotient with pcf::pattern(WAVE).
- Core::blocks() does not count the 256 settle blocks (settle calls render_call directly).
- First aligned comparison differed by up to 1.27: the pattern's step at tick 0 after the clean start (Stop, Locate 0, Play, Position 0) plays in the second block (seq.tick steps the devices only when trunc(pos) > last_tick, i.e. from tick 1), while a note at frame 0 plays in the first. pattern_frame uses max(t, 1).

Results (tests/note_clock.rs, 2 loops at 120 BPM, host rolling from bar 1, PCF wave 0 on the section's channel, one-frame host calls so every block is seen):
- PCF step quotient after every block: identical between Latch Pattern Mode and the drag-out in note mode, for 808_accent_level127, 909_accent_level127 and shuffle_drums_g64 (808+909, shuffle 64).
- Output, notes at the host's frames (frame_of = first frame whose position reaches the tick): first difference frame 64, max 1.587 (808), 1.743 (909), 0.214 (shuffled): the notes sound up to one block later than the steps (task-227's rule: a note reaches the first block at or after its frame; a step plays in the first block whose advanced position reaches it), and the tick-0 step one block later than the tick-0 note.
- Output, notes in the pattern's blocks: within one LSB (3.05e-5): first difference frame 3394 (808, 3452 frames differ), 3396 (909, 3408), 3438 (shuffled, 3639) of 176400. Also present with the PCF routed off (experiment with d0c2=0: first 3364, max 1 LSB), so it is the drum path, not the clock; not isolated: follow-up task-243.
- Host stopped: the clock runs from the last host position (+inc per block); with the host stopped from the start it runs from the song's position 0; 1 s advances 8 steps of wave 0.
- Pattern modes, the preview: note_clock stays None (test the_pattern_modes_and_the_preview_run_no_note_clock).

Repro: cargo test -p orb-plugin --test note_clock -- --nocapture (prints the differences).

AC evidence:
- AC 1: tests/note_clock.rs the_dragged_808_pattern_steps_the_pcf_as_the_pattern_does (and the 909 and shuffled twins): the drag-out's notes (clip::notes) into note mode with the host rolling from bar 1 at 120 BPM give the same PCF step quotient after every block as Latch Pattern Mode; output differences as recorded above (one block of note timing at the host's frames; within one LSB in the pattern's blocks).
- AC 2: with_the_host_stopped_the_pcf_runs_on_from_the_last_host_position (clock = last host tick + inc, then +inc per block after the stop, 8 steps per second) and a_stopped_host_from_the_start_runs_the_clock_from_the_songs_position.
- AC 3: cargo test (whole workspace) 903 passed, 0 failed, including corpus_golden 4/4 and the plugin's timing, sync, first_note, exports, note_mode, gate_mode, preview tests unchanged; the_pattern_modes_and_the_preview_run_no_note_clock shows the clock stays off there; every new song has note_clock None so no corpus path changes.
- AC 4: docs/plugin.md 9.5.
- AC 5: cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean.
Follow-up: task-243 (the one-LSB drum difference). Main had not moved (5e4a14f): no rebase needed; git diff main --stat lists only this task's files.

Review round 1 (send back on df10ce2), fixed:
1. Preview exit with the host stopped restarted the clock from the bar start: the preview turns the clock Off, and Run after the exit's Stop took seq.pos, which the Stop's locate (sub_1001E100) had moved to the bar start. Fix: core.rs sync calls note_clock before ensure_playing/ensure_stopped, so Run takes the position the preview's song reached; At and Off are unaffected by the order. New test leaving_the_preview_with_the_host_stopped_continues_the_pcf_from_the_previews_tick: a second of preview (pos > 384), then the clock runs from that pos + inc per block while the song sits at tick 0. The preview test's back-to-note-mode check now asserts the clock equals the host tick + inc (it only checked is_some). docs/plugin.md 9.5 describes it.
2. docs/plugin.md 9.5: the 'original behaves so' sentence now cites the PCF block 0x100166D0 reading the tick (sub_1001F870 at 0x10016756) and the clock sub_1001EF20 returning while the play flag is clear (0x1001EF34, docs/initial_investigation.md section 5 [C]).
3. note_clock.rs: the host-frame comparisons assert the measured upper bounds (1.5874329, 1.7430725, 0.21432495) with a comment naming task-227's timing rule, instead of requiring a mismatch.
4. task-243 ordinal set to 600.
5. main has not moved (5e4a14f). cargo test 904 passed 0 failed (corpus_golden 4/4); clippy -D warnings and nightly fmt check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
In the plugin's note mode the PCF now steps through its wave. A note-mode clock in the engine (Frame::note_clock, Engine::set_note_clock with Off/Run/At, orb-player Command::NoteClock) gives the inserts their tick while the song is stopped. Each block adds the song clock's advance (tempo*TEMPO_STEP) to it. The plugin's sync sets it to the host's tick (beats*192, pattern mode's formula) before every block while the host plays. Otherwise it runs free from where it was. It is started before the Stop the mode owes, so leaving the preview with the host stopped continues from the tick the preview reached (review round 1). It is off in the pattern modes and the preview. Every new song has it off, so the corpus, the app and pattern playback are unchanged. Only the PCF reads the tick; the delay's tempo is already the host's.

Measured (tests/note_clock.rs): the drag-out of an 808, a 909 and a shuffled 808+909 pattern steps the PCF block for block as the patterns do in Latch Pattern Mode. At the host's frames the notes sound up to one block later than the steps (task-227's rule), with differences of at most 1.59, 1.74 and 0.21. Put in the pattern's blocks, they match within one LSB. That remaining LSB is the drum path's, not the clock's (it also shows with the PCF off), and goes to follow-up task-243. Documented in docs/plugin.md 9.5.

Branch worktree-agent-ac12181c464339de4, one commit. Tests: cargo test 904 passed 0 failed (corpus_golden 4/4); clippy -D warnings and nightly fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
