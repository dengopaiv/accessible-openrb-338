@echo off
rem Runs win_diag.ps1, the script beside this file, without changing PowerShell's execution
rem policy (task-219). Windows runs no PowerShell script files by default; -ExecutionPolicy
rem Bypass lifts that for this one run only and changes nothing on the PC. A batch file is not
rem subject to that policy, so typing  win_diag  in this folder is enough. The options go on to
rem the script as they are typed, for example:  win_diag -Runs 10 -Backend dx12
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0win_diag.ps1" %*
set "code=%errorlevel%"
rem Started from Explorer or PowerShell (cmd /c with this file), the window would close at once:
rem wait for a key so the report's path can be read. Typed in a command prompt, return at once.
rem The command line is looked at with delayed expansion, not echoed into a pipe, so a folder
rem name with & or quotes in it cannot break the test.
setlocal EnableDelayedExpansion
set "line=!cmdcmdline!"
if /i not "!line:%~nx0=!"=="!line!" (
    echo.
    pause
)
exit /b %code%
