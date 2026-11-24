---
id: TASK-186
title: After File > New the panel's controls stay at zero until the mouse moves
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-14T16:39:00Z'
updated_date: '2026-11-24T12:10:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
priority: medium
type: bug
ordinal: 202000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-14 in the standalone app (OpenRB-338): after File > New, the panel's controls (knobs, faders and the other positioned controls) are drawn at zero, not at the new song's values. They jump to the right positions only when the mouse moves over the window. So the new song's values do reach the panel, but the window is not redrawn when they arrive. Someone who chooses New and does not touch the mouse sees a panel that does not match what plays.

Check whether Open, Demo Song and a mod change show the same stale panel, since they also replace the song.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 After File > New the panel shows the new song's control positions without any mouse or keyboard input, checked by hand and recorded in the notes
- [x] #2 The same holds for every other path that replaces the song (Open, Demo Song, a mod change), each checked and recorded in the notes
- [x] #3 A test in crates/orb-ui fails before the fix and passes after it, checking that replacing the song makes the panel redraw with the new values with no input event
- [x] #4 cargo test, clippy and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reproduce headless: a Window driver in crates/orb-ui/tests/repaint.rs (menu bar, sounding stream, frames only while one asks for a repaint, File > New clicked through the menu). 2. Find the cause. 3. Fix in Link. 4. Check Open, Demo Song, mod change. 5. Tests, clippy, fmt, dev app.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ab66c196af53796c4

Cause: App::show draws the panel first and takes the engine's values after (Link::sync -> follow). File > New sets Panel::new() (zeros/defaults) and the engine's values (cutoff, tempo, mixer...) arrive in the first snapshot after the commands; that frame has drawn the old panel, and Link::moving then went false (frames > mark, nothing playing), so no repaint was asked until input. Reproduced headless by a Window driver (menu click File > New, frames only while one asks for a repaint): painted values [139000,...old] vs panel [140000,157,709,...]; test failed before the fix.

Fix: Link keeps taken/fresh; follow sets fresh on the first values taken after a mark; moving() returns true once for it, so one more frame draws them. Open and Demo Song: not affected (Link::load sets the panel from the song before the frame); their tests pass with the fix disabled, kept as guards. Mod change: panel values do not change; guard test passes. Plugin File > New: same App::fresh, but the plugin repaints at embed POLL always (embed.rs:540); not driven by hand.

Tests: crates/orb-ui/tests/repaint.rs file_new_shows_the_new_songs_values_without_input (fails before, passes after), open_/demo_song_/a_mod_change_ guards (demo and mod need ORB_CD_ISO). cargo test --workspace 1074 pass with ORB_CD_ISO set; clippy -D warnings, nightly fmt --check clean. AC 1 (by hand in the app) is left for the user.

Rebased onto local main 9b019ef (task-287/288 in): New test still fails without the fix (painted [140000,750,500,...] vs panel [140000,787,157,...]) and passes with it; cargo test --workspace 1092 pass (ORB_CD_ISO set), clippy and fmt clean.

User signed off on the hand check in the app (commit b8339f3) on 2026-11-24: good.

Review round 1 (send back, docs and message only): added the new repaint condition to docs/ui_spec.md section 16 and the Link::moving doc comment; commit body Testing now says 1092. Correction: the plugin's File > New goes through App::fresh (crates/orb-ui/src/menu.rs:675), not a Link method. The b8339f3 the user signed off has the same code as the final commit: the amends changed only the task file, docs, a doc comment and the commit message.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Cause: App::show draws the panel before Link::follow takes the engine's values, and Link::moving went false once the snapshot passed the mark, so after File > New the panel (Panel::new defaults) stayed drawn until input. Fix: Link marks the first values taken after a send (fresh) and moving() asks for one more frame (crates/orb-ui/src/link.rs). Paths: File > New was affected and is fixed; Open, Demo Song and a mod change were not (the panel is set at load or values do not change), guarded by tests; the plugin repaints by polling. Tests: crates/orb-ui/tests/repaint.rs (the New test, driven through the menu, fails without the fix); cargo test --workspace 1092 pass, clippy and nightly fmt clean. Branch worktree-agent-ab66c196af53796c4.
<!-- SECTION:FINAL_SUMMARY:END -->
