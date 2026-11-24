---
id: TASK-285
title: >-
  File browser: going Up into a big folder leaves the selected folder out of
  view
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-22T17:01:00Z'
updated_date: '2026-11-24T08:20:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/browser.rs
  - docs/ui_spec.md
priority: medium
type: bug
ordinal: 262700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user reported it on 2026-11-22. In the file browser (Open, Save, Export, Choose Mod), going Up selects the folder that was left (`Browser::up`, crates/orb-ui/src/browser.rs:369). When the parent folder has many entries, that selected row is not scrolled into view: the list shows some other part of the folder and the user has to look for the selection.

**Where:** `up()` calls `go(parent)`, which sets `self.scroll = 0.0`, then selects the left folder and sets `scroll_to_selected`. `draw_list` (around line 856) computes an offset from `self.scroll` and `self.view_h` and passes it to `ScrollArea::vertical_scroll_offset` before `show_rows`.

**Leads, not verified:**
- egui 0.37 may clamp the requested offset to the content size it remembered from the previous frame. That was the smaller child folder, so the offset into the big parent gets cut down. A fix could repeat the request until the area's content height covers it, or scroll the selected row into view from inside `show_rows` (e.g. `ui.scroll_to_rect` on the selected row's rect).
- `self.view_h` may be stale or 0 on the first frame.
Find the real cause and record it in the notes.

**Expected:** after Up, the selected folder's row is fully visible, also in a folder of thousands of entries and when the selection is near the end. The same holds for the other ways the selection moves without a click (keys, type-to-select, Backspace/Alt+Up). A click keeps not moving the view.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 After Up into a folder with at least 2000 entries, the selected (left) folder's row is fully inside the list's visible area, whether it is near the start, middle or end
- [x] #2 Keyboard selection moves, type-to-select and Backspace/Alt+Up keep the selected row visible; a click does not scroll
- [x] #3 A test drives the browser over a temporary folder of many entries through egui frames and checks the selected row's rect is inside the list's visible rect after Up
- [x] #4 The cause is recorded in the notes
- [x] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [x] #6 The user has checked it in the app
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Failing tests first (crates/orb-ui/tests/browser_scroll.rs: 2400 subfolders, Up/keys/type-to-select/click, row rect inside the list rect). 2. Find the cause in egui 0.37 ScrollArea::show_rows vs draw_list. 3. Fix, run workspace tests, clippy, fmt, build the dev app.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a3ae082ada438aeca

CAUSE (two bugs, both reproduced by failing tests before the fix). (1) draw_list computed a row's top as i*ROW_H (20), but ScrollArea::show_rows and the vertical layout use the pitch ROW_H + item_spacing.y (20+3=23): show_rows derives the visible rows from viewport.min.y / pitch and each allocate_exact_size adds the spacing. So the requested offset was short by 3*i points, which for row 2000 is 6000 points (about 260 rows): the selected row was far out of view, and more so deeper in the list; the list-top/bottom check for 'already visible' was also wrong. Same pitch error in the PageUp/PageDown step. (2) Alt+Up never reached Browser::up: egui consume_key matches logically (an extra Alt is ignored), and the plain ArrowUp was consumed first, so Alt+Up moved the selection up one row instead. The two leads of the task were checked: egui does not clamp the requested offset to the previous frame's content size (scroll_area.rs:822 sets state.offset from the request, the clamp at :1575 uses this frame's max_offset), and view_h is only stale on the very first frame, where nothing is selected yet. Fix: row_pitch field (ROW_H + ui.spacing().item_spacing.y) used for the offset and the page step; Alt+Up handled before plain Up. Added a 'list' hit (the list's visible rect) for tests. Before: End/Home/Up tests failed ('row d2399 is not drawn', Alt+Up selection None). After: 3 new tests pass. cargo test --workspace 1069 passed 0 failed; clippy -D warnings clean; nightly fmt --check clean.

Review round 1 (user, after trying the app): going to the parent now centers the folder that was left (Browser::up sets center_selected; draw_list requests offset = row center - view_h/2, clamped to 0..max so the row stays fully visible near the start and end). Other selection moves keep scrolling only as much as needed; a click does not scroll. type-to-select is left as is (not centered). Test up_into_a_big_folder... now also checks the row center is within one pitch (23) of the list center for k in 1000..=1400 and, near the ends, that the row is on the clamped side. docs/ui_spec.md section 17 updated. cargo test --workspace 1069 passed, clippy and nightly fmt clean, dev app rebuilt.

User sign-off 2026-11-24: looks good (centering on Up only; type-to-select scrolls as little as needed).

Review round 2 (reviewer verdict: merge, small points): ui_spec keys paragraph reflowed to 100 columns; the Up button is said to center like Backspace; Option+Up named for macOS in ui_spec, the browser's header comment and Up-button hover text, and the Keyboard Shortcuts window (key_help.rs, by is_mac); up() doc comment says it centers; the initial row_pitch carries a comment (value before the first frame). Alt+Shift+Up: chose to keep it going to the parent and document it ('a Shift held with it is ignored' in ui_spec), since egui's logical match ignores extra Shift and Alt+Up is rarely mixed with Shift. New test plain_up_arrow_moves_the_selection_and_does_not_go_to_the_parent. cargo test --workspace 1070 passed, clippy and nightly fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed on branch worktree-agent-a3ae082ada438aeca. Two causes: (1) draw_list took a row's top as i*20 but ScrollArea::show_rows and the layout use row height plus item spacing (23), so the requested offset fell short by 3 points per row and the selected row was far out of view in a long list (also the PageUp/PageDown step); now row_pitch is read from the spacing. (2) Alt+Up never reached Browser::up because egui consume_key ignores an extra Alt and the plain Up took it; Alt+Up is now handled first. After the user's try-out, going Up centers the folder left (clamped at the list ends so the row stays fully visible); other moves scroll just enough, a click does not scroll. Tests: crates/orb-ui/tests/browser_scroll.rs (2400 folders; Up, keys, type-to-select, Backspace, click, centering); cargo test --workspace 1069 passed, clippy -D warnings and nightly fmt clean. docs/ui_spec.md section 17 updated.
<!-- SECTION:FINAL_SUMMARY:END -->
