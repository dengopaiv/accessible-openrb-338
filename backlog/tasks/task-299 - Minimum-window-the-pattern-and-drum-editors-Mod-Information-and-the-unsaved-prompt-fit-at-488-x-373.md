---
id: TASK-299
title: >-
  Minimum window: the pattern and drum editors, Mod Information and the unsaved
  prompt fit at 488 x 373
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T08:34:00Z'
updated_date: '2026-11-25T09:32:00Z'
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
- [x] #1 Each of the four windows is measured at 488 x 373 (or the new minimum) at its worst case, and the numbers are in the notes
- [x] #2 Each fits inside the main window with the 12-point margin at the minimum, either by a raised minimum or by bounding the window; min_window.rs tests each
- [x] #3 docs/ui_spec.md states the minimum and lists Mod Information correctly (not resizable)
- [x] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #5 The user has checked the four windows at the minimum window in the app
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Decision (user, 2026-11-25): raise the minimum to fit the editors (536 x 485), editors unchanged. 1. layout.rs MIN_WINDOW_W/H 536 x 485 (widest 808/909 editors 512 + 24, highest 303 editor 461 + 24). 2. min_window.rs: add 303/808/909 editors, Mod Information (long missing-mod notice), unsaved prompt (App::prompt_rect); equality test excludes the file dialogs, which follow the window. 3. Mod Information max_size margin 12. 4. ui_spec, plugin.md, plugin tests/comments. 5. fmt, clippy macOS+xwin, cargo test --workspace.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ae405f8bfb1c9e5ce

Measured (headless, App::with_link, at 488 x 373 and at 1400 x 1000, default scale; same sizes at both, the editors are content-sized and do not follow the window; only the window place is stored in settings, not a size): pattern editor 303 464.8 x 461; 808 editor 512 x 366; 909 editor 512 x 393; Mod Information 400 x 313 at the minimum (fits 464 x 349; its max_size is screen-16, not -24, so its margin is 8, to fix). Unsaved prompt not measured: Modal width 320 + 24 margin = 344, height about 90 (fits). With the 12-point margin the editors need 536 x 485 (pattern 303: 489 x 485; 909: 536 x 417). That is above the 520 x 400 limit, so stopped before coding; recommendation sent to the supervisor.

Decision 2026-11-25 (user): raise the minimum to 536 x 485, editors unchanged. At the new minimum measured: file dialogs 512 x 382 (Open/Choose) / 405 (Save/Export), About 464 x 254, Audio 375 x 210, error box 406 x 388, 303 editor 464.8 x 461, 808 512 x 366, 909 512 x 393 (with entry), Mod Information 400 x 425 (long missing-mod notice), unsaved prompt 346 x 175 (long song name). All inside 12-point margin.

Checks: cargo test --workspace 518 pass 0 fail (the_corner_drag_resizes_steadily_through_the_host's shrink phase shortened, it went below the new minimum); clippy macOS and x86_64-pc-windows-msvc -D warnings clean; nightly fmt clean. Dev app built (target/OpenRB-338.app). AC 5 (user check) open; task left To Do.

Correction: cargo test --workspace on 3e33b50 is 1151 passed, 0 failed (the 518 above was a miscount of a partial tally). User check 2026-11-25: accepted, in the dev app and the plugin bundle built from 3e33b50, plugin installed in REAPER and Ableton. Observation (user, not worked on, no task): resizing with the corner handle behaves differently in REAPER and Ableton.

Review round 1: commit subject shortened to 62 characters; the corner-drag test (orb-plugin tests/editor.rs) got a comment on why its back steps are shorter (the old drag went below the 536 x 485 minimum); the commit body says so and gives the error box 406 x 388 (was 276). Correction to the earlier Correction line: the sha 3e33b50 it cites was amended away; the final sha is the branch's one commit. cargo test -p orb-ui -p orb-plugin 650 pass 0 fail; nightly fmt --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: the smallest main window is 536 x 485 points (layout::MIN_WINDOW_W/H, was 488 x 373; shared by the standalone viewport and the plugin editor), the user's decision of 2026-11-25 to raise the minimum and leave the editors unchanged (derived from the widest windows, the 808 and 909 editors at 512, and the highest, the 303 editor at 461, plus 2 x 12-point DIALOG_MARGIN). Measured at the minimum, all inside the margin: 303 editor 464.8 x 461, 808 editor 512 x 366, 909 editor 512 x 393 (with entry), Mod Information 400 x 425 (long missing-mod notice), unsaved prompt 346 x 175 (long song name), file dialogs 512 x 382 / 405, About 464 x 254, Audio Output 375 x 210. Mod Information's max size now keeps 12 points (was 8). tests/min_window.rs tests each window and that the minimum equals what the editors need; App::prompt_rect added for the prompt. docs/ui_spec.md section 17 and docs/plugin.md updated; Mod Information is listed as not resizable. The editors store only their place, no size, so no stored size can exceed the bounds. Tests: cargo test --workspace 1151 passed, 0 failed; clippy -D warnings clean on macOS and x86_64-pc-windows-msvc; nightly fmt clean. User check: accepted 2026-11-25 in the dev app and the plugin bundle from 3e33b50, plugin installed in REAPER and Ableton. Observation from the user, not worked on: resizing with the corner handle behaves differently in REAPER and Ableton.
<!-- SECTION:FINAL_SUMMARY:END -->
