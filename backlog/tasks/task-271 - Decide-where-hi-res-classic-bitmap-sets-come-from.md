---
id: TASK-271
title: 'Decide: where hi-res classic bitmap sets come from'
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-21T12:08:00Z'
updated_date: '2026-11-21T12:53:00Z'
labels:
  - decision
  - ui
milestone: m-8
dependencies: []
references:
  - docs/ui_spec.md
  - crates/orb-ui/src/skin/bitmaps.rs
priority: low
type: spike
ordinal: 258000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** Task-267 made the classic skin (View > Classic) scale-ready, as the user asked on 2026-11-21: a classic set in the same format as a kit's 58 panel bitmaps (12015.jpg .. 12571.jpg, docs/rbm.md 13.1) but N times larger is composed at 624N x 510N pixels (bitmaps::find_scale: N is the background 12057's width over 592 and height over 50, rounded, when the two agree, else 1; every bitmap's sides must be multiples of N). The criterion left open where such a set comes from; today the classic skin draws only a kit's own 58 files (a mod's, or the built-in Standard kit's), so a hi-res set can only be a mod whose panel JPEGs are larger, which the original ReBirth would draw wrongly.

**Options.**
1. A mod resource: hi-res files inside the .rbm under other names (for example 2x/12057.jpg), read by orb only; the 1x files stay for the original. Needs a packer change (orb mod pack) and a naming rule.
2. A sidecar folder next to the mod (MyMod.rbm plus MyMod.classic/ with the 58 files), or in the mods folder; easy to make by hand, no repacking; can get separated from the mod.
3. A theme file: a separate package (zip or folder) chosen in the View menu, independent of the kit; a classic look for every kit, but then a mod's own panel is not shown.

**Recommendation.** Option 2 to start (no format change, works for the built-in kit with a folder of its own), with option 1 later if sets get shared.

Context: docs/ui_spec.md section 21 (Hi-res sets), task-267's notes, task-268 (the transition pixelates to the set's 624N density).
<!-- SECTION:DESCRIPTION:END -->
