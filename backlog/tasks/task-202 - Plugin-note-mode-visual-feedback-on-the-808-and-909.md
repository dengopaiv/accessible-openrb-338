---
id: TASK-202
title: 'Plugin: note-mode visual feedback on the 808 and 909'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T07:47:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-200
  - TASK-201
references:
  - backlog task-185 (final summary)
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 215000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
In note mode (task-200) the panel shows which drum voices are sounding, as task-185's final summary decides. It is separate from the editor's selection, which notes never change. The user reviews the visual result before merge.

- **808:** a green outline of the same shape as the orange selection outline (crates/orb-ui/src/app.rs, the selection marks of task-179), on the chip of the voice played: the lower legend (LT, RS, CP, ...) or the upper chip (LC, CL, MA, ...) by the voice the note named, whatever the slot's switch says. On an instrument that is also selected, only the green shows. A hit with AC also outlines AC green.
- **909:** the instrument's LED glows green instead of red; the AC LED glows green for a hit with AC.
- **When lit:** while the voice sounds, from task-200's per-voice sounding mask (active and gain above about -60 dB), with a minimum on-time of about 100 ms (tune by eye). Several voices can be lit at once; a CH choke darkens OH; a retrigger restarts the light; the BD is lit while either of its two alternating voices sounds. AC is lit while any voice whose last hit had AC is lit.
- **303s:** nothing.
- **Color-blind users:** green must stay distinguishable from the red/orange selection by brightness or glow, not by hue alone.
- Pattern mode: no change (its running lights as now).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Green outlines and LEDs appear for sounding voices in note mode as described, including shared-slot chips by the played voice, AC, choke and minimum on-time
- [ ] #2 The selection marks are unchanged by notes; with both on one instrument only green shows
- [ ] #3 Green vs red/orange is distinguishable in a deuteranopia simulation of a screenshot
- [ ] #4 The user reviewed the feedback (screenshots or a build) and approved it before merge
- [ ] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
