---
id: TASK-301
title: 'Plugin: 909 cymbal parameters read 909 CC / 909 RC, like the drum editor'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T10:34:00Z'
updated_date: '2026-11-25T11:59:00Z'
labels:
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/params.rs
  - crates/orb-plugin/tests/params.rs
  - crates/orb-ui/src/drum_editor.rs
  - docs/plugin.md
priority: low
type: enhancement
ordinal: 273700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user decided on 2026-11-25 that the 909's cymbals are called CC and RC everywhere a user reads a short name. The drum editor already uses CC and RC (crates/orb-ui/src/drum_editor.rs:60). task-300's per-instrument outputs are named '909 CC' and '909 RC'. The plugin's parameters still say 'Crash' and 'Ride'.

**Change:** the TR909 display-name table in crates/orb-plugin/src/params.rs, around lines 354-357:
- 'Crash Level' becomes 'CC Level', so the parameter reads '909 CC Level';
- 'Crash Tune' becomes 'CC Tune';
- 'Ride Level' becomes 'RC Level';
- 'Ride Tune' becomes 'RC Tune'.

The parameter string ids must NOT change: tests/param_ids.txt and the_ids_do_not_change stay green, so host automation keeps working. Display names are labels and may change, following task-220 and task-293.

**Where Crash/Ride appear today** (the supervisor's grep, 2026-11-25):
- crates/orb-plugin/src/params.rs:354-357: the four parameter names. CHANGE these.
- crates/orb-ui/src/drum_editor.rs:315-316: the tooltips 'Crash Cymbal' / 'Ride Cymbal'. They give the full name behind CC/RC. KEEP.
- docs/ui_spec.md:2662: describes those tooltips. KEEP.
- docs/midi.md:305-306: 'crash (CC)', 'ride (RC)', a note table explaining the instruments. KEEP.
- crates/rbs/src/lib.rs:157-158: TR909_COLUMNS 'crash', 'ride' are the song file model's field names (JSON keys), not labels. KEEP.
- Comments in crates/orb-engine/src/tr909.rs and crates/orb-plugin/tests/note_clock.rs describe the original. KEEP.

Run your own grep for any other user-visible 'Crash'/'Ride' (the parameter docs in docs/plugin.md, the site, README, tests that assert names) and update tests that assert the old names.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The 909 cymbal parameters display as 909 CC Level, 909 CC Tune, 909 RC Level and 909 RC Tune; parameter ids unchanged (param_ids.txt test green)
- [x] #2 No other user-visible short label says Crash or Ride; the full-name tooltips and the file model's keys stay
- [x] #3 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #4 The user has checked the names in a host
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rename the four TR909 display names in params.rs (ids untouched). 2. Fix the stray /// in routing_window.rs:24. 3. grep other user-visible Crash/Ride (none besides KEEP items found). 4. Run workspace tests, clippy mac+win, nightly fmt, build the bundle.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-25, supervisor: from task-300's review, also fix the stray '///' inside the doc comment at crates/orb-ui/src/routing_window.rs:24 ('and /// its name.'); it renders as literal text in rustdoc.

Branch: worktree-agent-ac8c31e157fa91a82

Renamed the four TR909 names (params.rs); ids untouched. Fixed stray /// in routing_window.rs. Grep found no other user-visible Crash/Ride besides the KEEP items. cargo test --workspace 1164 pass 0 fail; clippy mac and xwin clean; nightly fmt clean. Bundle built. AC 1 and 4 left for the user's host check.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The 909 cymbal parameters now display as 909 CC Level, 909 CC Tune, 909 RC Level and 909 RC Tune (params.rs); string ids unchanged (param_ids.txt, the_ids_do_not_change green). Fixed the stray /// in routing_window.rs. Tooltips, file keys and comments kept. cargo test --workspace 1164 pass; clippy mac and Windows clean; nightly fmt clean. The user checked the names in a host on 2026-11-25 with the bundle built from 9a3a08e. Branch worktree-agent-ac8c31e157fa91a82.
<!-- SECTION:FINAL_SUMMARY:END -->
