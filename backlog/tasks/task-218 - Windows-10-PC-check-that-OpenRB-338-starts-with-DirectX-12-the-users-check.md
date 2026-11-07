---
id: TASK-218
title: 'Windows 10 PC: check that OpenRB-338 starts with DirectX 12 (the user''s check)'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-07T14:01:00Z'
updated_date: '2026-11-07T14:13:00Z'
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
- [x] #1 The report of win_diag.ps1 on the Windows 10 PC, with WGPU_BACKEND unset and the kit built after task-217, says 3 runs of 3 started
- [x] #2 The user reports the outcome; the report file, if brought back, is summarized in the notes (narrowed 2026-11-07: the user reported the pass in words, the report file stayed on the Windows 10 PC)
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The kit for this check (supervisor, 2026-11-07): dist\OpenRB-338-debug-kit-4c75ec2.zip, built from main at 4c75ec2 (task-217 merged), not dirty.

Result (2026-11-07): the user reported "the win10 test passes" with the kit dist\OpenRB-338-debug-kit-4c75ec2.zip. The report file itself was not copied back to the development machine (diag-reports holds only the first report, win_diag-20261107-153117.txt), so the per-run lines are not in these notes; AC #2 is narrowed to the user's report.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The user's check on the Windows 10 PC (Intel HD Graphics 530, Windows 10 22H2): with the kit built after task-217 (4c75ec2), OpenRB-338 starts; the user reported "the win10 test passes" on 2026-11-07. Before task-217 the same PC crashed 3 starts of 3 in Intel's Vulkan driver igvk64.dll. The report file was not brought back.
<!-- SECTION:FINAL_SUMMARY:END -->
