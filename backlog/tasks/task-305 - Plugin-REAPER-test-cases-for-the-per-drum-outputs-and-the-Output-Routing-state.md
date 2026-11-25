---
id: TASK-305
title: >-
  Plugin: REAPER test cases for the per-drum outputs and the Output Routing
  state
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-25T12:00:00Z'
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
- [ ] #1 A REAPER case routes all 22 drum outputs (and the 4 device outputs) to their own tracks; each track matches the in-process host's bus within the existing rules, and Master is unchanged against the unrouted project
- [ ] #2 REAPER cases with saved routing changes (a device out of Mix-L/R; an 808 and a 909 drum unchecked) render Master and the output tracks as the in-process host does, the device keeping its delay send
- [ ] #3 The existing REAPER cases keep the released layout; docs/plugin.md 7.1 lists the new cases
- [ ] #4 The REAPER test passes with ORB_REAPER set on the user's Mac (run by the agent if REAPER is reachable, else by the user); cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
<!-- AC:END -->
