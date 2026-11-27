---
id: TASK-238
title: Screen reader mode with the classic skin
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:08:00Z'
updated_date: '2026-11-27T23:21:00Z'
labels:
  - ui
  - tests
milestone: m-7
dependencies:
  - TASK-233
references:
  - crates/orb-ui/src/controller.rs
  - crates/orb-ui/src/skin/classic.rs
  - crates/orb-ui/src/a11y/nav.rs
  - crates/orb-ui/src/a11y/drive.rs
priority: medium
type: task
ordinal: 254000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream's View > Classic (task-267) draws the panel from the kit's bitmaps (skin/classic.rs, assets/ui/classic.tsv) in the classic skin's own units and layout. The merge (task-233) moved the fork's screen reader hooks into upstream's controller.rs, so they run on whatever layout the current skin gives: the focus order is made from the shown skin's layout (App::sync_nav), the hit areas keep the ids the tree knows (controller.rs hit_id, Placed::first). Some of the fork's sizes are the modern panel's, not the skin's: drive::SELECTOR_SIZE (the pattern selector's rect, in modern panel units) and the drum grid's cells (App::drum_grid, from the layout's 'steps' controls). Nothing yet checks the tree in the classic skin.

Context: the merge of upstream f83c54f (task-233; the user's decision task-232, question 2: take everything, each new window gets its own m-7 task) brought this window in as upstream built it, for the mouse. The fork's screen reader work is docs/accessibility.md (the analysis, egui's AccessKit API, how to measure: tools/uia_tree.ps1) and docs/accessibility_plan.md (the plan and its ground rules: with screen reader mode off the app behaves exactly as upstream). Existing patterns to follow: crates/orb-ui/src/a11y/ (names.rs, announce.rs with App::set_status and App::announce, window.rs with win::line for focusable text lines, nav.rs), the Win enum and App::window_focus in app.rs (a window takes the focus as it opens in screen reader mode and gives it back when it closes), and the a11y_* tests in crates/orb-ui/tests (AppTree drives the app and reads the AccessKit tree). By-ear NVDA checks are not criteria here: they go to the user's pass, TASK-197.

What to do: run the a11y tests' checks (names, the focus order, the section groups, the drive keys, the drum grids, the 303 step editor) with the classic skin too (App::set_theme(Theme::Classic) in the harness), fix what differs (the selector's rect from the skin's layout, missing or extra controls of the classic layout in the names table), and check that a crossfade between the skins does not move the focus or announce anything.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The a11y tests that check the tree run against both the modern and the classic skin and pass
- [x] #2 Switching View > Classic / Modern in screen reader mode keeps the focus on the same control and announces nothing but the menu's own item
- [x] #3 No size in a11y/ assumes the modern panel's units where the skin's layout gives the place
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. drive::selector_rect from the skin's controls replaces SELECTOR_SIZE (modern units); App::pattern_selector and controller::control take the layout's controls. 2. Classic layout has an extra knob tr808.selector (the 808's instrument knob): not a focus stop, hidden node. 3. New tests/a11y_classic.rs runs names, focus order and groups, selector, drum grids, 303 editor and crossfade over both skins. 4. TASK-244 gets the by-ear criterion. 5. cargo test --release --workspace, clippy, fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ae7c69f3f6c9efb56

Evidence: classic layout differs from the modern one in two ways. (1) The pattern selector's rect: drive::SELECTOR_SIZE (61.4x56.6 from pattern1) is the modern keycaps' size; the classic buttons are 17x20 on another pitch. Now drive::selector_rect(ctls, section) is the union of the section's grid buttons in the shown skin's layout (App::pattern_selector gets the layout's controls). Drum grid cells already came from the layout's steps controls (checked: bounds equal the union in both skins). (2) The classic layout has an extra knob tr808.selector (the 808's instrument knob, n=12) that the modern one lacks; it was a nameless focus stop (195 stops vs 194, no usable node). Now nav::focusable skips it and a11y::describe hides its node; the mouse still works and the grid's Up/Down choose the instrument. Order inside a section differs by skin (each layout file's order); the stops per section are the same. Crossfade: focus and announcer serial unchanged on every frame, both directions, no engine event. New tests/a11y_classic.rs, 6 tests over both skins. Tests: cargo test --release --workspace 1171 passed 0 failed 7 ignored; clippy -D warnings clean; nightly fmt clean. By-ear check added as AC 4 of TASK-244.

Review round 1 (send back), fixed in the amended commit: (1) the hiding of the classic 808 instrument knob moved from a11y::describe to controller::knob and is gated on screen reader mode; with the mode off the node is described as before. (2) The skins-agree test now compares all interactive controls without the focusable filter, allows tr808.selector as the one classic-only control, asserts it hidden with the mode on and not hidden with the mode off. (3) The selector test checks the literal classic rect (17, 69 + 115 r, 68 x 69 from classic.tsv). By-ear checks go to TASK-244, not TASK-197, because TASK-197 is Done (the supervisor's instruction; the task description still names TASK-197). Tests: cargo test --release -p orb-ui all pass (a11y_classic 6), clippy -p orb-ui -D warnings and nightly fmt --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Screen reader mode now checked and fixed for the classic skin. New tests/a11y_classic.rs runs names/nodes, focus order and groups, pattern selector, drum grids, 303 step editor and the crossfade over both skins (6 tests). Fixes: drive::selector_rect from the skin's layout replaces the modern-only SELECTOR_SIZE; the classic-only 808 instrument knob is no longer a focus stop and its node is hidden. Crossfade keeps focus and announces nothing (machine-verified). cargo test --release --workspace 1171 passed; clippy and nightly fmt clean. The by-ear check is AC 4 of TASK-244. Branch worktree-agent-ae7c69f3f6c9efb56.
<!-- SECTION:FINAL_SUMMARY:END -->
