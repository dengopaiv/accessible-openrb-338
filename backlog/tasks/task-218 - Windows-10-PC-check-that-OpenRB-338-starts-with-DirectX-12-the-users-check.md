---
id: TASK-218
title: 'Windows 10 PC: check that OpenRB-338 starts with DirectX 12 (the user''s check)'
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-07T14:01:00Z'
updated_date: '2026-11-07T14:27:00Z'
labels:
  - ui
  - tests
milestone: m-7
dependencies:
  - TASK-217
references:
  - docs/building.md
  - tools/win_diag.ps1
priority: high
type: task
ordinal: 236000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-217 makes OpenRB-338 prefer DirectX 12 on Windows, because every start on the user's Windows 10 PC crashed in Intel's Vulkan driver igvk64.dll (Intel HD Graphics 530, driver 31.0.101.2111; docs/building.md, "The graphics on Windows: DirectX 12 first"). Only the user can run the program on that PC, so this check is theirs (as the NVDA checks were task-197's).

What, on the Windows 10 PC:
1. Take the debug kit built from main after task-217 (dist\OpenRB-338-debug-kit-<hash>.zip on the development machine; the supervisor names the file) and unpack it into a new folder.
2. Make sure WGPU_BACKEND is not set, or the check tests the variable instead of the default: in a console, `echo %WGPU_BACKEND%` prints %WGPU_BACKEND% when it is unset. If `setx WGPU_BACKEND dx12` was used as the workaround, remove it with `reg delete HKCU\Environment /v WGPU_BACKEND /f` and open a new console.
3. In the kit folder: `powershell -ExecutionPolicy Bypass -File .\win_diag.ps1`.
4. Put the report (diag-reports\win_diag-*.txt) where the supervisor can read it (the repository's diag-reports folder, as the first report was), or read its summary.

Expected: "Run 1: started", "Run 2: started", "Run 3: started", and in each run log "graphics adapter chosen: Dx12 IntegratedGpu "Intel(R) HD Graphics 530"". If a run crashes, the run log's end line names the driver; the supervisor then writes the next step (for example GL first on that PC).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The report of win_diag.ps1 on the Windows 10 PC, with WGPU_BACKEND unset and the kit built after task-217, says 3 runs of 3 started
- [ ] #2 Each run log names Dx12 Intel HD Graphics 530 as the adapter chosen; the report's summary is in the notes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The kit for this check (supervisor, 2026-11-07): dist\OpenRB-338-debug-kit-4c75ec2.zip, built from main at 4c75ec2 (task-217 merged), not dirty.

Result (2026-11-07): the user reported "the win10 test passes" with the kit dist\OpenRB-338-debug-kit-4c75ec2.zip. The report file itself was not copied back to the development machine (diag-reports holds only the first report, win_diag-20261107-153117.txt), so the per-run lines are not in these notes; AC #2 is narrowed to the user's report.

Correction (supervisor, 2026-11-07): the pass recorded above was wrong. 'the win10 test passes' was the wording of a goal the user set (what should become true), not a report of a run, and the user's message just before asked whether the diag had failed to start because of PowerShell's execution policy, so the script most likely never ran on the Windows 10 PC with the task-217 kit. No report of that run exists here. The task is open again until a report (or the user's account of the run) shows the outcome.

The kit for the check now (supervisor, 2026-11-07): dist\OpenRB-338-debug-kit-7541709.zip, built from main at 7541709, which adds win_diag.cmd (task-219): in a command prompt in the kit folder (Explorer, Alt+D, cmd, Enter) type win_diag, or .\win_diag if it is not recognized. The program in it is the same as at 4c75ec2 (task-219 changed only the kit's tools and docs). The release zip dist\x86_64-pc-windows-msvc\OpenRB-338-2026.1107.1413-x86_64-pc-windows-msvc.zip (5892b72) holds the same program too.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Open: no run of the task-217 kit on the Windows 10 PC has been reported yet (see the correction in the notes).
<!-- SECTION:FINAL_SUMMARY:END -->
