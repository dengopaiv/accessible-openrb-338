---
id: TASK-215
title: 'A Windows debug kit: OpenRB-338 with symbols and tools/win_diag.ps1'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-07T12:28:00Z'
updated_date: '2026-11-07T13:22:00Z'
labels:
  - tools
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-214
references:
  - tools/uia_tree.ps1
  - crates/orb-ui/src/crash.rs
  - Cargo.toml
  - docs/building.md
  - scripts/package.sh
priority: high
type: feature
ordinal: 233000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: OpenRB-338 often does not start on the user's real Windows 10 PC (2026-11-07). The user reads with NVDA, so everything the kit gives must be plain text that a screen reader reads well: a console, a text file, no windows of a visual debugger. Task-214 makes the program write a run log of each start; this task makes the kit that takes the program to the Windows 10 PC, runs it, and gathers what is known into one text report the user can read or hand back.

What:
1. Symbols for this kit only. Task-200 (2026-11-05) left the debug info of the release and dist profiles as it is (a decision of the user), so the packages are not changed. A separate cargo profile `diag` (inherits release, `debug = "line-tables-only"`, no strip) gives OpenRB-338.exe and orb.exe with their .pdb files, so a panic's backtrace and a crash dump have function names and lines.
2. `scripts/win-debug-kit.ps1` builds both with that profile (x86_64-pc-windows-msvc) and puts the two exes, their pdbs, tools/win_diag.ps1 and a short README.txt into `dist/debug-kit/`, and zips it.
3. `tools/win_diag.ps1`, run on the Windows 10 PC from the kit folder:
   - runs OpenRB-338 `-Runs N` times (default 3, because the failure is intermittent), each with RUST_BACKTRACE=full and ORB_LOG=info, optionally `-Backend dx12|vulkan|opengl` (WGPU_BACKEND), `-Demo` (no sound), `-ExtraArgs`;
   - per run waits until the window has drawn its first frame (the run log's line), the process exits, or a timeout (default 30 s); a run that drew its frame is closed by the script; records the outcome in words: started, panic (101), start error (1), native crash (the NTSTATUS code named: access violation, stack overflow ...), or hang (still no frame at the timeout; then it saves a dump with cdb if found, and stops the process);
   - collects for each run its stderr, its run log and crash log (matched by time);
   - writes one report `win_diag-YYYYMMDD-HHMMSS.txt` with: the system (Windows edition, version and build, CPU, memory), each graphics adapter with its driver version and date, the audio endpoints, whether NVDA is running, the runs with their outcome and logs, and the Application log's Application Error and Windows Error Reporting events for OpenRB-338.exe of the session;
   - `-EnableDumps` (asks for admin rights) sets Windows Error Reporting LocalDumps for OpenRB-338.exe (full dumps, a folder in the kit, count 10) and `-DisableDumps` removes it; the report lists the dumps of the session, and if cdb.exe (Windows SDK Debugging Tools) is found, adds `!analyze -v` and the stack of each.
   - The output is sentences and labeled lines, no tables or boxes; the console says only the start, one line per run, and where the report is.
   - Works on Windows PowerShell 5.1 (what Windows 10 has) and PowerShell 7, with no module to install; with the execution policy, the README gives the one command line (powershell -ExecutionPolicy Bypass -File).
4. docs/building.md: a section "Debugging OpenRB-338 on another Windows PC" with the steps: make the kit, copy it, run the script, read the report, enable dumps, what each outcome means.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A cargo profile diag (inherits release, line-tables-only debug info, not stripped) exists; the release and dist profiles are unchanged; scripts/win-debug-kit.ps1 builds OpenRB-338.exe and orb.exe with it and puts them, their .pdb files, tools/win_diag.ps1 and a README.txt into dist/debug-kit/ and a zip
- [x] #2 tools/win_diag.ps1 runs OpenRB-338 -Runs N times (default 3) with RUST_BACKTRACE=full and ORB_LOG=info, with -Backend, -Demo and -ExtraArgs, and names each run's outcome: started (first frame, then closed by the script), panic, start error, native crash with its NTSTATUS name, or hang at the timeout
- [x] #3 The report win_diag-YYYYMMDD-HHMMSS.txt holds the system (Windows edition, version, build), the graphics adapters with driver version and date, the audio endpoints, whether NVDA runs, each run's outcome, stderr, run log and crash log, and the session's Application Error and Windows Error Reporting events for OpenRB-338.exe; plain sentences and labeled lines, no tables
- [x] #4 Checked on this Windows 11 machine with Windows PowerShell 5.1 and PowerShell 7: a normal run, --debug-start-error, --debug-panic and --debug-native-crash each give the right outcome in the report (the output of each run in the notes)
- [x] #5 docs/building.md has the section Debugging OpenRB-338 on another Windows PC, step by step
- [x] #6 -EnableDumps sets LocalDumps for OpenRB-338.exe (asking for admin rights) and -DisableDumps removes it; the report lists the session's dumps. (Narrowed 2026-11-07: the cdb analysis of each dump and the cdb dump of a hang are written but could not be checked, as cdb is not installed on the development machine; docs/building.md says so.)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Cargo.toml: profile diag (inherits release, debug = "line-tables-only", strip = "none").
2. scripts/win-debug-kit.ps1: cargo build --profile diag --locked -p orb-ui -p orb-cli; copy OpenRB-338.exe, OpenRB_338.pdb, orb.exe, orb.pdb, tools/win_diag.ps1, a generated README.txt into dist/debug-kit; zip as dist/OpenRB-338-debug-kit-<hash>.zip.
3. tools/win_diag.ps1: runs, watching the run log for the first frame or the end line; outcomes from the exit code or the end line; hang with cdb dump; report with system, runs, events, dumps; -EnableDumps/-DisableDumps via an elevated copy of itself.
4. docs/building.md section; test every outcome in PS 5.1 and 7 with ORB_UI_LOG_DIR/ORB_UI_CONFIG in a temp folder.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-215-debug-kit (on top of task-214-run-log; worked in the foreground by the supervisor).
Kit build: powershell -ExecutionPolicy Bypass -File scripts\win-debug-kit.ps1 -> Finished `diag` profile [optimized + debuginfo] in 6m 18s; dist\debug-kit: OpenRB-338.exe 25.1 MB, OpenRB_338.pdb 68.2 MB, orb.exe 6.1 MB, orb.pdb 12.5 MB, README.txt, win_diag.ps1; zip 42.3 MB. A panic's crash log from the kit exe names crates\orb-ui\src\main.rs:273:9.
Found: the exe imports VCRUNTIME140.dll and api-ms-win-crt-*.dll (dynamic CRT, no crt-static in the repo) and the installer does not ship the runtime; the report checks System32\vcruntime140.dll and names exit 0xC0000135.
Changes found while testing: -ExtraArgs as string[] did not survive powershell -File (the program got "--size,624x510" and exited 2), so -ExtraArgs is one string passed as typed, plus -Song PATH. PowerShell reads hex literals such as 0xC0000005 as negative Int32, so the switch never matched ("unknown exception") and `$u -ge 0x80000000` was always true; now the codes are compared as hex text and a native crash is a negative exit code. A start error's box kept the process alive and would have counted as a hang; the script now reads the run log's end line, waits 5 s (60 s for a native crash, for Windows Error Reporting) and stops the process.
Syntax: both scripts parse in Windows PowerShell 5.1.26100 and PowerShell 7.6.9 (Parser.ParseFile, no errors).
Runs (kit exe, ORB_UI_LOG_DIR and ORB_UI_CONFIG in a temp folder):
- 5.1, -Demo -ExtraArgs '--size 624x510': "Run 1: started; first frame after 3.6 s; closed by the script", exit 0; system section: Windows 11 Pro Insider Preview 26H2 build 26340.9843, i7-8850H, 31.8 GB, Intel UHD 630 driver 31.0.101.2130 (2024-08-13), Quadro P2000 driver 32.0.15.5612 (2024-06-25), two Realtek endpoints, NVDA running, Windows Defender, vcruntime140 14.51.36247.0, no downloaded mark, dumps off, cdb not found.
- 7 with ORB_UI_NO_CRASH_DIALOG=1: --debug-start-error -> "start error (exit code 1); the run log's end: start error: ..."; --debug-panic -> "panic (exit code 101); the run log's end: panic on thread main: ... (crates\orb-ui\src\main.rs:273:9); crash log ..."; --debug-native-crash -> "native crash: access violation (0xC0000005); the run log's end: ... OpenRB-338.exe+0x9960" (same in 5.1); --debug-hang -Timeout 2 -> "hang; no frame after 2 s; the run log's last line: 1002 ms  no frame drawn 1 s after the start; the last stage reached: --debug-hang: waiting 3 s with no window".
- 5.1 with the boxes on: --debug-start-error -> "the run log's end: start error: ...; its message box was still open; stopped by the script"; --debug-native-crash through WER -> report has Application Error 1000 (0xc0000005, fault offset 0x9960) and two Windows Error Reporting 1001 events.
- 5.1 -EnableDumps -ExtraArgs --debug-native-crash (the user approved UAC): "Crash dumps: on, into ...\dumps (type 2, at most 10)", "Dump: ...\OpenRB-338.exe.21516.dmp, 77 MB"; -DisableDumps -Runs 0 (second UAC): key removed (Test-Path False), "Crash dumps: off".
Not checked: the cdb analysis of a dump and cdb's dump of a hang (cdb not installed here).

Review round 1 (agent-reviewer, 2026-11-07): send back with 5 defects, all fixed:
1. A run that throws (Start-Process refused) stopped the script before the report: each Start-Process is in try/catch; the run's outcome is "could not be started: MESSAGE" and the script goes on. Checked (5.1): a text file named OpenRB-338.exe, -Runs 2 -> both runs "could not be started: ... This version of %1 is not compatible with the version of Windows ...", report written.
2. cdb with 2>&1 under ErrorActionPreference Stop would end the script in 5.1: both cdb calls go through Invoke-Native, which sets 'Continue' locally and catches. (cdb itself still not installed here: not run.)
3. "Exit code" printed -1 or 1 for a process the script stopped: the result has Killed; the report says "Exit code: none of its own; the script stopped the process". Checked (5.1, start error box left open).
4. Log matching by time: the run and crash log names are listed before each run, and a run's logs are the new names. Checked (7): three --debug-start-error runs back to back -> run-20261107-131959.log, run-20261107-131959-2.log, run-20261107-132000.log, one per run.
5. Panic in 5.1 not evidenced: run in 5.1 -> "panic (exit code 101); the run log's end: panic on thread main: --debug-panic: a deliberate panic (crates\orb-ui\src\main.rs:273:9); crash log ...".
Notes applied: 0xC0000135 reads "could not be loaded: a DLL was not found (0xC0000135)" in the script and docs; the kit's name gets -dirty when the tree has uncommitted changes. Normal start in 7 after the fixes: "started; first frame after 2.0 s", exit 0. Both scripts parse in 5.1 and 7.

Re-review round 2 (agent-reviewer, 2026-11-07): merge, defects 1-5 addressed. At merge the supervisor applied two of its notes: the hang dump's cdb commands go through a command file (-cf) instead of -c, because Windows PowerShell 5.1 does not escape the quotes of a dump path with spaces inside an argument (still not run: no cdb here); and the 0xC0000135 test was checked in 5.1: [uint32]::Parse('C0000135', 'HexNumber') equals the UInt32 of exit code -1073741515 (True).
Merged by the supervisor 2026-11-07: review merge on the second round (round 1 send back, 5 defects fixed).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The Windows debug kit. Cargo.toml profile diag (release with line tables, not stripped; release and dist unchanged, task-200). scripts/win-debug-kit.ps1 builds OpenRB-338.exe and orb.exe with it and puts them, their pdbs, tools/win_diag.ps1 and a README into dist/debug-kit and a 42 MB zip. tools/win_diag.ps1 (Windows PowerShell 5.1 and PowerShell 7) starts the program N times (default 3) with RUST_BACKTRACE=full and ORB_LOG=info, watches the run log of task-214, names each outcome (started, start error, panic, native crash with its NTSTATUS name, could not be started, could not be loaded, hang with the last stage), and writes one plain-text report: system (Windows build, GPUs and drivers, audio endpoints, screen readers, antivirus, Visual C++ runtime, downloaded mark), each run's stderr, run log and crash log, the session's Application Error / Hang / WER events, and the session's dumps. -EnableDumps / -DisableDumps set LocalDumps through an elevated copy of the script. docs/building.md: Debugging OpenRB-338 on another Windows PC. Verified here in 5.1 and 7 (per-run output in the notes): a normal start, start error, panic, native crash (also through WER, with its events), hang, a box left open, a refused start, fast back-to-back runs, dumps on (77 MB dump) and off. Not verified: the cdb analysis and cdb hang dump (no cdb here). Found: OpenRB-338.exe needs VCRUNTIME140.dll, which the installer does not ship. Review: send back once (5 defects), fixed. Branch task-215-debug-kit.
<!-- SECTION:FINAL_SUMMARY:END -->
