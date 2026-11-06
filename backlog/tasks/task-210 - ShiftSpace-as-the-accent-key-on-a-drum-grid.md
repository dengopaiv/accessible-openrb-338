---
id: TASK-210
title: Shift+Space as the accent key on a drum grid
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-06T22:00:00Z'
updated_date: '2026-11-06T22:06:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-191
  - TASK-208
references:
  - crates/orb-ui/src/a11y/grid.rs
  - docs/ui_spec.md
  - crates/orb-ui/tests/a11y_grid.rs
priority: low
type: enhancement
ordinal: 228000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Decision TASK-208 (the user, 2026-11-06: "Add shift+space."). In screen reader mode, Alt+Space on a focused TR-909 drum grid sets an accent as the mouse's Alt does (TASK-191), but on Windows it also opens the window's system menu (winit passes WM_SYSKEYDOWN to DefWindowProc). Add Shift+Space (Shift alone) as the same key as Alt+Space on a grid (crates/orb-ui/src/a11y/grid.rs GridKey::of), keep Alt+Space; Shift with Ctrl or Alt stays not the grid's. The 808 ignores the modifiers, so Shift+Space is its plain Space.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 GridKey::of maps Shift+Space to the Alt modifier; Shift+Ctrl+Space and Shift+Alt+Space are not the grid's (unit test).
- [x] #2 Test: Shift+Space on the 909 and 808 grids gives the same engine events as Alt-clicks on the same steps; the 909 cell then reads accent.
- [x] #3 Help > Keyboard Shortcuts, docs/ui_spec.md 10 and 20.6, accessibility.md and the plan's NVDA checklist item 9 name Shift+Space.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch task-shift-space (in the main checkout, no worktree; done by the supervisor in the foreground at the user's request "Add shift+space.", 2026-11-06). grid.rs GridKey::of: Space with Shift alone (no Ctrl, Alt or Cmd) -> Set(Mods{ctrl: false, alt: true}); Shift with Ctrl or Alt -> None (left to the key map, as before); unit test the_keys_of_the_grid updated (it asserted Shift+Space was None). App::grid_key unchanged: it already sees the key first in App::keyboard. Test a11y_grid::shift_space_is_the_accent_key: on the 909 and the 808, Shift+Space on steps 1, 1, 4 vs Alt-clicks on the same steps (mouse app, mode off): equal events (3 each); the 909 cell reads "Bass drum, step 1, accent"; Shift+Ctrl+Space sets nothing. Runs: nightly fmt --check clean; clippy --release --workspace --all-targets -D warnings clean; cargo test --release -p orb-ui --no-fail-fast 402 passed, 1 failed (crash.rs, the known TASK-200 failure); a11y_grid 10/10.

Review round 1 (agent-reviewer): send back, one test gap: AC #1 names Shift+Alt+Space but the unit test asserted only Shift+Ctrl+Space. Fixed: the_keys_of_the_grid asserts Shift+Alt+Space -> None too. Minor note taken: the integration test's Shift+Ctrl+Space check is that no drum step is set (no Step909 event), not that nothing at all happens (the key falls to the key map as before this task). Runs: grid unit tests and a11y_grid 10/10 pass, fmt --check clean.

Review round 2: merge. Merged by the supervisor on 2026-11-07: fast-forward of main to the branch's one commit, this note amended in. Tests before the merge: orb-ui 402 pass, 1 known failure (crash.rs, TASK-200); after the review fix, grid unit tests and a11y_grid 10/10 pass. Pushed to origin (the fork).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Shift+Space on a drum grid is now Alt+Space: an accent on/off on the TR-909 (the 808's plain Space), without the Windows system menu that Alt+Space also opens; Alt+Space stays (decision TASK-208, option B). grid.rs GridKey::of; Help > Keyboard Shortcuts, docs/ui_spec.md 10 and 20.6, accessibility.md, the plan's checklist item 9 and TASK-197 AC #8 say so. Tests: a11y_grid::shift_space_is_the_accent_key (grid vs Alt-clicks on both machines), the unit test of the keys; orb-ui 402 passed, 1 known failure (crash.rs, TASK-200). Branch task-shift-space, one commit on main d05000d.
<!-- SECTION:FINAL_SUMMARY:END -->
