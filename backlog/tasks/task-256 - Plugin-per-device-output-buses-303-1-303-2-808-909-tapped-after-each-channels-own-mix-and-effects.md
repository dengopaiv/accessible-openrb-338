---
id: TASK-256
title: >-
  Plugin: per-device output buses (303-1, 303-2, 808, 909), tapped after each
  channel's own mix and effects
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-20T10:13:00Z'
updated_date: '2026-11-21T07:18:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/lib.rs
  - crates/orb-engine/src/frame/mod.rs
  - docs/plugin.md
  - docs/plugin_usage.md
  - task-196
  - task-185
priority: medium
type: feature
ordinal: 250000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Requested by the user on 2026-11-20 (item 7 "multi-out" of task-196's deferred plugin features, deferred until after the first plugin release, which has shipped). The VST3/CLAP plugin had one stereo output, the final mix (crates/orb-plugin/src/lib.rs, AUDIO_IO_LAYOUTS: main_output_channels 2). The user wants each device on its own output so it can be recorded and processed separately in the DAW.

What to build:
- Four extra stereo output buses (aux outputs, VST3 and CLAP), named "303-1", "303-2", "808", "909", in addition to the main "Mix" bus.
- Tap point (the user's decision of 2026-11-21, task-259, replacing the post-fader tap decided on 2026-11-20): ReBirth's own ReWire tap (docs/rewire.md section 4). Each bus carries its mixer channel's signal after the channel's insert effects where the mixer routes them (DIST, PCF, COMP, per `Routing` in crates/orb-engine/src/frame/mod.rs; 0x100135AF..0x100135F1), and before its level, pan and on switch. The signal is mono and goes on both sides of the stereo bus. So the channel's level, pan and on switch, the delay (a send whose return goes only into the master) and the master processing (master compressor, master level) are not on the per-device buses.
- The buses are copies, not ReWire's routing: the main Mix output is unchanged and stays bit-exact whether or not the extra buses are connected or used; no device leaves the main mix, nothing in the mixing order, gains or rounding changes. The corpus (`cargo test -p orb-engine --test corpus_golden`, `orb compare --manifest ...`) must stay EQUAL. Turning a channel off in the panel removes it from the Mix but not from its own output.
- No per-instrument drum outputs and no delay-return output (task-259).
- Document the tap with addresses in docs/ (plugin.md, and the engine doc where the mixer is described, linking docs/rewire.md).
- The standalone app and the CLI are not changed.
- The plugin's latency, sample-accurate timing and the 64-frame block alignment are unchanged; the extra buses add no allocation in the audio thread.
- Host behavior: buses not connected by the host cost little (skip the copy when the host gives no buffer); check REAPER (output pins / multichannel routing) and Ableton (Audio From the plugin's track, its outputs) with the user; document the outputs (what each carries, the on switch) in docs/plugin_usage.md (and Help > Plugin Usage, which shows that page), without host-specific routing steps (the user's request of 2026-11-21).

Not in scope: dry (pre-insert) taps, per-voice drum groups, audio inputs into the effects.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The plugin exposes, besides the main stereo Mix output, four stereo aux outputs named 303-1, 303-2, 808 and 909 in VST3 and CLAP; scripts/validate-plugin.sh passes
- [x] #2 Each aux output is its mixer channel after that channel's inserts and before its level, pan and on switch (ReWire's tap, task-259), mono on both sides, with no delay return and no master processing; a test checks a device alone with an insert on reproduces its contribution to the main mix, and a test checks the channel's level, pan and on switch do not change its output
- [x] #3 The main Mix output is bit-identical with and without the aux outputs connected/processed, checked by a test; the reference corpus stays EQUAL
- [x] #4 No allocation in the audio thread for the aux outputs; unconnected buses are skipped
- [x] #5 docs/plugin.md documents the buses and the tap with addresses; docs/plugin_usage.md describes the outputs and the on switch (no host-specific routing steps, the user's request of 2026-11-21); the user checked the routing in REAPER and Ableton
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. orb-engine frame: Frame::block_taps(l, r, Option<&mut Taps>), Taps = [[Block; 2]; 4]; block() calls it with None (unchanged loop). With taps the pan-and-mix loop (0x10013742) computes d = bus*g (f64, as before) once, stores the tap f32(d) and adds d into the master exactly as before (f32(d + master)): same operations, so the master is bit-identical. The tap is the exact per-channel contribution rounded to f32 (the original keeps it in an x87 register and never stores it).
2. orb-engine Engine::render_taps(buf, n, taps): as render, plus the taps frame-interleaved taps[8i + 2c + s], with its own leftover tap block for a partial block; a fault zeroes them.
3. orb-player Audio::enable_taps() (main thread, allocates 8*call_frames floats); render_call renders with taps when enabled (zeroed on a fault), Audio::taps().
4. orb-plugin: AUDIO_IO_LAYOUTS gets four stereo aux outputs named 303-1, 303-2, 808, 909 (main named Mix). Core::new enables the taps; Blocks::next_block returns the main block and its taps; the 44.1k ring keeps the taps of each block; the resampler keeps a tap ring and convolves only the connected buses. Core::process_aux(l, r, aux, ...) with aux = [Option<[&mut [f32]; 2]>; 4]; process_with passes none. lib.rs process maps nice-plug's aux buffers, an empty slice (no host buffer) is None and skipped.
5. Tests: engine unit test (taps equal f32(bus*g), main identical to block()); plugin tests: main bit-identical with and without aux at 44.1k and 48k across buffer sizes; a device alone with an insert (dist on a 303, PCF on the 909, comp on the 808) reproduces the main mix from its bus within 1 LSB; no_alloc with aux connected; exports test the port layout (CLAP audio-ports). corpus_golden and orb compare stay EQUAL.
6. Docs: plugin.md new section (buses, tap, addresses), initial_investigation.md section 4 note, plugin_usage.md routing in REAPER and Ableton. validate-plugin.sh on the bundles. Host checks (REAPER, Ableton) left to the user.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a6eea10703863b353

Research (2026-11-20):
- Mixer block sub_10013444, pan-and-mix loop 0x10013742 (crates/orb-engine/src/frame/mod.rs): master[j] = f32(bus[j]*g + master[j]) with the ramped gain g in x87; the product is never stored ([C] docs/initial_investigation.md section 4). So the tap is defined as f32(bus*g), the exact per-channel contribution, as the task asks for a value the original does not store. The engine computes pl = b*gl once and uses it for both the tap and f32(pl + master): the same f64 operations in the same order, no FMA, so the master is bit-identical.
- Not on the buses: the delay return (0x10013844), the master compressor (0x10013883), the master gain (0x100138AE), the 1e-6 guard.
- The original's own per-channel output entries 2, 3, 4, 11 of the 18-entry output array (ReWire; null in the app, cmp eax, ebp; jnz at 0x100136B1): recorded in docs/plugin.md section 13 as a [G] (mono, pre-level/pan, channel out of the master) and not checked in IDA (no database path given; not needed for the task).
- nice-plug 0.4.2 facts (read in ~/.cargo/registry nice-plug-0.4.2): an aux output without a host buffer becomes empty slices (wrapper/util/buffer_management.rs, about line 335); but the CLAP and VST3 wrappers call the plugin's process only when every output slice is non-empty (buffer_is_valid, clap/wrapper.rs:2114, vst3/wrapper.rs:1505), so a host that passes a null buffer for an unconnected bus would get no processing at all (the Mix too). The aux loop reads one bus past a short count (aux_output_idx > audio_outputs_count, clap/wrapper.rs:2089, vst3/wrapper.rs:1482): the old exports tests passed only the main bus and with the new layout read past it (a panic at buffer_management.rs:359, observed once); the tests now pass all five buses. main_output_name() returns names.main_input (nice-plug-core audio_setup.rs:160): the layout sets main_input to "Mix" too (there is no main input).

Implementation:
- orb-engine: Frame::block_taps(l, r, Option<&mut Taps>) (block() = block_taps(.., None), the old loop untouched for None); Engine::render_taps(buf, n, taps) with TAP_WIDTH = 8 frame-interleaved values and its own leftover tap block; render() unchanged in output.
- orb-player: Audio::enable_taps() (main-thread allocation), render_call renders the taps when enabled (zeroed on a fault), Audio::taps().
- orb-plugin: AUDIO_IO_LAYOUTS main "Mix" plus four stereo aux outputs 303-1, 303-2, 808, 909; Core::new enables the taps; Blocks::next_block returns BlockOut {main, taps}; the 44.1 kHz ring keeps a taps slot per block; the resampler keeps a tap ring and convolves only the connected buses; Core::process_aux(l, r, aux, ...), process_with = process_aux with none; lib.rs maps nice-plug's aux buffers, empty -> None (skipped), and zeroes them when there is no core.
- orb-ui: the Plugin Usage page test counts four tables (the new outputs table).

Results:
- cargo test --no-fail-fast (workspace, ORB_CD_ISO unset: the ISO-gated tests skip): 969 passed, 0 failed, 5 ignored.
- New tests: frame::tests::the_taps_are_each_channels_contribution_and_leave_the_master_alone (exact: taps = bus after an insert times gain; master = guard + four taps in order; master bits = block()'s), frame::tests::render_taps_outputs_the_same_int16_and_frame_interleaved_taps (across the leftover block), resample::tests::the_taps_are_resampled_as_the_main_output, tests/outputs.rs (Mix of full_mix bit-identical with none/some/all buses at 44.1 and 48 kHz, buffers 1/100/512/4096; device alone with an insert: dist_a100_s64 303-1 + dist, pcf_on_909, comp_ch_808 reproduce the Mix within 1 LSB from int16(f32((1e-6 + bus) * m)), m = 0.6338391304016113, worst 1 LSB in each; the other buses below 1e-6), tests/no_alloc.rs with the buses connected in all 16 combinations, tests/exports.rs (CLAP audio ports and VST3 buses: count 5, names Mix, 303-1, 303-2, 808, 909, stereo, main first; all buses passed by the test hosts; every bus sounds in full_mix).
- cargo test -p orb-engine --test corpus_golden: 4 passed. cargo run --release -p orb-cli -- compare --manifest tests/refs/manifest.json: 164 of 164 equal, exit 0.
- cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean.
- cargo xtask bundle orb-plugin --release, then scripts/validate-plugin.sh: "validators: all passed" (pluginval lists Outputs: Stereo x5, main 0 in / 2 out; enabling all buses and disabling non-main buses pass; clap-validator passed).
- The diff against main lists only this task's files; main had not moved (15a9c42), so no rebase was needed.

Pending: AC 5's host part. The user checks the routing in REAPER and Ableton (docs/plugin_usage.md section 6) before the agent review; AC 5 stays unchecked until then. If a host passes null buffers for unconnected buses, nice-plug skips the whole process call (above): that would show as a silent Mix while the device tracks are not routed, and would need a Decide task (patch or fork nice-plug's buffer check).

Rework 2026-11-21 (the user's decision in task-259, after task-257's docs/rewire.md):
- The device taps moved to ReWire's point: the channel's bus after its inserts (0x100135AF..0x100135F1), before its peak meter (0x100135FA), delay send, gain smoothing (0x100136B7) and pan-and-mix loop (0x10013742). Mono, put on both sides of each stereo bus. The bus is a stored f32 block, so the tap is an exact copy; the pan-and-mix loop is back to main's code (the first version's tap of bus*g is gone). Taps = [Block; 4], TAP_WIDTH = 4 (frame i, channel c at [4i + c]).
- A first pass of this rework also added the 12 per-instrument drum outputs (a Sink/Tee in drums.rs teeing each voice into its group). The user withdrew that the same day ("only the tap change"): all of that code was removed (drums.rs, tr808.rs, tr909.rs are main's), nothing of it is in the commit.
- Docs: plugin.md 13 rewritten (the tap from docs/rewire.md section 4 [C], the on switch: a channel turned off leaves the Mix and its delay send but stays on its own bus), plugin_usage.md 6 (mono, pre-fader, the on switch), initial_investigation.md 4 (the sentence points at the new tap), rewire.md 6 (the table rows and "task-256 implements it", present tense).
- Tests: frame::tests::the_taps_are_each_channels_bus_after_its_inserts_and_leave_the_master_alone (taps exactly the bus after an insert, identical under two gain sets, one channel at the guard as when off; master bit-identical to block()); outputs.rs: a_device_alone_with_an_insert_is_its_share_of_the_mix now checks Mix = int16(f32(f32(1e-6 + bus*g) * m)) within 1 LSB (worst 1 LSB in each of dist_a100_s64, pcf_on_909, comp_ch_808; m = 0.6338391304016113), both sides of a bus identical, the other buses below 1e-5 (an idle 303 outputs about 3e-6, its amp's 1e-6 per block; the earlier post-fader bus scaled it below 1e-6); new a_device_output_is_before_its_level_pan_and_on_switch (303-1 level 40, pan 0, level 127 + pan 127, on 0: its bus bit-identical, the Mix differs, and with the channel off the Mix peak is below 1e-3).
- Rebased onto main bee0781 (task-257 merged).
- Results: cargo test --no-fail-fast 970 passed, 0 failed, 5 ignored (ORB_CD_ISO unset); clippy -D warnings and nightly fmt clean; corpus_golden 4 passed; orb compare --manifest 164 of 164 equal; cargo xtask bundle orb-plugin --release then scripts/validate-plugin.sh: validators: all passed.
- AC 5 stays unchecked: the REAPER and Ableton check is the user's.

The user's host check (2026-11-21), on the f437394 bundles installed in /Library, in REAPER and Ableton Live: both good.
- The outputs show as 303-1, 303-2, 808 and 909.
- Each device track carries only its device, with its routed inserts.
- The channel's level, pan and on switch change only the Mix.
- The delay and the master level stay off the device tracks.
- In Ableton the Mix still plays with no device outputs routed: the nice-plug null-buffer risk (docs/plugin.md section 13) did not show.
- Routing used: in REAPER, sends from a 10-channel instrument track; in Ableton, audio tracks with Audio From set to the device output.
AC 5 checked.

The user's request (2026-11-21): the REAPER and Ableton Live routing steps are removed from docs/plugin_usage.md section 6; the output table and the bullets stay. The Mix bullet's "(below)" now reads "mute the instrument track's own output in the host". The description and criterion 5 no longer ask for host routing steps in the docs (criterion 5 rewritten and checked again: the host check itself stands). orb-ui's plugin_usage tests still pass (they check the page against the file and count 4 tables; the removed steps were lists, not tables).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin has four stereo aux outputs, 303-1, 303-2, 808 and 909, besides the main output, now named Mix (VST3 and CLAP, one layout). The tap point is ReBirth's ReWire tap (the user's decision of 2026-11-21, task-259; docs/rewire.md section 4). Each bus is its mixer channel's bus after the routed inserts (0x100135AF..0x100135F1) and before the meter, delay send, level, pan and on switch. It is mono, the same signal on both sides, and an exact f32 copy. The buses are copies: the Mix is bit-identical whatever is connected. Turning a channel off in the panel removes it from the Mix (and its delay send) but not from its own output. There are no per-instrument drum outputs and no delay return.

The engine has Frame::block_taps and Engine::render_taps, orb-player has Audio::enable_taps and Audio::taps, and the plugin has Core::process_aux. The taps go through the 44.1 kHz ring and the resampler beside the int16 blocks, so the buses have the Mix's latency. An unconnected bus is skipped, and nothing allocates in process.

Corpus: corpus_golden passes, and orb compare --manifest gives 164 of 164 equal. Tests: cargo test --no-fail-fast passes 970 with 5 ignored (ORB_CD_ISO unset). Clippy and nightly fmt are clean. scripts/validate-plugin.sh passes on the rebuilt bundles. The docs are plugin.md section 13, plugin_usage.md section 6 (Help > Plugin Usage), initial_investigation.md section 4, and rewire.md section 6.

Host check: the user checked REAPER and Ableton Live on 2026-11-21, and both were good. Each device track carried only its device and its inserts; level, pan and on changed only the Mix; the delay and the master level stayed off the device tracks. In Ableton the Mix played with no device outputs routed, so the nice-plug null-buffer risk did not show.

Branch worktree-agent-a6eea10703863b353, one commit on main bee0781.
<!-- SECTION:FINAL_SUMMARY:END -->
