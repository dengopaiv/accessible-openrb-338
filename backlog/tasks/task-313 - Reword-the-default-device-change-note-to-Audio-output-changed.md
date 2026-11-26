---
id: TASK-313
title: Reword the default-device change note to Audio output changed
status: To Do
assignee:
  - '@agent-chore'
created_date: '2026-11-26T07:10:00Z'
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
- [ ] #1 control.rs sends Event::Note with the text Audio output changed: <device> when the default device changes
- [ ] #2 git grep shows no remaining user-visible Audio output follows the system text
- [ ] #3 cargo test -p orb-player, clippy with -D warnings and nightly fmt --check pass
<!-- AC:END -->
