---
id: TASK-300
title: >-
  Plugin: Options > Output Routing, per-drum-group outputs and taking channels
  out of the mix
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-25T10:19:00Z'
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
**Why:** the user decided on 2026-11-25 to give the plugin the original's ReWire routing: per-drum-group outputs, and channels that can be taken out of the mix. Today the plugin has the main output Mix and four stereo aux outputs 303-1, 303-2, 808 and 909, which are copies (task-256, docs/plugin.md section 13). VST3's activateBus cannot drive this: nice-plug 0.4.2 ignores its state (src/wrapper/vst3/wrapper.rs:347), and hosts do not report whether an output is connected. So the user chooses in a window.

**The window.** It is the plugin's Options > Output Routing..., in the plugin only. The standalone plays a stereo mix and does not show it. The number in parentheses is the output's number: Mix is the main output and the aux outputs are numbered from 1. All boxes are checked by default.

    To Mix-L/R
    [x] (1) 303-1   [x] (2) 303-2   [x] (3) 808   [x] (4) 909
    To 808
    [x] (5) BD  [x] (6) SD  [x] (7) Tom  [x] (8) Per  [x] (9) HH  [x] (10) CY
    To 909
    [x] (11) BD [x] (12) SD [x] (13) Tom [x] (14) Per [x] (15) HH [x] (16) CY

**Semantics.** They follow docs/rewire.md sections 4 and 6, the original's ReWire output, with addresses.
- **A device unchecked under Mix-L/R** (303-1, 303-2, 808 or 909) leaves Mix-L/R. It keeps its delay send (send x level), as ReWire does (rewire.md section 4 summary table). Its own output stays what it is today: the tap after the channel's inserts, before level and pan.
- **A drum group unchecked under 808 or 909** leaves the machine's bus. The group's voices add into their own buffer instead of the bus (the 808 entries 5..10 and 909 entries 12..17 in rewire.md, slot 14 0x100280B0 / 0x1002B6F0). So the group is also out of the machine's inserts, meter, level, pan, delay send, Mix-L/R and the machine's own output (3)/(4). The rest of the bus keeps its voices in their order, so the bus's f32 sum is the same sum without the removed terms. An idle group is exactly 0, with no 1e-6 guard.
- **A group's own output** (5)..(16) always carries the group's raw voice sum, whether checked or not. That is the trigger gain with the level knob, accent and envelope, and nothing after it. When the group is checked, its output is a copy: compute the group sum separately, so that the machine's bus keeps the original's exact accumulation order and the Mix stays bit-identical. Never derive the bus from the group sums.
- **Defaults:** with every box checked, the Mix and the four existing outputs are bit-identical to today. The corpus and the plugin's existing output tests guard this.

**Buses.** Add twelve stereo aux outputs, L = R like the existing four, AFTER the existing four. Hosts store routing by bus index, so a released project keeps 303-1/303-2/808/909 on buses 1..4. Name them in the 303-1 style without ORB- or TR-, for example '808 BD' .. '909 CY'. Show the names to the user before finalizing. The original's group names are BD, SD, Tom, Per, HH, CY. Check nice-plug 0.4.2's caveats with 16 aux buses (docs/plugin.md section 13: process runs only when every output bus has a buffer; the off-by-one on a lower count) in REAPER and Ableton.

**State.** The sixteen checkboxes are saved in the plugin's state. They are not parameters, so the parameter ids and param_ids.txt stay unchanged. A state saved by the released plugin (v2026.1118.0919) loads with every box checked. process allocates nothing: no_alloc.rs stays green.

**Docs:** docs/plugin.md section 13 (routing, buses, semantics with rewire.md references) and the plugin's menu in docs/ui_spec.md.

**Tests:**
- The defaults are bit-identical to today.
- Each device unchecked: the Mix equals a reference made from the channel's level path removed but its delay send kept.
- Each group unchecked: it is gone from the bus, the machine's output, the meter and the delay, and its output carries the group.
- A group checked: its output equals that group's sum while the Mix is unchanged.
- State round trip, and loading a released-plugin state.
- Bus count, order and names.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Options > Output Routing in the plugin shows the 16 checkboxes in the three sections with their output numbers; they are saved in the plugin state, and a state from the released plugin loads with all checked
- [ ] #2 Twelve stereo per-group aux outputs (5)..(16) follow the existing four; each carries its group's raw voice sum whether checked or not
- [ ] #3 Unchecking a device takes it out of Mix-L/R and keeps its delay send; unchecking a drum group takes it out of its machine's bus (inserts, meter, level, pan, delay, Mix-L/R and the machine's output), as docs/rewire.md describes
- [ ] #4 With every box checked, the Mix and the four existing outputs are bit-identical to before (corpus and plugin tests); parameter ids are unchanged; process allocates nothing
- [ ] #5 docs/plugin.md and docs/ui_spec.md describe the routing; cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #6 The user has checked the window and the routing in REAPER and Ableton, including a project saved with the released plugin
<!-- AC:END -->
