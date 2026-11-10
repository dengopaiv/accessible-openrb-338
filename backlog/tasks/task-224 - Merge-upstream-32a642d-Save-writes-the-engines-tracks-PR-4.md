---
id: TASK-224
title: 'Merge upstream 32a642d: Save writes the engine''s tracks (PR #4)'
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-10T19:47:00Z'
updated_date: '2026-11-10T20:04:00Z'
labels:
  - ui
dependencies:
  - TASK-220
references:
  - https://github.com/pingw33n/openrb-338/commit/32a642d
  - https://github.com/pingw33n/openrb-338/pull/4
priority: medium
type: task
ordinal: 242000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user decided on 2026-11-10 (task-220) to take upstream's pull request #4 into the fork now that upstream has merged it: commit 32a642d on github.com/pingw33n/openrb-338 main, "tools: Save writes the engine's tracks for every song (#4)" (upstream task-143, decision task-080). It is upstream's only commit after the fork's base cbca449 (2026-11-03).

What it does: orb-ui's Save writes the TRAK chunks from the engine's tracks for every song (4.x, 1.x and new), as the original's TRAK writer sub_437CFF; an unedited save is no longer byte for byte the file; a stop's prune does not mark the song modified; File > Info's Last event row follows the engine's tracks. Files: crates/orb-ui/src/{document,info,link,menu}.rs, crates/orb-ui/tests/{menu,song_mode_save,switch_values}.rs, docs/{rbm,song_load,ui_spec}.md, upstream's task-143 file. The engine is not changed.

How: a merge commit of 32a642d into main (not a cherry-pick, so later upstream merges have the right base; not a rebase, the fork's main is public). The one conflict is crates/orb-ui/src/menu.rs, App::open: take upstream's self.settle(path, opened.source, ...) without the 1.x document::set_engine_tracks copy, and keep the fork's task-189 announcement "Opened {name}, mod {mod}" after it. Save and Export announcements in screen reader mode are not touched by the change.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 main contains 32a642d as a parent of a merge commit, with menu.rs resolved as described (upstream's settle call, the fork's Opened announcement kept)
- [x] #2 cargo build --release, cargo test --release -p orb-ui, cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check pass
- [x] #3 The fork's Save and Open announcements (task-189) are unchanged: their tests pass
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-224-upstream-32a642d (in the main checkout, no worktree, to share its target folder).
Upstream fetched as refs/remotes/upstream/main = 32a642d. git merge --no-ff upstream/main: auto-merged document.rs, info.rs, link.rs, tests/menu.rs, song_mode_save.rs, switch_values.rs, docs/rbm.md, song_load.md, ui_spec.md; one conflict, menu.rs App::open: resolved to upstream's self.settle(path, opened.source, ...) (the 1.x set_engine_tracks copy is now in document::compose for every song) followed by the fork's task-189 announcement. No leftover use of LoadForm or set_engine_tracks in menu.rs.
Tests: cargo +nightly fmt --all --check clean; cargo clippy --all-targets -- -D warnings clean; cargo test --release -p orb-ui 467 passed, 0 failed, 3 ignored (35 binaries), among them tests/a11y_announce.rs ("Opened Trakka Rekka.rbs, mod ...") and upstream's new tests/menu.rs engine-tracks checks. ORB_CD_ISO and the third-party folders not set here, so their gated tests skipped.

Review round 1 (agent-reviewer): code merge correct; send back for (A) docs/accessibility_plan.md's Upstream paragraph still saying PR #4 was unmerged with no conflict, now rewritten to the merged state of 2026-11-10 with the conflict's resolution and PR #6; (B) the debug run, recorded here: cargo test -p orb-ui (debug) 468 passed, 0 failed, 3 ignored (one more than release: debug_asserts in set_engine_tracks and the new !pending assert in Link::queue_song run and hold). Landing shape: a two-parent merge commit, not a squash, deliberately (the task's How), so later upstream merges have 32a642d as their base; the supervisor accepts this deviation from AGENTS.md's one-squashed-commit rule for upstream merges and reports it to the user.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Upstream's 32a642d (PR #4, Save writes the engine's tracks for every song) is merged into the fork as a merge commit on branch task-224-upstream-32a642d, the one conflict (menu.rs App::open) resolved to upstream's code plus the fork's Opened announcement. fmt and clippy clean; cargo test --release -p orb-ui 467 passed, 3 ignored. Decision: task-220 (the user, 2026-11-10).
<!-- SECTION:FINAL_SUMMARY:END -->
