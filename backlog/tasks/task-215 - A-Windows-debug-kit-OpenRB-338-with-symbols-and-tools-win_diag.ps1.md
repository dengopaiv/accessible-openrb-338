---
id: TASK-215
title: 'A Windows debug kit: OpenRB-338 with symbols and tools/win_diag.ps1'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-07T12:28:00Z'
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
- [ ] #1 A cargo profile diag (inherits release, line-tables-only debug info, not stripped) exists; the release and dist profiles are unchanged; scripts/win-debug-kit.ps1 builds OpenRB-338.exe and orb.exe with it and puts them, their .pdb files, tools/win_diag.ps1 and a README.txt into dist/debug-kit/ and a zip
- [ ] #2 tools/win_diag.ps1 runs OpenRB-338 -Runs N times (default 3) with RUST_BACKTRACE=full and ORB_LOG=info, with -Backend, -Demo and -ExtraArgs, and names each run's outcome: started (first frame, then closed by the script), panic, start error, native crash with its NTSTATUS name, or hang at the timeout
- [ ] #3 The report win_diag-YYYYMMDD-HHMMSS.txt holds the system (Windows edition, version, build), the graphics adapters with driver version and date, the audio endpoints, whether NVDA runs, each run's outcome, stderr, run log and crash log, and the session's Application Error and Windows Error Reporting events for OpenRB-338.exe; plain sentences and labeled lines, no tables
- [ ] #4 -EnableDumps sets LocalDumps for OpenRB-338.exe (asking for admin rights) and -DisableDumps removes it; the report lists the session's dumps and, when cdb.exe is found, adds !analyze -v and the stack of each; a hang is dumped with cdb when it is found
- [ ] #5 Checked on this Windows 11 machine with Windows PowerShell 5.1 and PowerShell 7: a normal run, --debug-start-error, --debug-panic and --debug-native-crash each give the right outcome in the report (the output of each run in the notes)
- [ ] #6 docs/building.md has the section Debugging OpenRB-338 on another Windows PC, step by step
<!-- AC:END -->
