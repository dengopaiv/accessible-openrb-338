---
id: TASK-307
title: >-
  Decide: Save writes the original's bytes where its chunk packers write none or
  zeros
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-25T13:34:00Z'
updated_date: '2026-11-25T14:27:00Z'
labels:
  - decision
  - ui
milestone: m-5
dependencies: []
references:
  - docs/song_load.md
  - docs/original_bugs.md
  - crates/orb-ui/src/document.rs
priority: low
type: spike
ordinal: 278700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** task-306 compared orb's saved .rbs with the original's saves of the same songs (tests/songs/edit_init, made on the test machine) and found three kinds of bytes where orb's Save differs from the original's although the song is the same. orb keeps them as read (docs/ui_spec.md section 18, Save: 'unknown chunks and unused bytes stay as read'); the original does not. Everything else of those four files is equal byte for byte.

**The facts [C, Rebirth.exe, task-306]:**
- The effect chunks: the save's packers clear the chunk first and write only the controllers: DELY `sub_434605` (memset 8, writes 0..4), PCF `sub_4346E1` (memset 12, 0..6), DIST `sub_434567` (memset 8, 0..2), COMP `sub_4347D1` (memset 8, 0..2). So the original always saves zeros in the meter and padding bytes (COMP 3..7, DIST 3..7, DELY 5..7, PCF 7..11). The Silent Default Song template has COMP byte 3 = 127, which orb keeps and the original writes as 0.
- GLOB 483..511 and USRI 544..711: the packers `sub_433EC2` (GLOB, writes 0..482) and `sub_43489D` (USRI, 0..543) never write them, so the original's save leaves there whatever its shared chunk buffer `unk_4DB948` held from the chunks read or written before (OB-26 in docs/original_bugs.md). The shipped songs carry such bytes (0, 1, 2 in scattered places); the task-306 saves have GLOB[498] = 1 and a few USRI tail bytes set.

**Options:**
1. Keep as read (today): a song orb opens and saves gives back its own bytes; orb's saves differ from the original's in these bytes only.
2. Write the original's zeros in the effect chunks' unwritten bytes (they are deterministic), and keep GLOB 483..511 and USRI 544..711 as read (the original's are not reproducible: they depend on the session's earlier file operations).
3. Write zeros in all of them.

**Recommendation:** option 2: it matches every deterministic byte of the original's save; the stale bytes cannot be matched, and keeping them as read loses nothing. It changes `crates/orb-ui/src/document.rs` (compose) and the test `saving_an_unedited_corpus_song_gives_back_the_file` (crates/orb-ui/tests/menu.rs), which would then expect zeros in those bytes; crates/orb-ui/tests/edit_init.rs would drop its COMP mask.

**Acceptance criteria (for the follow-up, once decided):** the decision recorded with its date; if option 2 or 3, a task to implement it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The decision is recorded with its date; if option 2 or 3, a task implements it
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Outcome: option 1 chosen by the user on 2026-11-25 (keep the bytes as read), so no implementation task. Recorded on branch worktree-agent-aa9a478c0a04f2813 with task-306's commit.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-25: option 1, keep the bytes as read. The user does not need saves to match the original byte for byte, only to be valid and readable by both programs. No follow-up task.
<!-- SECTION:FINAL_SUMMARY:END -->
