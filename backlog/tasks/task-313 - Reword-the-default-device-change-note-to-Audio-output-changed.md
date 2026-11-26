---
id: TASK-313
title: Reword the default-device change note to Audio output changed
status: Done
assignee:
  - '@agent-chore'
created_date: '2026-11-26T07:10:00Z'
updated_date: '2026-11-26T07:14:00Z'
labels:
  - player
  - ui
dependencies: []
priority: low
type: enhancement
ordinal: 284700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user asked (2026-11-26) to reword the status-line note that orb-player sends when the system default output device changes and the stream reopens on the new one. Today it reads "Audio output follows the system: <device>"; it must read "Audio output changed: <device>".

Where: crates/orb-player/src/control.rs:134, the Event::Note in the default-device watch. No test or doc quotes the old text (git grep "follows the system" finds only this line, two doc comments in crates/orb-player/src/output.rs:63 and :167, which describe behaviour and stay, and old backlog notes, which are not edited).

Keep the ": <device>" suffix. Change nothing else.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 control.rs sends Event::Note with the text Audio output changed: <device> when the default device changes
- [x] #2 git grep shows no remaining user-visible Audio output follows the system text
- [x] #3 cargo test -p orb-player, clippy with -D warnings and nightly fmt --check pass
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Evidence (worktree root, nightly fmt, stable clippy):
- Edit: crates/orb-player/src/control.rs:134 changed to Event::Note(format!("Audio output changed: {}", s.device)). Only that line changed (git diff --stat: 1 file, 1 insertion, 1 deletion). The ': {}' suffix and the argument are unchanged. Longest line in the file is 99 columns.
- cargo +nightly fmt --all --check: exit 0 (no reflow needed).
- cargo test -p orb-player: exit 0; every suite ok (8, 1, 2, 3, 3, 2, 2 and 1 passed, 0 failed; the reopen test a_reopen_keeps_the_engine_running passed).
- cargo clippy --all-targets -- -D warnings: exit 0.
- git grep 'follows the system': only backlog history (task-127, task-191, and this task's own description) and output.rs:63 and :167, the two doc comments that the task says stay. No user-visible text remains.
- Logs: /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/worktree-agent-a45a5e0355606f3b9/ (fmt.log, test.log, clippy.log, summary.log).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reworded the default-device change note in crates/orb-player/src/control.rs:134 from 'Audio output follows the system: <device>' to 'Audio output changed: <device>'. Nothing else changed. Checks, run in the worktree: cargo +nightly fmt --all --check exit 0; cargo test -p orb-player exit 0 (all suites pass); cargo clippy --all-targets -- -D warnings exit 0. git grep finds no user-visible 'follows the system' text left; the two output.rs doc comments stay, as the task asks. Branch worktree-agent-a45a5e0355606f3b9, one commit on top of 1544621.
<!-- SECTION:FINAL_SUMMARY:END -->
