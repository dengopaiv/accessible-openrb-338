---
id: TASK-249
title: 'PR 16: pattern copy, paste and duplicate, fixed on main'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-18T14:13:00Z'
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
- [ ] #1 Branch pr-16 is one commit on current main, author unchanged
- [ ] #2 cargo test (whole workspace), clippy -D warnings and nightly fmt check pass in the worktree
- [ ] #3 Undo of a Duplicate during playback plays no emptied pattern before the selection switches back, covered by a test
- [ ] #4 Selecting another pattern after a Duplicate makes Undo not move the selection, covered by a test
- [ ] #5 docs/ui_spec.md section 12 and 18 agree
- [ ] #6 Commit subject is ui:, body has Change/Result/Testing/Task lines
- [ ] #7 Nothing pushed to GitHub, no PR comment
<!-- AC:END -->
