---
id: TASK-200
title: 'crash.rs: the release-build backtrace assertion fails on Windows'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T20:08:00Z'
labels:
  - tests
  - ui
dependencies: []
references:
  - crates/orb-ui/tests/crash.rs
  - crates/orb-ui/src/crash.rs
priority: low
type: bug
ordinal: 221000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: `cargo test --release -p orb-ui` on Windows fails crates/orb-ui/tests/crash.rs `a_deliberate_panic_exits_101_and_writes_the_log` at crash.rs:56, which asserts that the crash log's backtrace contains a frame named `orb_ui::crash`. It fails the same way on main at 2611c80, before the m-7 work, so it is not caused by it; found while merging TASK-183 (2026-11-05). The likely cause: the release profile builds without debug info, and on Windows (MSVC) a backtrace's frames are named only from a PDB, so the release log has no `orb_ui::crash`. CI runs `cargo test` in debug (with symbols), where the test is expected to pass. Not checked yet: whether the debug run passes on Windows, and what the release log's backtrace looks like.

What: find out (run the test in debug and in release on Windows; read the log it writes), then either make the assertion hold where symbols exist and be skipped or relaxed (with a comment saying why) where they cannot, or make the release build keep enough to name the frame (`debug = "line-tables-only"` in the profile costs size; that is a decision for @dengopaiv, a Decide task, if it comes to it).

References: crates/orb-ui/tests/crash.rs, crates/orb-ui/src/crash.rs, Cargo.toml (profiles), docs/building.md (the crash log section).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The cause is recorded in the notes with the log's backtrace from a debug and a release run on Windows
- [ ] #2 cargo test -p orb-ui and cargo test --release -p orb-ui both pass on Windows, or the remaining case is a Decide task
<!-- AC:END -->
