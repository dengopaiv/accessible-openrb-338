---
id: TASK-256
title: >-
  Plugin: per-device output buses (303-1, 303-2, 808, 909), tapped after each
  channel's own mix and effects
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-20T10:13:00Z'
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
Requested by the user on 2026-11-20 (item 7 "multi-out" of task-196's deferred plugin features, deferred until after the first plugin release, which has shipped). Today the VST3/CLAP plugin has one stereo output, the final mix (crates/orb-plugin/src/lib.rs, AUDIO_IO_LAYOUTS: main_output_channels 2). The user wants each device on its own output so it can be recorded and processed separately in the DAW.

What to build:
- Four extra stereo output buses (aux outputs, VST3 and CLAP), named "303-1", "303-2", "808", "909", in addition to the main "Mix" bus.
- Tap point (the user's decision, 2026-11-20): each bus carries its mixer channel's signal after that channel's own processing — its insert effects where the channel routes them (DIST, PCF, COMP, per `Routing` in crates/orb-engine/src/frame/mod.rs), then its level and pan (stereo) — just before the channels are summed into the master. So the delay (a send whose return goes only into the master) and the master bus processing (master compressor, master level) are not on the per-device buses.
- The main Mix output is unchanged and stays bit-exact, whether or not the extra buses are connected or used: the taps only copy the channel signal; no device leaves the main mix, nothing in the mixing order, gains or rounding changes. The corpus (`cargo test -p orb-engine --test corpus_golden`, `orb compare --manifest ...`) must stay EQUAL.
- Where the original computes these values (the channel loop after the inserts and the pan gains, before the sum into the master bus) is in the engine already; document the tap with addresses in docs/ (plugin.md, and the engine doc where the mixer is described). Any f32 rounding of the tapped values follows where the original stores floats; if the per-channel stereo signal does not exist as stored values in the original (e.g. it is accumulated directly into the master), define the tap as the exact per-channel contribution that is added, and say so.
- The standalone app and the CLI are not changed (optional: none).
- The plugin's latency, sample-accurate timing and the 64-frame block alignment are unchanged; the extra buses add no allocation in the audio thread.
- Host behavior: buses not connected by the host cost little (skip the copy when the host gives no buffer); check REAPER (output pins / multichannel routing) and Ableton (Audio From the plugin's track, its outputs) with the user; document how to route them in docs/plugin_usage.md (and Help > Plugin Usage, which shows that page).

Not in scope: dry (pre-mixer) taps, per-voice drum groups, audio inputs into the effects (discussed 2026-11-20, not requested).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The plugin exposes, besides the main stereo Mix output, four stereo aux outputs named 303-1, 303-2, 808 and 909 in VST3 and CLAP; scripts/validate-plugin.sh passes
- [ ] #2 Each aux output is its mixer channel after that channel's inserts, level and pan, before the master sum (no delay return, no master processing); a test checks a device alone with an insert on reproduces its contribution to the main mix
- [ ] #3 The main Mix output is bit-identical with and without the aux outputs connected/processed, checked by a test; the reference corpus stays EQUAL
- [ ] #4 No allocation in the audio thread for the aux outputs; unconnected buses are skipped
- [ ] #5 docs/plugin.md documents the buses and the tap with addresses; docs/plugin_usage.md explains routing in REAPER and Ableton; the user checked both hosts
<!-- AC:END -->
