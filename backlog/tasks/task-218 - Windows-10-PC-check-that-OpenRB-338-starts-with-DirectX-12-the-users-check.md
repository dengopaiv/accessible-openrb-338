---
id: TASK-218
title: 'Windows 10 PC: check that OpenRB-338 starts with DirectX 12 (the user''s check)'
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-07T14:01:00Z'
updated_date: '2026-11-07T14:06:00Z'
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
<!-- SECTION:NOTES:END -->
