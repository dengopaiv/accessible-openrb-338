---
id: TASK-305
title: >-
  Plugin: REAPER test cases for the per-drum outputs and the Output Routing
  state
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T12:00:00Z'
updated_date: '2026-11-25T12:55:00Z'
labels:
  - tests
  - player
milestone: m-7
dependencies:
  - TASK-300
references:
  - crates/orb-plugin/tests/reaper.rs
  - crates/orb-plugin/tests/reaper/cases.rs
  - docs/plugin.md
priority: medium
type: task
ordinal: 276700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** task-300 (2ea3e21) added Options > Output Routing and 22 per-drum stereo outputs (808 BD .. 909 RC, buses 5..26 after the Mix and the four device outputs). Its tests run in the in-process host: tests/outputs.rs, engine frame/tests.rs, and exports.rs for the bus list. It only updated the opt-in REAPER test (tests/reaper.rs and tests/reaper/cases.rs, run with ORB_REAPER, docs/plugin.md 7.1) for the new state. Its projects keep the released layout on purpose: 10 outputs mapped, no drum outputs (cases.rs:40). Nothing checks in REAPER that the drum outputs and the routing state work. The user asked for this on 2026-11-25.

**Add REAPER cases** next to outputs_routed / outputs_unrouted. Generate the projects the same way the existing cases are generated, as fixtures under tests/reaper/:
1. **Drum outputs routed:** a project whose track has all 54 channels, with each of the 22 drum outputs and the 4 device outputs received by its own track (AUXRECV, as the device tracks are now). Compare each output's track with the in-process host's bus for the same song and rate, under the existing tolerance rules (cases.rs: what may differ from bit-identical, e.g. REAPER's denormal flush). Compare the Mix (Master) with outputs_unrouted: routing the drum outputs must not change it.
2. **Routing state saved:** one project per change, or one combined if the comparison stays clear:
   - one device unchecked under Mix-L/R (e.g. the 808): Master equals the in-process host's Mix with that routing, and the device keeps its delay send (docs/plugin.md 13.1, docs/rewire.md section 4);
   - one 808 drum (e.g. BD) and one 909 drum (e.g. CP) unchecked: the drum is gone from its machine's output track and from Master, and its own output track carries it.
   The state is written through the plugin's own state, the "routing" key with stable ids such as "808.bd".
3. Use songs where the checked drums and devices actually sound, and a delay send that is non-zero for case 2's device.

Keep the existing cases' released layout unchanged. Follow docs/plugin.md 7.1 for how the REAPER test runs (the copy that ran, the plugin log, the scratch folders) and document the new cases there.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A REAPER case routes all 22 drum outputs (and the 4 device outputs) to their own tracks; each track matches the in-process host's bus within the existing rules, and Master is unchanged against the unrouted project
- [x] #2 REAPER cases with saved routing changes (a device out of Mix-L/R; an 808 and a 909 drum unchecked) render Master and the output tracks as the in-process host does, the device keeping its delay send
- [x] #3 The existing REAPER cases keep the released layout; docs/plugin.md 7.1 lists the new cases
- [x] #4 The REAPER test passes with ORB_REAPER set on the user's Mac (run by the agent if REAPER is reachable, else by the user); cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. cases.rs: Case gains drums (all 27 outputs, 54 channels, 22 drum tracks + 4 device tracks) and off (routing ids unchecked, saved in the state's routing key); tracks(c) lists the stems. 2. New cases on full_mix Latch: outputs_drums (all routed, all checked), routing_device (808 out of Mix-L/R, 808 send 100), routing_drums (808.bd and 909.cp unchecked). 3. reaper.rs: check() compares every track with the in-process bus (flush allowed), Master with the in-process Mix under the routing; cross-case: outputs_drums Master == outputs_unrouted; device case 808 track == outputs_drums', Master differs, send contributes; drum case BD/CP tracks == outputs_drums', 808/909 tracks and Master differ. 4. Regenerate fixtures, docs/plugin.md 7.1, run with ORB_REAPER, checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-afc1ba6dd40e72cd9 (rebased onto 5b81c7b)

REAPER 7.86 run (ORB_REAPER=/Applications/REAPER.app/Contents/MacOS/REAPER), cargo test -p orb-plugin --test reaper: 4 passed (twice; second run 55 s). New cases outputs_drums, routing_device, routing_drums (full_mix, Latch, 54 channels, 26 stem tracks): every track equals the in-process bus bit for bit (1e-16 flush exemption), Master equals the in-process Mix under the routing. Drums that sound in full_mix: 808 BD, 808 CH, 909 SD, 909 CP, 909 OH. outputs_drums Master == outputs_unrouted Master bit for bit. routing_device (mix.808 off, d0c20 send 100): all 26 tracks equal outputs_drums', Master differs, in-process Mix with send 0 differs (send kept). routing_drums (808.bd, 909.cp off): 808 and 909 tracks and Master differ, the other 24 tracks (incl. BD and CP outputs) equal outputs_drums'. Existing 11 fixtures unchanged (10 channels). cargo test --workspace 1164 passed, 0 failed, 5 ignored; clippy -D warnings clean on macOS and xwin x86_64-pc-windows-msvc; nightly fmt --check clean.

Review round 1: cases.rs fx_block string literal was 103 columns; split with a backslash continuation (content byte-identical: the_fixtures_are_current passes, regenerating the fixtures leaves git status with no fixture change). reaper.rs routing_cases derives Allow from the format as check() does. cargo test -p orb-plugin 172 passed, 0 failed; nightly fmt --check and clippy clean; the REAPER test not rerun (no change to what it renders).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Three REAPER cases added to crates/orb-plugin/tests/reaper (cases.rs, reaper.rs, fixtures outputs_drums, routing_device, routing_drums .vst3.RPP) and documented in docs/plugin.md 7.1. outputs_drums routes all 26 aux outputs to own tracks (54 channels); its Master equals outputs_unrouted. routing_device (808 out of Mix-L/R, send 100) and routing_drums (808 BD, 909 CP unchecked, state key routing) match the in-process host on every track and the Master, and cross-checks against outputs_drums prove the device keeps its send and the drum outputs carry the unchecked drums. REAPER 7.86 run passed; cargo test --workspace 1164 passed; clippy macOS and Windows and nightly fmt clean. Existing fixtures unchanged.
<!-- SECTION:FINAL_SUMMARY:END -->
