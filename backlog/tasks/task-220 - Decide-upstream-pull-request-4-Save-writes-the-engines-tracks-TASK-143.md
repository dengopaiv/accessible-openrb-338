---
id: TASK-220
title: 'Decide: upstream pull request #4, Save writes the engine''s tracks (TASK-143)'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-09T17:33:00Z'
updated_date: '2026-11-10T19:47:00Z'
labels:
  - decision
  - ui
dependencies: []
references:
  - https://github.com/pingw33n/openrb-338/pull/4
  - >-
    backlog/tasks/task-143 -
    Save-always-writes-the-engines-tracks-as-the-original-does.md
priority: low
type: spike
ordinal: 238000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: on 2026-11-09 a contributor opened upstream pull request #4, "Save engine tracks for all songs" (github.com/pingw33n/openrb-338/pull/4, head 8f34804, not merged). It is an implementation of TASK-143, which this fork's backlog also holds, open, in no milestone ("scheduled by the user"). The fork has to choose what to do with it.

What the PR does: every Save writes the TRAK chunks from the engine's tracks (task-080's changed decision), for 4.x, 1.x and new songs: events the load rejected are left out, the song constructor's tick-0 events and merged recordings are written, and a stop's prune as far as it has run. An unedited save is no longer the file byte for byte; Export uses the same composed song. Files: crates/orb-ui/src/document.rs, link.rs, menu.rs (App::song_bytes takes &mut self, composed_song becomes song_for_save), tests/menu.rs, song_mode_save.rs, switch_values.rs, docs/rbm.md, docs/ui_spec.md. The engine is not changed. The author reports cargo test --release 679 pass and the golden corpus equal, without ORB_CD_ISO or the third-party folders.

What was checked here (2026-11-09): upstream's main is still cbca449, the fork's base. git merge-tree of the fork's main (7ebf46b) with the PR gives no conflict, and the fork adds no caller of the changed functions. It was not built or tested here: building a third party's unreviewed code was not run without the user's approval.

Options:
A. Wait for upstream. If pingw33n merges it (or something else for TASK-143), it comes in with the next upstream merge; if not, TASK-143 stays as it is. No work now.
B. Take the PR into the fork now as the implementation of TASK-143: a task branch with the PR's commit, reviewed like any task, the full cargo test --release, clippy and fmt, plus the gated tests with ORB_CD_ISO and the third-party folders, and a by-ear check that Save and Export still announce as before in screen reader mode.
C. Leave TASK-143 unscheduled and ignore the PR.

Recommendation: A. The PR's own text calls it a proposal for upstream's maintainer to decide; it changes what Save writes (no more byte-identical unedited saves), which is upstream's decision of 2026-10-29 rather than an accessibility need; and taking it before upstream decides risks a second, different version later. Screen reader mode does not depend on it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen A, B or C; the outcome and date are in the final summary
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome (the user, 2026-11-10): include it. Upstream merged pull request #4 on 2026-11-10 as 32a642d ("tools: Save writes the engine's tracks for every song (#4)", task-143 and decision task-080 upstream), which settles option A's condition; the user: "since it's now answered, [it] should then also be included". A merge of the fork's main (cc62823) with 32a642d conflicts in one place, crates/orb-ui/src/menu.rs in App::open (upstream drops the 1.x set_engine_tracks copy; the fork adds the "Opened ..., mod ..." announcement of task-189). The merge is its own task (the next task created).
<!-- SECTION:FINAL_SUMMARY:END -->
