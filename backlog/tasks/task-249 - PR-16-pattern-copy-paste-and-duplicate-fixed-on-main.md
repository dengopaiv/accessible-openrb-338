---
id: TASK-249
title: 'PR 16: pattern copy, paste and duplicate, fixed on main'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-18T14:13:00Z'
updated_date: '2026-11-18T14:27:00Z'
labels:
  - ui
milestone: m-3
dependencies: []
references:
  - https://github.com/pingw33n/OpenRB-338/pull/16
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/tests/menu.rs
  - docs/ui_spec.md
  - task-235
priority: medium
type: feature
ordinal: 190800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
GitHub PR 16 by an outside contributor (jochenrieger), "copy, paste and duplicate a pattern to another slot": Edit menu (Cut/Copy/Paste Pattern, Duplicate Ctrl/Cmd+D, one-level Undo Ctrl/Cmd+Z) in the standalone, writing whole patterns as msg 41 sub 44/61/78 through `Link::set_pattern`. Read the PR description with `gh pr view 16`.

The supervisor rebased it onto main locally as branch `pr-16` (one commit, 0a672f9) in worktree `.claude/worktrees/pr-16`; the only conflict was `focus_click` in crates/orb-ui/src/app.rs, which main's task-235 had already fixed the same way. The user (2026-11-18) wants the review's fixes made locally on that branch, not pushed to GitHub, and will check the standalone .dmg and the plugins before the agent review.

Review findings to fix (reviewer, 2026-11-18):
1. Two orb-ui tests fail after the rebase: `the_standalone_has_no_plugin_usage` (tests/menu.rs ~2650) clicks Help at a fixed x=150, but the Edit menu moved Help right: find Help's position instead. `selected_text_of_a_window_copies_as_text` (tests/menu.rs ~1174) drags at a fixed offset in About, which task-235's default inner margin moved off the text: derive the position.
2. Undo of a Duplicate while playing (menu.rs ~1087-1100) empties the copy at once but the selection returns only at the next bar line, so an emptied pattern plays until the bar line; reverse of Duplicate's pattern-then-selection order. Make Undo not produce that gap (e.g. empty the copy only after the selection has switched back, or keep the copy's bytes until then). In song mode with record on, Undo records a second selection change: handle or document it. Add tests.
3. After a Duplicate, a pattern-button click does not clear the undo (menu.rs ~1248-1261), so a later Undo re-selects the source wherever the user is: clear the undo (or make it not change the selection) when the user selects another pattern. Add a test.
4. docs/ui_spec.md section 12 (~line 839) still says Edit's pattern functions are out of scope; make it agree with section 18.
5. Commit: subject `ui: copy, paste and duplicate a pattern to another slot`; body per AGENTS.md with an up-to-date Testing line and `Task:` this task; keep the contributor as the commit author (git amend keeps the author) and keep their Co-Authored-By lines.

Not in scope: sharing the pattern reading and `pattern_name` with the plugin's drag-out (crates/orb-plugin/src/clip.rs), the drag-and-drop follow-up, a status-line timer.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Branch pr-16 is one commit on current main, author unchanged
- [x] #2 cargo test (whole workspace), clippy -D warnings and nightly fmt check pass in the worktree
- [x] #3 Undo of a Duplicate during playback plays no emptied pattern before the selection switches back, covered by a test
- [x] #4 Selecting another pattern after a Duplicate makes Undo not move the selection, covered by a test
- [x] #5 docs/ui_spec.md section 12 and 18 agree
- [x] #6 Commit subject is ui:, body has Change/Result/Testing/Task lines
- [x] #7 Nothing pushed to GitHub, no PR comment
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Finding 1: tests/menu.rs: the_standalone_has_no_plugin_usage finds Help with open_menu_with (scan along the bar) instead of x=150; selected_text_of_a_window_copies_as_text drags across the "Version " line at the rect where the frame painted it (new Harness::text_rect from the frame's Shape::Text), instead of a fixed offset.
2. Finding 2: Undo sends the selection first; a pattern the row no longer selects goes to the mirror and the panel at once but to the engine only once the engine has run the selection and is stopped or plays another pattern of the row (new Link::set_pattern_unplayed, deferred message sent from Link::follow; a selection or step edit of that pattern sends it first, in Link::forward; load/new/reset_view drop it; Link::moving keeps repainting while one waits). Song mode with record on: Undo's selection is recorded as a click's, documented in ui_spec section 18 and pinned by a test.
3. Finding 3: note_edits clears Undo's select on any Event::Pattern; Duplicate forwards its own selection at once so its own event does not count.
4. Finding 4: ui_spec section 12 names orb's Edit pattern commands (section 18).
5. Finding 5: amend the one commit (author kept) with subject "ui: copy, paste and duplicate a pattern to another slot" and Change/Result/Testing/Task body.
Tests: wired.rs: an_undo_of_a_duplicate_while_playing_never_plays_the_emptied_copy, a_pattern_held_back_by_undo_goes_before_a_selection_of_it, after_a_duplicate_a_selection_makes_undo_leave_the_selection_alone, an_undo_of_a_duplicate_while_recording_is_recorded_as_a_click_is; undo_puts_back_... updated for the new order.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: pr-16 (worktree .claude/worktrees/pr-16), rebased onto main 0928b55 (bbc7452, no conflict).

Work notes (2026-11-18):
- Finding 2, cause: Undo called Link::set_pattern (msg 41 sub 44/61/78 at once) and then select_pattern; while playing the engine switches selection only at the next absolute bar line (Snapshot::patterns = the device's cur_pattern, orb-player lib.rs:968), so the emptied copy played until then.
- Fix: Link::set_pattern_unplayed writes the mirror and the panel at once and holds the message (Link::deferred, with the link's mark); Link::follow sends it from the first snapshot with frames > mark that is stopped or does not play that pattern of the row (cur_pattern does not change while stopped, so a stopped engine's snapshot still names the copy: !playing sends). Link::forward sends it before a Pattern selection of it or a step edit of it (Step303/Clear303 by their pattern, Step808/909 by row_pattern). load, load_new and reset_view drop it; Link::moving repaints while one waits. App::undo uses it when the row's selected pattern is not the undone one (after a Duplicate's undo selected the source, or after the user selected elsewhere), and set_pattern otherwise (Cut/Paste of the selected pattern, unchanged).
- Mark timing: flush_marked sets mark = snapshot frames + follow_frames (1024), so the held pattern goes after 2 engine calls when stopped (test undo_puts_back_... now waits calls(2)).
- Song mode with record on: Undo's selection is a click's and is recorded after the Duplicate's (test an_undo_of_a_duplicate_while_recording_is_recorded_as_a_click_is: 808 controller 1 after tick 0 = [2, 3, 2]); documented in ui_spec section 18 instead of removing the recorded event.
- Finding 3: App::note_edits clears Undo::select on any Event::Pattern (any row); App::duplicate_pattern now note_edits + forwards its own selection at once so the next frame's selections are the user's.
- Checked the new tests catch the old code: with undo forced to set_pattern, an_undo_of_a_duplicate_while_playing_never_plays_the_emptied_copy fails at "the engine plays the copy whole" (wired.rs:1852), a_pattern_held_back... and undo_puts_back... fail too; with the select clearing disabled, after_a_duplicate_a_selection_makes_undo_leave_the_selection_alone fails (wired.rs:1918).
- Finding 1: the_standalone_has_no_plugin_usage now uses open_menu_with("Keyboard Shortcuts..."); selected_text_of_a_window_copies_as_text drags across the "Version " line at the rect the frame painted (Harness::text_rect, from the frame's Shape::Text).
- Finding 4: ui_spec section 12 now names orb's Edit pattern commands (section 18); section 18 and the implementation bullet (~line 969) describe the Undo order and the held-back pattern.

Checks (2026-11-18, worktree, without ORB_CD_ISO and the third-party folders): cargo test 938 passed, 0 failed, 5 ignored (was 2 failing in orb-ui menu before the fix); cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. git diff main --stat lists only the PR's files and this task's file.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed review findings 1-5 of PR 16 locally on branch pr-16 (one commit, author Jochen Rieger kept, rebased onto main 0928b55).
- 1: the two failing orb-ui menu tests find Help along the bar (open_menu_with) and drag across the About window's "Version " line where the frame painted it (Harness::text_rect).
- 2: Undo sends the source's selection first; a pattern the row no longer selects goes to the mirror and panel at once and to the engine only once the engine has run the selection and is stopped or plays another pattern (Link::set_pattern_unplayed, sent from Link::follow; a selection or step edit of it sends it first). A playing row now plays the whole copy up to the bar line (test an_undo_of_a_duplicate_while_playing_never_plays_the_emptied_copy, which fails on the old code). In song mode with record on, Undo's selection is recorded as a click's (documented, test an_undo_of_a_duplicate_while_recording_is_recorded_as_a_click_is).
- 3: any pattern selection after a Duplicate clears Undo's re-selection (test after_a_duplicate_a_selection_makes_undo_leave_the_selection_alone).
- 4: docs/ui_spec.md section 12 agrees with section 18; section 18 describes the Undo order.
- 5: commit subject ui:, body Change/Result/Testing/Task.
Tests: cargo test 938 passed, 5 ignored (without ORB_CD_ISO), clippy and nightly fmt clean. Nothing pushed, no PR comment.
<!-- SECTION:FINAL_SUMMARY:END -->
