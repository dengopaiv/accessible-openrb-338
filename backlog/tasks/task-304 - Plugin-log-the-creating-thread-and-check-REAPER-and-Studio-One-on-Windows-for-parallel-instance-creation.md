---
id: TASK-304
title: >-
  Plugin: log the creating thread, and check REAPER and Studio One on Windows
  for parallel instance creation
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T11:54:00Z'
updated_date: '2026-11-25T12:39:00Z'
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
- [x] #1 Creating an instance logs its thread id, timestamp and sequence number when ORB_PLUGIN_LOG is set; nothing when it is not
- [x] #2 A host that creates instances on several threads gets a follow-up task; otherwise the notes say none was seen
- [x] #3 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #4 The user has run the how-to's actions in REAPER and Studio One on Windows, and the result per host and session (the logs carry no action labels) is in docs/plugin.md section 7 and the notes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. plugin_log::created(): one line per instance (sequence number, OS thread id, QPC ticks / ns) called first in OrbPlugin::default; activated() also logs thread and clock. 2. docs/plugin.md sections 7 and 10: the line and the host check. 3. Windows zip and a how-to. 4. Tests, clippy (mac, xwin), fmt. Not Done: AC 2 and 3 need the user's runs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ade0905a1138fac74

Done: plugin_log::created() (crates/orb-plugin/src/plugin_log.rs), called first in OrbPlugin::default (lib.rs); line 'create: instance N thread T clock C unit' (T = GetCurrentThreadId, C = QueryPerformanceCounter ticks, unit qpc; non-Windows: Rust ThreadId, ns). activated() adds 'activate: thread T clock C unit'. Log off: one OnceLock lookup, nothing else. docs/plugin.md sections 7 (table of host results, empty) and 10. Checks: fmt --check ok, clippy -D warnings macOS ok, cargo xwin clippy x86_64-pc-windows-msvc ok, cargo test --workspace 1164 passed 0 failed. Zip: dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1125.1211-x86_64-pc-windows-msvc.zip. Waiting for the user's REAPER/Studio One results (AC 2, 3); task stays To Do.

HOW-TO (task-304): do REAPER and Studio One create plugin instances on one thread?

Package: OpenRB-338-2026.1125.1211-x86_64-pc-windows-msvc.zip (C:\tmp). Unzip it to C:\tmp\orb304\ and keep the folder name: the OpenRB-338.vst3 inside is a single file.

1. Install: copy C:\tmp\orb304\OpenRB-338-...\OpenRB-338.vst3 into a VST3 folder the host scans, for example C:\Program Files\Common Files\VST3\ (remove any other OpenRB-338 copy there first, so the host loads this one). Then rescan in the host (step 4 does that too).

2. Turn the log on, one log file per host. Open a cmd window (not PowerShell) and start the host from it, so the host inherits the variable:

   REAPER:
     set ORB_PLUGIN_LOG=C:\tmp\orb-reaper.log
     start "" "C:\Program Files\REAPER (x64)\reaper.exe"

   Studio One (the folder name has its version, adjust it):
     set ORB_PLUGIN_LOG=C:\tmp\orb-studioone.log
     start "" "C:\Program Files\PreSonus\Studio One 7\Studio One.exe"

   Close the host fully first (Task Manager: no reaper.exe / Studio One.exe left). The log is appended to; delete the file before a run for a clean one. If no file appears, the host did not pass the variable on: create an empty file %LOCALAPPDATA%\OpenRB-338\logs\plugin.log instead (the log then goes there), and send that.

3. Check it works: add one OpenRB-338 instance. In the log (type C:\tmp\orb-reaper.log) there must be a line like
     12.3 [ThreadId(1)] create: instance 1 thread 11234 clock 4123456789 qpc

4. The actions (note the order; say in your message where each starts, or add the instance count):
   a. Plugin rescan: REAPER: Options > Preferences > Plug-ins > VST > Re-scan (clear cache and re-scan). Studio One: Options > Locations > VST Plug-ins > Reset Blocklist and Rescan (or the Plug-in Manager's rescan).
   b. A project with about 20 OpenRB-338 instances (20 tracks, one instance each; also try several in one track's FX chain). Save it, close the project, reopen it (loading creates the instances). Do the reopen twice, and once also with the host just started and the project opened from its start screen / recent list.
   c. Duplicate tracks in bulk: select 10 tracks with instances, duplicate them (REAPER: Track > Duplicate tracks; Studio One: Track > Duplicate Tracks), then select all 20 and duplicate again.
   d. If the host has an offline or background project load (REAPER: File > Open with "Open project in background tab" / "Run FX in parallel/multiprocessing" under Preferences > Audio > Buffering, turn "Allow live FX multiprocessing" on), repeat b once with it on.
   Crash/abort: if the host closes by itself, note which action. Windows event log is not needed.

5. Send back, per host: the log file (C:\tmp\orb-reaper.log, C:\tmp\orb-studioone.log) and a line per action in the order tried. To summarize yourself: findstr /C:"create:" the-log.log shows the creations; the number after "thread" is the OS thread id. One distinct number for all lines is the answer "one thread". Two numbers: send the file as it is.

Host results (user's run 2026-11-25, logs fetched by the coordinator; sessions split where the instance counter restarts at 1; the logs do not label which action made which session, so it is per session). REAPER (76 creates): s1 1 create thread 22408 (probably the scan); s2 25 creates thread 12560, closest two 4.5 ms apart; s3 50 creates thread 9572, closest 4.1 ms. Studio One (96 creates): s1 32 on 21400, closest 34.0 ms; s2 32 on 21400, closest 12.5 ms; s3 32 on 20528, closest 12.3 ms. No panic or abort line. All creations of every session on one thread; REAPER's activations on the creating thread. Closest gap about 41,000 ticks of 100 ns. REAPER live FX multiprocessing state unknown. No host seen creating on several threads: no follow-up task; task-303 (accept) stands. Table in docs/plugin.md section 7.

Review round 1 (docs and evidence wording; code accepted): (1) AC 2 narrowed, with the supervisor's acceptance, from 'per host and action' to 'per host and session, the user having done the how-to's actions' (it was removed and re-added, so it is now the last criterion, and checked); the action-to-session mapping is unknown, because the logs carry no action labels; the task's question, one thread or several, is answered per session. (2) docs/plugin.md section 7 now states the evidence: six unlabeled sessions of 1, 25, 50 (REAPER) and 32, 32, 32 (Studio One) creates, and does not claim each action ran. (3) 'in REAPER and Studio One' instead of 'no host'. (4) Build provenance: the logs' headers read 2026.1125.1211 (dev-6d61023-dirty): the user ran the zip built from the uncommitted tree. The crates sources (lib.rs 14:04:11, plugin_log.rs 14:04:37) were last changed before the build (zip 14:14, build started after the fmt check), git diff HEAD -- crates is empty after the commit, so the built code equals the committed code; stated in section 7. (5) Section 10 says the switch is now first read at the first instance creation. (6) Section 10 says T is the OS thread id on Windows and the Rust ThreadId text elsewhere.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Plugin log line per instance creation (create: instance N thread T clock C unit, plus an activate: thread line), first in OrbPlugin::default; zero cost with the log off. The user checked REAPER and Studio One on Windows: every session created all instances on one thread, closest creations 4.1 ms apart (the abort needs the same 100 ns tick); no follow-up task. Result in docs/plugin.md section 7. Branch worktree-agent-ade0905a1138fac74, one commit. Tests: cargo test --workspace 1164 pass, clippy macOS and Windows, nightly fmt.
<!-- SECTION:FINAL_SUMMARY:END -->
