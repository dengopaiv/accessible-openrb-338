---
id: TASK-200
title: 'crash.rs: the release-build backtrace assertion fails on Windows'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T20:08:00Z'
updated_date: '2026-11-07T09:29:00Z'
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
- [x] #1 The cause is recorded in the notes with the log's backtrace from a debug and a release run on Windows
- [x] #2 cargo test -p orb-ui and cargo test --release -p orb-ui both pass on Windows, or the remaining case is a Decide task
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Run the crash test in debug and release on Windows, read both logs, relax the frame-name check only for Windows release builds (no PDB debug info), keep it everywhere else. No profile change.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ad6959b7472feb57e

Cause: the release profile has no debug info, so on Windows MSVC the frames are named from the exe's public symbols only. Release log (cargo test --release): frame 4 'alloc::io::read::default_read_to_end::<...Cursor...>', frame 5 '<png::text_metadata::TEXtChunk as EncodableTextChunk>::encode::<...BufWriter<File>>' (these are the hook closure's code, named after neighbouring symbols), then 6 std::panicking::panic_with_hook, 7-10 std::panicking::panic_handler / core::panicking::panic_fmt, 11 OpenRB_338::main. No orb_ui::crash and no file:line. Debug log (cargo test): 3 std::backtrace::Backtrace::force_capture, 4 orb_ui::crash::handle at crash.rs:255, 5 orb_ui::crash::install::closure$0::closure$1 at crash.rs:229, ..., 12 orb_ui::crash::install::closure$0. The debug test passes. Fix: crates/orb-ui/tests/crash.rs now always requires a frame list ('   0: '); on cfg!(all(windows, not(debug_assertions))) it requires 'std::panicking' instead of orb_ui::crash, with a comment; elsewhere unchanged. Release profile untouched (line-tables-only would be a user decision; not proposed).

Merged by the supervisor 2026-11-07: review merge on the first round (notes, not defects: the Windows-release check std::panicking is weaker by design; the cfg also covers windows-gnu release; the Linux/macOS release path is not run in CI). crash (2) passes in release on main after the squash.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Cause: Windows release builds have no debug info, so the backtrace names the hook closure after neighbouring public symbols (default_read_to_end, TEXtChunk::encode) and orb_ui::crash never appears; the debug build names it (orb_ui::crash::handle, crash.rs:255). Fix: crates/orb-ui/tests/crash.rs relaxes the frame-name check to std::panicking only on Windows release, with a comment; debug (CI) still checks orb_ui::crash. Release profile unchanged. cargo test -p orb-ui --test crash: 2 pass (debug); cargo test --release -p orb-ui --no-fail-fast: all pass; fmt and clippy clean. Branch worktree-agent-ad6959b7472feb57e.
<!-- SECTION:FINAL_SUMMARY:END -->
