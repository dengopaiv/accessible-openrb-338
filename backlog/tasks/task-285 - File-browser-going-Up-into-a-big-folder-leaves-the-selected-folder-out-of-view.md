---
id: TASK-285
title: >-
  File browser: going Up into a big folder leaves the selected folder out of
  view
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-22T17:01:00Z'
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
- [ ] #1 After Up into a folder with at least 2000 entries, the selected (left) folder's row is fully inside the list's visible area, whether it is near the start, middle or end
- [ ] #2 Keyboard selection moves, type-to-select and Backspace/Alt+Up keep the selected row visible; a click does not scroll
- [ ] #3 A test drives the browser over a temporary folder of many entries through egui frames and checks the selected row's rect is inside the list's visible rect after Up
- [ ] #4 The cause is recorded in the notes
- [ ] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [ ] #6 The user has checked it in the app
<!-- AC:END -->
