---
id: TASK-304
title: >-
  Plugin: log the creating thread, and check REAPER and Studio One on Windows
  for parallel instance creation
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-25T11:54:00Z'
labels:
  - player
  - testmachine
milestone: m-5
dependencies:
  - TASK-303
references:
  - crates/orb-plugin/src/plugin_log.rs
  - docs/plugin.md
priority: low
type: task
ordinal: 275700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** task-303 (decided 2026-11-25: accept) left one question open. On Windows, nice-plug 0.4.2 aborts the host when two threads create instances within the same 100 ns QueryPerformanceCounter tick (docs/plugin.md section 7; task-302 has the full cause). The belief that hosts create instances only on their main thread is unverified. The user offered to check it in REAPER and Studio One on Windows; their other hosts are trial or lite versions with limits.

**Change:** add one line to the plugin's diagnostic log (`ORB_PLUGIN_LOG`, crates/orb-plugin/src/plugin_log.rs, docs/plugin.md section 10) at instance creation, the earliest point in our code inside the host's createInstance call (the plugin's construction). The line holds:
- the OS thread id;
- a high-resolution timestamp (QueryPerformanceCounter on Windows, a monotonic clock elsewhere);
- the instance's sequence number in the process.

It costs nothing when the log is off. Also log the thread id on the main-thread entry points that already log (activation), so creation can be compared with them.

Build the Windows x86_64 zip (scripts/package.sh) for the user, and write a short how-to for them:
- which log file to set with ORB_PLUGIN_LOG, and how to set it for REAPER and Studio One on Windows;
- the steps: a project with about 20 instances saved and reopened, duplicating tracks in bulk, and a plugin rescan;
- what to send back.

**Result to record:** for each host and action, whether every creation is on one thread id, or two threads overlap. Put it in docs/plugin.md section 7 next to the race, and in the notes. If a host creates instances on several threads, create a follow-up task that reopens the task-303 choice (vendor/patch nice-plug), and tell the supervisor.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Creating an instance logs its thread id, timestamp and sequence number when ORB_PLUGIN_LOG is set; nothing when it is not
- [ ] #2 The user has run the how-to in REAPER and Studio One on Windows, and the result per host and action is in docs/plugin.md section 7 and the notes
- [ ] #3 A host that creates instances on several threads gets a follow-up task; otherwise the notes say none was seen
- [ ] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
<!-- AC:END -->
