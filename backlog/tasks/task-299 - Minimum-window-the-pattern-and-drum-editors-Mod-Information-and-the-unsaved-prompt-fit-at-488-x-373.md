---
id: TASK-299
title: >-
  Minimum window: the pattern and drum editors, Mod Information and the unsaved
  prompt fit at 488 x 373
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-25T08:34:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/layout.rs
  - crates/orb-ui/tests/min_window.rs
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/src/drum_editor.rs
  - crates/orb-ui/src/editor_frame.rs
  - crates/orb-ui/src/mod_info.rs
  - crates/orb-ui/src/menu.rs
  - docs/ui_spec.md
priority: medium
type: bug
ordinal: 270750
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked (2026-11-25, task-297) that the main window's minimum size be large enough for every non-resizable window to fit. Task-297 raised the minimum to 488 x 373 points (MIN_WINDOW_W/H in crates/orb-ui/src/layout.rs, also the plugin editor's minimum) and tests the file dialogs, About, Audio Output and the error box in crates/orb-ui/tests/min_window.rs with the 12-point DIALOG_MARGIN. Its review found four non-resizable windows that were never measured:
1. **Pattern editor** (per 303): crates/orb-ui/src/pattern_editor.rs around line 617, default guess 480 x 340, through crates/orb-ui/src/editor_frame.rs:133-143 (resizable(false), no max_size). The window size is stored and its columns widen with the window (docs/ui_spec.md around line 2478).
2. **Drum editor** (per machine): crates/orb-ui/src/drum_editor.rs around line 840, guess 520 x 360 (385 with an entry), the same editor_frame path. That is wider than 488 - 24.
3. **Mod Information**: crates/orb-ui/src/mod_info.rs:314-318, resizable(false). At 488 x 373 its splash shrinks to 64 points and the window is capped about 472 x 357, so it fits without the margin. docs/ui_spec.md section 17's 'Resizable windows' bullet wrongly lists it as resizable.
4. **Unsaved-changes prompt**: crates/orb-ui/src/menu.rs:1469-1476, a Modal with set_width(320); height content-driven.

Measure each at its worst case (stored or default size, a long mod text, a long file name in the prompt). Then either raise the minimum (keep it derived from the measured sizes, as min_window.rs does) or bound the window to the main window (max size, scrolling content) where raising the minimum would be unreasonable. Recommend which in the notes before coding if raising goes above about 520 x 400. The plugin editor uses the same constants.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each of the four windows is measured at 488 x 373 (or the new minimum) at its worst case, and the numbers are in the notes
- [ ] #2 Each fits inside the main window with the 12-point margin at the minimum, either by a raised minimum or by bounding the window; min_window.rs tests each
- [ ] #3 docs/ui_spec.md states the minimum and lists Mod Information correctly (not resizable)
- [ ] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #5 The user has checked the four windows at the minimum window in the app
<!-- AC:END -->
