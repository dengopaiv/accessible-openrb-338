---
id: TASK-238
title: Screen reader mode with the classic skin
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:08:00Z'
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
ordinal: 255000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream's View > Classic (task-267) draws the panel from the kit's bitmaps (skin/classic.rs, assets/ui/classic.tsv) in the classic skin's own units and layout. The merge (task-233) moved the fork's screen reader hooks into upstream's controller.rs, so they run on whatever layout the current skin gives: the focus order is made from the shown skin's layout (App::sync_nav), the hit areas keep the ids the tree knows (controller.rs hit_id, Placed::first). Some of the fork's sizes are the modern panel's, not the skin's: drive::SELECTOR_SIZE (the pattern selector's rect, in modern panel units) and the drum grid's cells (App::drum_grid, from the layout's 'steps' controls). Nothing yet checks the tree in the classic skin.

Context: the merge of upstream f83c54f (task-233; the user's decision task-232, question 2: take everything, each new window gets its own m-7 task) brought this window in as upstream built it, for the mouse. The fork's screen reader work is docs/accessibility.md (the analysis, egui's AccessKit API, how to measure: tools/uia_tree.ps1) and docs/accessibility_plan.md (the plan and its ground rules: with screen reader mode off the app behaves exactly as upstream). Existing patterns to follow: crates/orb-ui/src/a11y/ (names.rs, announce.rs with App::set_status and App::announce, window.rs with win::line for focusable text lines, nav.rs), the Win enum and App::window_focus in app.rs (a window takes the focus as it opens in screen reader mode and gives it back when it closes), and the a11y_* tests in crates/orb-ui/tests (AppTree drives the app and reads the AccessKit tree). By-ear NVDA checks are not criteria here: they go to the user's pass, TASK-197.

What to do: run the a11y tests' checks (names, the focus order, the section groups, the drive keys, the drum grids, the 303 step editor) with the classic skin too (App::set_theme(Theme::Classic) in the harness), fix what differs (the selector's rect from the skin's layout, missing or extra controls of the classic layout in the names table), and check that a crossfade between the skins does not move the focus or announce anything.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The a11y tests that check the tree run against both the modern and the classic skin and pass
- [ ] #2 Switching View > Classic / Modern in screen reader mode keeps the focus on the same control and announces nothing but the menu's own item
- [ ] #3 No size in a11y/ assumes the modern panel's units where the skin's layout gives the place
<!-- AC:END -->
