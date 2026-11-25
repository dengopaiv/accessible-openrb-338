---
id: TASK-300
title: >-
  Plugin: Options > Output Routing, per-drum outputs and taking channels out of
  the mix
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-25T10:19:00Z'
updated_date: '2026-11-25T11:42:00Z'
labels:
  - player
  - engine
  - ui
milestone: m-7
dependencies: []
references:
  - docs/rewire.md
  - docs/plugin.md
  - crates/orb-plugin/src/lib.rs
  - crates/orb-plugin/src/core.rs
  - crates/orb-engine/src/frame/mod.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 272700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user decided on 2026-11-25 to give the plugin the original's ReWire routing: per-drum outputs, and channels that can be taken out of the mix. Today the plugin has the main output Mix and four stereo aux outputs 303-1, 303-2, 808 and 909, which are copies (task-256, docs/plugin.md section 13). VST3's activateBus cannot drive this: nice-plug 0.4.2 ignores its state (src/wrapper/vst3/wrapper.rs:347), and hosts do not report whether an output is connected. So the user chooses in a window.

**The window.** It is the plugin's Options > Output Routing..., in the plugin only. The standalone plays a stereo mix and does not show it. All boxes are checked by default.

    To Mix-L/R
    [x] 303-1   [x] 303-2   [x] 808   [x] 909
    To 808
    [x] BD  [x] SD  [x] LT  [x] MT  [x] HT  [x] RS  [x] CP  [x] CB  [x] OH  [x] CH  [x] CY
    To 909
    [x] BD  [x] SD  [x] LT  [x] MT  [x] HT  [x] RS  [x] CP  [x] CH  [x] OH  [x] CC  [x] RC

**The user's review of 2026-11-25:** no numbers on the labels. Each box has the tooltip "#N",
N the output's VST3 bus index (Mix is bus 0: 303-1 #1, 808 BD #5, 909 RC #26), generated from
the routing table. REAPER numbers the channels flat from 1 (Mix 1/2, bus n 2n+1/2n+2), so the
labels' numbers did not match REAPER's; other hosts show the names. User-visible text says
"output", not "bus".

**The user's decision of 2026-11-25:** one output per instrument, not ReWire's six groups
(BD, SD, Tom, Per, HH, CY). The 808's LT/LC, MT/MC, HT/HC, RS/CL and CP/MA share a voice, so
each pair is one output. The mapping of voices to outputs is one table per machine
(orb-engine `tr808::OUTPUTS` and `VOICE_OUTPUT`, `tr909` the same), which the engine, the
buses, the state and the window follow. The window fits the plugin's minimum window
(536 x 485), scrolling if needed.

**Semantics.** They follow docs/rewire.md sections 4 and 6, the original's ReWire output, with addresses.
- **A device unchecked under Mix-L/R** (303-1, 303-2, 808 or 909) leaves Mix-L/R. It keeps its delay send (send x level), as ReWire does (rewire.md section 4 summary table). Its own output stays what it is today: the tap after the channel's inserts, before level and pan.
- **A drum instrument unchecked under 808 or 909** leaves the machine's bus. Its voice (both BD voices for BD) adds into its own buffer instead of the bus, as a ReWire group does (the 808 entries 5..10 and 909 entries 12..17 in rewire.md, slot 14 0x100280B0 / 0x1002B6F0), at instrument granularity. So the instrument is also out of the machine's inserts, meter, level, pan, delay send, Mix-L/R and the machine's own output (3)/(4). The rest of the bus keeps its voices in their order, so the bus's f32 sum is the same sum without the removed terms. An idle instrument is exactly 0, with no 1e-6 guard.
- **An instrument's own output** (5)..(26) always carries the instrument's raw voice sum, whether checked or not. That is the trigger gain with the level knob, accent and envelope, and nothing after it. When the instrument is checked, its output is a copy: compute its sum separately, so that the machine's bus keeps the original's exact accumulation order and the Mix stays bit-identical. Never derive the bus from the instrument sums.
- **Defaults:** with every box checked, the Mix and the four existing outputs are bit-identical to today. The corpus and the plugin's existing output tests guard this.

**Buses.** Add twenty-two stereo aux outputs (5)..(26), L = R like the existing four, AFTER the existing four. Hosts store routing by bus index, so a released project keeps 303-1/303-2/808/909 on buses 1..4. Name them in the 303-1 style without ORB- or TR-, '808 BD' .. '909 RC' (the user's decisions; the 909's cymbals are CC and RC as in the drum editor). The original's ReWire group names are BD, SD, Tom, Per, HH, CY. Check nice-plug 0.4.2's caveats with 26 aux buses (docs/plugin.md section 13: process runs only when every output bus has a buffer; the off-by-one on a lower count) in REAPER and Ableton.

**State.** The twenty-six checkboxes are saved in the plugin's state, keyed by stable ids from the tables ('303-1', '808', '808.lt', ...; an absent key is checked), not by the display names, which may be renamed. They are not parameters, so the parameter ids and param_ids.txt stay unchanged. A state saved by the released plugin (v2026.1118.0919) loads with every box checked. process allocates nothing: no_alloc.rs stays green.

**Docs:** docs/plugin.md section 13 (routing, buses, semantics with rewire.md references) and the plugin's menu in docs/ui_spec.md.

**Tests:**
- The defaults are bit-identical to today.
- Each device unchecked: the Mix equals a reference made from the channel's level path removed but its delay send kept.
- Each instrument unchecked: it is gone from the bus, the machine's output, the meter and the delay, and its output carries the instrument.
- An instrument checked: its output equals that instrument's sum while the Mix is unchanged.
- State round trip, and loading a released-plugin state.
- Bus count, order and names.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Options > Output Routing in the plugin shows the 26 checkboxes in the three sections, labeled with the outputs' names, each with the tooltip #N (its VST3 bus index), fitting the 536 x 485 minimum window; they are saved in the plugin state by stable ids, and a state from the released plugin loads with all checked
- [x] #2 Twenty-two stereo per-instrument aux outputs (5)..(26) follow the existing four; each carries its instrument's raw voice sum whether checked or not
- [x] #3 Unchecking a device takes it out of Mix-L/R and keeps its delay send; unchecking a drum instrument takes it out of its machine's bus (inserts, meter, level, pan, delay, Mix-L/R and the machine's output), as docs/rewire.md describes for a group
- [x] #4 With every box checked, the Mix and the four existing outputs are bit-identical to before (corpus and plugin tests); parameter ids are unchanged; process allocates nothing
- [x] #5 docs/plugin.md and docs/ui_spec.md describe the routing; cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #6 The user has checked the window and the routing in REAPER and Ableton (including the nice-plug caveats with 26 aux buses), including a project saved with the released plugin
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. orb-engine: a per-machine output table (tr808::OUTPUTS / VOICE_OUTPUT, tr909 the same; the supervisor asked on 2026-11-25 that the voice-to-output mapping be one table per machine, so the count and names can change), voices generic over an output sink so a voice term goes into the bus, its own output block, or both (bus and copy: same f64 term, same f32 store, so the bus is bit-identical). ChannelDevice::process_split for the 808 and 909; frame::Routing (to_mix per channel, to_bus per drum output); TAP_WIDTH = 4 + outputs of the 808 + of the 909; block_taps skips the gain smoothing and the pan-and-mix loop of a channel not to the Mix (rewire.md 4, 0x100136AB/0x100136B1), keeps meter and send; the gate fade applies to the machine's outputs too.
2. orb-player: Audio::set_routing, applied to the current and pending engine before each call (a new song keeps it).
3. orb-plugin: buses after the existing four from the tables (names '808 BD'..); Shared routing atomic; Core applies it each block; Doc/state key 'routing' (map by output name, absent = checked); AuxOut widened.
4. orb-ui: Embed::routing/set_routing, Options > Output Routing... window (plugin only), generated from the tables.
5. Tests: engine (defaults identical, device out of mix, group out of bus, copy equals group), plugin outputs/exports/state/no_alloc; docs plugin.md 13, rewire.md 6, ui_spec.md.
6. corpus_golden, cargo test --workspace, clippy macOS+Windows, nightly fmt; bundle; commit To Do for the user's check.

Revised by the user's decision of 2026-11-25: one output per instrument (11 per machine), 26 aux buses, 30 boxes in the window (4 + 11 + 11 + the mix rows), state keyed by stable ids; the window scrolls if needed to fit the 536 x 485 minimum.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa089865e75ae6103 (rebased onto 1422d2a).

Decision (the user, 2026-11-25, through the supervisor): one output per instrument, not ReWire's six groups. 808: BD, SD, LT, MT, HT, RS, CP, CB, OH, CH, CY (LT/LC, MT/MC, HT/HC, RS/CL, CP/MA share a voice, so one output each); 909: BD, SD, LT, MT, HT, RS, CP, CH, OH, Crash, Ride. 26 stereo aux buses: (1)..(4) 303-1, 303-2, 808, 909; (5)..(15) the 808's; (16)..(26) the 909's. Bus names '808 BD' .. '909 Ride'. State keys are stable ids ('808.lt', the supervisor's instruction: names may be renamed, ids never). Unchecking an instrument takes its voice(s) out of the machine's bus as rewire.md 4 describes for a group, at instrument granularity. Also from the supervisor: the voice-to-output mapping is one table per machine (tr808::OUTPUTS / VOICE_OUTPUT, tr909 the same) that the engine, the buses, the state and the window all follow.

The window has 4 + 11 + 11 = 26 checkboxes (the supervisor's message said 30; the description and AC 1 say 26).

Naming (the user, 2026-11-25, through the supervisor): the 909's cymbal outputs are CC and RC (as the drum editor, crates/orb-ui/src/drum_editor.rs:60), buses '909 CC' / '909 RC', window rows '(25) CC' / '(26) RC', state ids '909.cc' / '909.rc'. Names keep a space ('808 BD'). The parameter display names (909 Crash Level ...) are unchanged.

Implementation notes:
- No new fact about the original was taken from IDA: the semantics are docs/rewire.md section 4's [C] items (task-257): the bypass cmp eax, ebp 0x100136AB / jnz 0x100136B1 past the smoothing 0x100136B7 and the mix loop 0x10013742 (meter 0x100135FA and send 0x1001364A before it), and the per-group entries of the 808 slot 14 (0x100280D8..0x10028273) and the 909's (0x1002B718..0x1002B8AF). The supervisor's prompt gave no IDA path, and nothing new was needed.
- Engine: drums::Out (out[i] = f32(out[i] + x), x the voice's f64 term; an IEEE sum is commutative, so the rewritten voice loops store the same bits), drums::Sink::Both adds the same term into the bus and the output copy, drums::render_split routes by VOICE_OUTPUT; ChannelDevice::process_split (808, 909; others zero the outputs). Frame::routing (OutputRouting: to_mix[4], to_808[11], to_909[11]; bits in tap order), TAP_808 = 4, TAP_909 = 15, TAP_WIDTH = 26. block_taps runs process_split only with taps or an instrument off the bus, else the export's process: the corpus path is unchanged. A channel out of the Mix continues before the smoothing (gains freeze, Q-6 in docs/original_bugs.md now says the plugin models it). The gate fade and close apply to the machine's outputs too.
- Player: Audio::set_routing, applied to the current engine before each render_call (a loaded or switched song keeps it).
- Plugin: crates/orb-plugin/src/routing.rs (AUX_OUTPUTS 26, OUTPUT_NAMES and ROUTING_IDS built const from the tables, to_saved/from_saved), Shared routing AtomicU32 read before each engine block, Doc/state key routing (map id -> bool; absent = checked), OrbParams::routing/set_routing; Embed::routing/set_routing; orb-ui routing_window.rs (Options > Output Routing..., plugin only, three columns of 4/11/11 boxes, scrolls if the area is short).
- Corpus: cargo test -p orb-engine (corpus_golden 4 passed, 244 lib tests incl. 4 new routing tests) all green after the engine change.

Results (before the commit):
- cargo test --workspace: 1164 passed, 0 failed, 5 ignored. corpus_golden 4 passed (diag, app at the fitted N, shipped: EQUAL, the engine change leaves the export path unchanged); orb-engine lib 244 incl. frame::tests::routing (4 new: default routing identical, each of the 22 drum outputs on/off the bus against renders with the voices silenced, each device out of the Mix against zero gain with send/meter/taps kept and gains frozen, bit order); orb-plugin outputs 5 (2 new), state 10 (1 new: round trip by keys, a state without 'routing' loads all checked), exports 3 (27 VST3 buses and names), no_alloc 4 (26-bus patterns and routing changes while playing), editor (new: Options > Output Routing fits 536 x 485, every box inside, clicks set the routing), params 8 (param_ids.txt unchanged), orb-plugin routing unit tests 3, orb-ui routing_window 1.
- The REAPER fixtures (crates/orb-plugin/tests/reaper/*.RPP) were regenerated (cargo run -p orb-plugin --example reaper_fixtures): only the state JSON changed (the new 'routing' key); the pin map stays the released plugin's 10 outputs (cases.rs OUTPUTS, commented), as a project saved with the released build.
- cargo clippy --all-targets -D warnings (macOS): clean. cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets -D warnings (CL, VCINSTALLDIR, XWIN_ACCEPT_LICENSE, Homebrew LLVM on PATH): clean. cargo +nightly fmt --all --check: clean.
- Bundle: cargo xtask bundle orb-plugin --release -> target/bundled/OpenRB-338.vst3 in the worktree.

Round 2 (the user's review, 2026-11-25, through the supervisor): the window's labels lose their numbers ('303-1', 'BD', 'CC'); each box gets the tooltip '#N', N the output's VST3 bus index (Mix 0, 303-1 #1, 808 BD #5, 909 RC #26), from the routing table (routing_window::columns, tooltip). Why: REAPER numbers the channels flat from 1 (Mix 1/2, bus n 2n+1/2n+2). The earlier note's '(25) CC' row labels are superseded. Docs: plugin.md 13.1, ui_spec.md, plugin_usage.md 6; description and AC 1 updated. The editor test now reads each column under its heading (names only, inside the window at 536 x 485), hovers LT, RC and 303-1 for '#7', '#26', '#1', and clicks by position. Checks: cargo test -p orb-plugin -p orb-ui 659 passed, 0 failed, 3 ignored; clippy -D warnings clean on macOS and xwin x86_64-pc-windows-msvc; nightly fmt --check clean; bundle rebuilt (target/bundled/OpenRB-338.vst3).

Round 3 (2026-11-25): the user signed task-300 off ("looks good") in REAPER and Ableton with the bundle of a0836fb: AC 6.
Help > Plugin Usage (docs/plugin_usage.md, embedded by crates/orb-ui/src/plugin_usage.rs) still had ORB- names that task-293 missed: "two ORB-303 bass lines" -> "two 303 bass lines", ORB-303 1/2 -> 303-1/303-2 (incl. "ORB-303 1 Cutoff" -> "303-1 Cutoff"), ORB-808 -> 808 ("ORB-808 BD Level" -> "808 BD Level"), ORB-909 -> 909, "ORB-303 (channels 1 and 2)" -> "303 (...)". The outputs table of section 6 lost its Device column (it repeated the name); section 6's intro and the header count the 4 device and 22 drum outputs. The name guard drum_editor::tests::no_hardware_names_in_user_visible_text now scans every line of plugin_usage::TEXT (no TB-/TR-/TR8/TR9/TB3, no "303 1", "303 2", "303 #", "ORB-"). A grep of the other include_str! files (assets/patterns/*.json, assets/ui/layout.tsv, assets/ui/classic.tsv, README.md, param_ids.txt) and of the crates' strings finds no other ORB- (only the guards themselves).
Rebased onto local main a74c22c (main's exports.rs change merged cleanly). Final checks: cargo test --workspace 1164 passed, 0 failed, 5 ignored (corpus_golden 4 passed, EQUAL); clippy -D warnings clean on macOS and xwin x86_64-pc-windows-msvc; nightly fmt --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin has the original's ReWire routing (the user's decisions of 2026-11-25), signed off by the user in REAPER and Ableton.
- Outputs: 22 stereo per-instrument aux buses after the released four (26 aux, 27 VST3 buses): 808 BD, SD, LT, MT, HT, RS, CP, CB, OH, CH, CY and 909 BD, SD, LT, MT, HT, RS, CP, CH, OH, CC, RC, each its instrument's raw voice sum, mono on both sides. One table per machine (tr808/tr909 OUTPUTS and VOICE_OUTPUT) drives the engine taps, the bus names, the state keys and the window.
- Options > Output Routing... (plugin only): 26 boxes in three columns (To Mix-L/R, To 808, To 909), labeled by name, tooltip "#N" the VST3 bus index; fits 536 x 485. Saved in the state as "routing" by stable ids ("mix.303-1", "808.lt", "909.rc"); absent = checked, so released states load with every box checked. Not parameters: param_ids.txt unchanged.
- Semantics (docs/rewire.md 4, docs/plugin.md 13.1): a device unchecked leaves Mix-L/R as ReWire's bypass (0x100136AB/0x100136B1), keeping meter and send, gains frozen (quirk Q-6); a drum instrument unchecked adds into its own block instead of the bus (as ReWire's group entries, 808 0x100280D8.., 909 0x1002B718..), so it is out of the machine's inserts, meter, level, pan, delay, Mix and output; a checked instrument's output is a copy of the same f64 terms, so the bus is bit-identical.
- With every box checked the Mix and the four device outputs are bit-identical; corpus_golden EQUAL; process allocates nothing.
- Also: Help > Plugin Usage lost its ORB- names (task-293's miss) and the name guard now scans that page.
Branch worktree-agent-aa089865e75ae6103, one commit on main a74c22c. Tests: cargo test --workspace 1164 passed, 0 failed, 5 ignored; clippy -D warnings macOS and Windows (xwin); nightly fmt --check.
<!-- SECTION:FINAL_SUMMARY:END -->
