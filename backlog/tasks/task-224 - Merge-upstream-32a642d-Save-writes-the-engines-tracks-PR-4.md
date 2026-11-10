---
id: TASK-224
title: 'Merge upstream 32a642d: Save writes the engine''s tracks (PR #4)'
status: To Do
assignee:
  - '@supervisor'
created_date: '2026-11-10T19:47:00Z'
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
- [ ] #1 main contains 32a642d as a parent of a merge commit, with menu.rs resolved as described (upstream's settle call, the fork's Opened announcement kept)
- [ ] #2 cargo build --release, cargo test --release -p orb-ui, cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check pass
- [ ] #3 The fork's Save and Open announcements (task-189) are unchanged: their tests pass
<!-- AC:END -->
