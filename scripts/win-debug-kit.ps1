<#
.SYNOPSIS
    Makes the Windows debug kit (task-215): OpenRB-338 and orb built with symbols, their .pdb
    files, tools/win_diag.ps1 and a README, in dist\debug-kit and a zip.

.DESCRIPTION
    For a Windows PC where OpenRB-338 does not start (the user's Windows 10 PC, 2026-11-07).
    The two programs are built with the cargo profile `diag` (Cargo.toml: the release build with
    line tables, not stripped), so a panic's backtrace and a crash dump name functions and
    lines. The packages of scripts/package.sh are not changed (the dist profile, task-200).

    Copy the zip to the other PC, unpack it, and run win_diag.ps1 there (README.txt and
    docs/building.md "Debugging OpenRB-338 on another Windows PC").

    Needs the Rust toolchain of the repository (MSVC, x86_64). Runs on Windows PowerShell 5.1
    and PowerShell 7.

.PARAMETER Out
    The kit folder. Default: dist\debug-kit under the repository. Emptied first.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\win-debug-kit.ps1
#>
param(
    [string]$Out = ''
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ($Out -eq '') { $Out = Join-Path $root 'dist\debug-kit' }

Push-Location $root
try {
    & cargo build --profile diag --locked -p orb-ui -p orb-cli
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
    $hash = (& git rev-parse --short HEAD).Trim()
    # uncommitted changes are in the build too: say so in the kit's name
    if (& git status --porcelain --untracked-files=no) { $hash += '-dirty' }
} finally {
    Pop-Location
}

$bin = Join-Path $root 'target\diag'
if (Test-Path $Out) { Remove-Item -Path $Out -Recurse -Force }
New-Item -ItemType Directory -Force -Path $Out | Out-Null
# the pdb names are the crate's (a hyphen becomes an underscore); the exe names them, and
# debuggers and Rust's backtraces look for them beside the exe
foreach ($f in 'OpenRB-338.exe', 'OpenRB_338.pdb', 'orb.exe', 'orb.pdb') {
    $src = Join-Path $bin $f
    if (-not (Test-Path $src)) { throw "missing $src" }
    Copy-Item -Path $src -Destination $Out
}
Copy-Item -Path (Join-Path $root 'tools\win_diag.ps1') -Destination $Out

$readme = @"
OpenRB-338 debug kit ($hash)

This folder is for finding out why OpenRB-338 does not start on a Windows PC. It holds:
- OpenRB-338.exe and orb.exe, built with symbols (the cargo profile diag);
- OpenRB_338.pdb and orb.pdb, their symbols (keep them beside the programs);
- win_diag.ps1, the script that starts OpenRB-338 a few times and writes a report.

1. Unpack the zip into a folder of your own, for example Documents\OpenRB-338-debug.
2. Open PowerShell in that folder: in Explorer, in the folder, press Alt+D (the address
   bar), type powershell and press Enter.
3. Type:
       powershell -ExecutionPolicy Bypass -File .\win_diag.ps1
   The program window opens and closes three times. The console says one line per run and,
   at the end, the path of the report.
4. Read the report (diag-reports\win_diag-DATE-TIME.txt in this folder) or send it.

More:
- More runs, for a failure that comes and goes:  -Runs 10
- Crash dumps of native crashes (Windows asks for administrator rights once; the setting
  stays until -DisableDumps):  -EnableDumps
- Another graphics backend:  -Backend dx12   (or vulkan, or opengl)
- Without sound:  -Demo
- A song:  -Song 'C:\My Songs\song.rbs'
- Turn crash dumps off again:  -DisableDumps -Runs 0

The program also writes a run log of every start, also when it is started from the Start
menu, in %LOCALAPPDATA%\OpenRB-338\logs (run-DATE-TIME.log; crash-DATE-TIME.log after an
internal error). Those files tell what happened in a start that failed on its own, outside
this script.

If cdb.exe (Windows SDK, Debugging Tools for Windows) is installed, the report includes the
analysis of each crash dump, and a hang is saved as a dump. Nothing in this kit goes online.
"@
$utf8 = New-Object System.Text.UTF8Encoding($false)
[IO.File]::WriteAllText((Join-Path $Out 'README.txt'), ($readme -replace "`r?`n", "`r`n"), $utf8)

$zip = Join-Path (Split-Path $Out) "OpenRB-338-debug-kit-$hash.zip"
if (Test-Path $zip) { Remove-Item -Path $zip -Force }
Compress-Archive -Path (Join-Path $Out '*') -DestinationPath $zip
Write-Host "Kit: $Out"
Write-Host "Zip: $zip"
