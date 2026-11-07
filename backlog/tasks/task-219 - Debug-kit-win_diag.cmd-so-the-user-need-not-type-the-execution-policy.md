---
id: TASK-219
title: 'Debug kit: win_diag.cmd, so the user need not type the execution policy'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-07T14:20:00Z'
updated_date: '2026-11-07T14:26:00Z'
labels:
  - tools
  - docs
milestone: m-7
dependencies: []
references:
  - tools/win_diag.ps1
  - scripts/win-debug-kit.ps1
  - docs/building.md
priority: high
type: enhancement
ordinal: 237000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user's check on the Windows 10 PC (task-218) did not run; the user asked (2026-11-07) "So the diag didn't start because of the execution policy? Do we need to run that policy at all?". Windows 10's default PowerShell execution policy runs no script files: typing `.\win_diag.ps1` fails with "running scripts is disabled on this system", and pressing Enter on a .ps1 in Explorer opens it in Notepad. The kit's README asks the user to type the whole `powershell -ExecutionPolicy Bypass -File .\win_diag.ps1` line, which is easy to get wrong. A batch file is not subject to PowerShell's execution policy.

What: tools/win_diag.cmd, beside win_diag.ps1 in the kit, runs `powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0win_diag.ps1" %*` (the policy lifted for that one run only, nothing changed on the PC), passes the options through (-Runs 10, -Backend dx12 ...), returns the script's exit code, and when started from Explorer (not from a console) waits for a key at the end so the window does not close before the report's path is read. scripts/win-debug-kit.ps1 copies it into the kit with CRLF line endings (cmd.exe misreads LF-only batch files in places; the repository stores text with LF, .gitattributes). The README and docs/building.md tell the user to type `win_diag` in the kit folder, and keep the long line as the alternative.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 scripts/win-debug-kit.ps1 puts win_diag.cmd into the kit with CRLF line endings; the kit's README and docs/building.md say to type win_diag in the kit folder
- [x] #2 Checked here from cmd.exe and from PowerShell: win_diag -Runs 1 with a debug failure gives the right outcome and the report; the arguments arrive (the report's Settings line); the exit code is the script's
- [x] #3 tools/win_diag.cmd runs win_diag.ps1 beside it with -ExecutionPolicy Bypass for that run only, passes its arguments through and returns the script's exit code; started from Explorer or from PowerShell it waits for a key at the end, typed in a command prompt it does not (a batch file cannot tell Explorer from PowerShell: both start it with cmd /c)
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-219-win-diag-cmd (main checkout, the supervisor in the foreground).
Done (2026-11-07): tools/win_diag.cmd (powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0win_diag.ps1" %*, exit /b with the script's code; pauses when %cmdcmdline% contains its own file name, i.e. started with cmd /c by Explorer or PowerShell). scripts/win-debug-kit.ps1 writes it into the kit with CRLF (od: "@echo off\r\n"); .gitattributes checks *.cmd out with CRLF. README and docs/building.md step 3: type win_diag in a command prompt (Alt+D, cmd); .\win_diag where NoDefaultCurrentDirectoryInExePath is set; .\win_diag.cmd in PowerShell (Get-Command .\win_diag resolves to win_diag.ps1 in both 5.1 and 7, checked); the long form stays.
Checks (kit rebuilt from this branch, ORB_UI_NO_CRASH_DIALOG=1, temp log dir and settings):
- command prompt, variable unset (a normal PC): win_diag -Runs 1 -ExtraArgs --debug-start-error -Song "C:\My Songs\a b.rbs" -> "Run 1: start error (exit code 1)", report Settings "arguments: --debug-start-error "C:\My Songs\a b.rbs"", no pause, exit 0.
- command prompt with NoDefaultCurrentDirectoryInExePath=1 (Claude Code's shell): 'win_diag' is not recognized; .\win_diag -Runs 1 -ExtraArgs --debug-panic -> "panic (exit code 101)", no pause, exit 0.
- PowerShell: .\win_diag.cmd -Runs 1 -ExtraArgs --debug-start-error -> the run, "Press any key to continue . . ." at the end, exit 0.
- a bad option (-Backend bogus): ValidateSet error, exit code 1 through win_diag.cmd and directly.

Review round 1 (agent-reviewer, 2026-11-07): merge, no defects. Applied at merge: its note 1 (a folder name with & broke 'echo %cmdcmdline% | find', so the Explorer pause was skipped): win_diag.cmd now tests the command line with delayed expansion and string substitution, no pipe. Checked from a folder named 'a & b' (kit copy, CRLF): PowerShell & .\win_diag.cmd -> run, 'Press any key to continue'; Explorer's exact line cmd.exe /c ""...\a & b\win_diag.cmd" -Runs 1 -ExtraArgs --debug-start-error" (ProcessStartInfo, raw arguments) -> run, 'Press any key'; typed in a command prompt (.\win_diag) -> run, no pause; -Backend bogus -> exit 1. Docs: the task-218 sentence names the task and says no report came back; the kit name's -dirty suffix; one reflowed line.
Merged by the supervisor 2026-11-07: review merge on the first round.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The debug kit gets win_diag.cmd: typing win_diag in a command prompt in the kit folder runs win_diag.ps1 with -ExecutionPolicy Bypass for that run only, so the user no longer types the policy (Windows 10 runs no PowerShell script file by default, which most likely stopped the user's check of the task-217 kit, task-218). Options pass through, the exit code is the script's, and from Explorer or PowerShell it waits for a key at the end. The kit writes it with CRLF; .gitattributes keeps *.cmd CRLF. README and docs/building.md updated. Checked from a command prompt (with and without NoDefaultCurrentDirectoryInExePath), from PowerShell, with a path with spaces and with a bad option. Branch task-219-win-diag-cmd.
<!-- SECTION:FINAL_SUMMARY:END -->
