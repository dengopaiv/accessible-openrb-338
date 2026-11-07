<#
.SYNOPSIS
    Starts OpenRB-338 a few times and writes one plain-text report of what happened and of the
    system, for a start that fails on a Windows PC (task-215).

.DESCRIPTION
    OpenRB-338 often did not start on the user's Windows 10 PC (2026-11-07). This script is the
    collecting half of the Windows debug kit (scripts/win-debug-kit.ps1 makes the kit); the
    program's half is its run log (task-214, docs/building.md "The run log").

    For each run it starts OpenRB-338 with RUST_BACKTRACE=full and ORB_LOG=info, watches the
    run log the program writes, and names the outcome:
    - started: the first frame was drawn; the script waits a few seconds more, then closes the
      window as a user would;
    - start error (exit code 1), panic (exit code 101), bad arguments (exit code 2);
    - native crash: an exit code that is a Windows exception code, named (access violation ...);
    - could not be started: Windows refused to start it (SmartScreen, AppLocker, an antivirus);
    - could not be loaded: a DLL was not found (0xC0000135): Windows could not load the program
      at all, as when the Visual C++ runtime is missing;
    - hang: no frame after the timeout; the script saves a dump with cdb if it finds cdb, then
      stops the process.

    The report (diag-reports\win_diag-YYYYMMDD-HHMMSS.txt beside the program) holds the outcome
    of each run with its standard error, its run log and its crash log; the system (Windows
    edition, version and build, processor, memory, graphics adapters with driver version and
    date, audio endpoints, running screen readers, antivirus products, the Visual C++ runtime,
    whether the program file carries the downloaded-from-the-internet mark); the Application
    log's Application Error, Application Hang and Windows Error Reporting events of
    OpenRB-338.exe during the session; and the crash dumps of the session, analyzed with cdb
    when it is installed (Windows SDK, Debugging Tools for Windows). It is sentences and
    labeled lines, for reading with a screen reader. The console shows only the start, one line
    per run and the report's path.

    The runs use the user's own settings, so that they start as the program does from the
    Start menu. Nothing here goes online: cdb is given only the folder of the program's symbols
    (and whatever _NT_SYMBOL_PATH the user has set).

    Runs on Windows PowerShell 5.1 (what Windows 10 has) and PowerShell 7. From a console:
        powershell -ExecutionPolicy Bypass -File win_diag.ps1

.PARAMETER Runs
    How many times to start the program. Default 3, because the failure comes and goes. 0 runs
    nothing (with -EnableDumps or -DisableDumps alone).

.PARAMETER Timeout
    Seconds to wait for the first frame before a run counts as a hang. Default 30.

.PARAMETER Backend
    The graphics backend wgpu must use (WGPU_BACKEND): dx12, vulkan or opengl. Default: wgpu's
    own choice.

.PARAMETER Demo
    Starts the program with --demo: the panel without sound (no audio device is opened).

.PARAMETER Song
    A song to open (its path; spaces need no extra quotes).

.PARAMETER ExtraArgs
    More arguments for the program, one string as it would be typed after the program's name,
    for example '--size 624x510' or '--debug-start-error'.

.PARAMETER LogLevel
    ORB_LOG for the runs: off, error, warn, info (the default), debug or trace.

.PARAMETER Exe
    The program. Default: OpenRB-338.exe beside this script (the kit), else the repository's
    target\diag, then target\release build.

.PARAMETER EnableDumps
    Turns on Windows Error Reporting's LocalDumps for OpenRB-338.exe: a native crash then leaves
    a dump in the dumps folder beside the program (10 at most). Changing it needs administrator
    rights: Windows asks for them. It stays on until -DisableDumps.

.PARAMETER DisableDumps
    Removes the LocalDumps setting of OpenRB-338.exe (asks for administrator rights).

.PARAMETER DumpType
    Full (the default: the whole memory, large but complete) or Mini (the stacks only, small).

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File win_diag.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File win_diag.ps1 -Runs 10 -EnableDumps

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File win_diag.ps1 -Backend dx12 -Demo
#>
param(
    [int]$Runs = 3,
    [int]$Timeout = 30,
    [ValidateSet('', 'dx12', 'vulkan', 'opengl')]
    [string]$Backend = '',
    [switch]$Demo,
    [string]$Song = '',
    [string]$ExtraArgs = '',
    [ValidateSet('off', 'error', 'warn', 'info', 'debug', 'trace')]
    [string]$LogLevel = 'info',
    [string]$Exe = '',
    [switch]$EnableDumps,
    [switch]$DisableDumps,
    [ValidateSet('Full', 'Mini')]
    [string]$DumpType = 'Full',
    # internal: the elevated copy of the script that writes or removes the LocalDumps key
    [string]$SetDumpsFolder = '',
    [switch]$RemoveDumpsKey
)

$ErrorActionPreference = 'Stop'
$DumpsKey = 'HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps\OpenRB-338.exe'

# --- the elevated half: only the registry ---------------------------------------------------
if ($SetDumpsFolder -ne '') {
    New-Item -Path $DumpsKey -Force | Out-Null
    New-ItemProperty -Path $DumpsKey -Name DumpFolder -PropertyType ExpandString `
        -Value $SetDumpsFolder -Force | Out-Null
    New-ItemProperty -Path $DumpsKey -Name DumpCount -PropertyType DWord -Value 10 -Force |
        Out-Null
    $type = 1
    if ($DumpType -eq 'Full') { $type = 2 }
    New-ItemProperty -Path $DumpsKey -Name DumpType -PropertyType DWord -Value $type -Force |
        Out-Null
    exit 0
}
if ($RemoveDumpsKey) {
    if (Test-Path $DumpsKey) { Remove-Item -Path $DumpsKey -Recurse -Force }
    exit 0
}

# --- helpers ----------------------------------------------------------------------------------

function Find-Exe {
    param([string]$Given)
    if ($Given -ne '') { return (Resolve-Path $Given).Path }
    $candidates = @(
        (Join-Path $PSScriptRoot 'OpenRB-338.exe'),
        (Join-Path $PSScriptRoot '..\target\diag\OpenRB-338.exe'),
        (Join-Path $PSScriptRoot '..\target\release\OpenRB-338.exe')
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return (Resolve-Path $c).Path }
    }
    throw ('no OpenRB-338.exe beside this script or in target\diag or target\release; ' +
        'give it with -Exe')
}

# The name of a Windows exception code (an NTSTATUS); the program's own list is
# exception_name in crates/orb-ui/src/runlog.rs.
#
# Compared as hex text: PowerShell reads a literal such as 0xC0000005 as a negative Int32, which
# never equals the UInt32 code.
function Get-StatusName {
    param([uint32]$Code)
    switch ('{0:X8}' -f $Code) {
        'C0000005' { return 'access violation' }
        'C0000006' { return 'in-page error' }
        'C000001D' { return 'illegal instruction' }
        'C0000094' { return 'integer division by zero' }
        'C0000096' { return 'privileged instruction' }
        'C00000FD' { return 'stack overflow' }
        'C0000135' { return 'a DLL was not found (Windows could not load the program)' }
        'C0000139' { return 'a function was not found in a DLL (entry point not found)' }
        'C0000142' { return 'a DLL failed to initialize' }
        'C000007B' { return 'a DLL is of the wrong kind (32-bit and 64-bit mixed)' }
        'C000013A' { return 'stopped with Ctrl+C or by closing its console' }
        'C0000374' { return 'heap corruption' }
        'C0000409' { return 'fail-fast abort (Rust abort, or stack buffer overrun)' }
        '80000003' { return 'breakpoint' }
        'E06D7363' { return 'C++ exception' }
    }
    return 'unknown exception'
}

function ConvertTo-UInt32 {
    param([int]$Value)
    return [BitConverter]::ToUInt32([BitConverter]::GetBytes($Value), 0)
}

# A file's text even while the program has it open for writing.
function Read-Shared {
    param([string]$Path)
    try {
        $fs = [IO.File]::Open($Path, 'Open', 'Read', 'ReadWrite, Delete')
        try {
            $r = New-Object IO.StreamReader($fs)
            return $r.ReadToEnd()
        } finally {
            $fs.Dispose()
        }
    } catch {
        return ''
    }
}

# The text of the run log's end line (`end: ...`, crates/orb-ui/src/runlog.rs), or $null.
function Get-EndText {
    param([string]$Text)
    $m = [regex]::Match($Text, '(?m)^\s*\d+ ms  end: (.*?)\r?$')
    if ($m.Success) { return $m.Groups[1].Value }
    return $null
}

# The last line of a run log, or a note that there is none.
function Get-LastLine {
    param($Log)
    if (-not $Log) { return 'no run log was written' }
    $lines = @((Read-Shared $Log.FullName).TrimEnd() -split "\r?\n")
    return $lines[-1].Trim()
}

# What an exit code says when the program ended by itself.
function Get-ExitKind {
    param([int]$Code)
    switch ($Code) {
        0 { return 'ended normally (exit code 0)' }
        1 { return 'start error (exit code 1)' }
        2 { return 'bad arguments (exit code 2)' }
        101 { return 'panic (exit code 101)' }
    }
    # an exception code has its top bit set: a negative Int32
    if ($Code -lt 0) {
        $u = ConvertTo-UInt32 $Code
        if ($u -eq [uint32]::Parse('C0000135', 'HexNumber')) {
            return 'could not be loaded: a DLL was not found (0xC0000135)'
        }
        return ('native crash: {0} (0x{1:X8})' -f (Get-StatusName $u), $u)
    }
    return "ended with exit code $Code"
}

function Get-LogDir {
    if ($env:ORB_UI_LOG_DIR) { return $env:ORB_UI_LOG_DIR }
    return (Join-Path $env:LOCALAPPDATA 'OpenRB-338\logs')
}

# The names of the logs of a kind (run, crash) in the folder now.
function Get-LogNames {
    param([string]$Dir, [string]$Prefix)
    if (-not (Test-Path $Dir)) { return @() }
    return @(Get-ChildItem -Path $Dir -Filter "$Prefix-*.log" -File | ForEach-Object { $_.Name })
}

# The logs of a kind whose names were not in $Before (taken just before the run): the run's own,
# whatever the timing.
function Get-NewLogs {
    param([string]$Dir, [string]$Prefix, [string[]]$Before)
    if (-not (Test-Path $Dir)) { return @() }
    return @(Get-ChildItem -Path $Dir -Filter "$Prefix-*.log" -File |
            Where-Object { $Before -notcontains $_.Name } |
            Sort-Object CreationTime)
}

# Runs a native program with its standard error merged, under 'Continue': in Windows
# PowerShell 5.1 a native program's stderr line is an error record, which 'Stop' turns into a
# terminating error.
function Invoke-Native {
    param([string]$Program, [string[]]$Arguments)
    $ErrorActionPreference = 'Continue'
    try {
        return (& $Program @Arguments 2>&1 | Out-String)
    } catch {
        return "could not run ${Program}: $($_.Exception.Message)"
    }
}

function Find-Cdb {
    $cmd = Get-Command cdb.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $roots = @(${env:ProgramFiles(x86)}, $env:ProgramFiles) | Where-Object { $_ }
    foreach ($r in $roots) {
        $p = Join-Path $r 'Windows Kits\10\Debuggers\x64\cdb.exe'
        if (Test-Path $p) { return $p }
    }
    return $null
}

function Test-Admin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($id)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Runs this script elevated with the given arguments and waits.
function Invoke-Elevated {
    param([string[]]$Arguments)
    $all = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSCommandPath`"") +
        $Arguments
    if (Test-Admin) {
        $p = Start-Process -FilePath 'powershell.exe' -ArgumentList $all -Wait -PassThru `
            -WindowStyle Hidden
    } else {
        $p = Start-Process -FilePath 'powershell.exe' -ArgumentList $all -Wait -PassThru `
            -Verb RunAs -WindowStyle Hidden
    }
    return $p.ExitCode
}


$report = New-Object System.Text.StringBuilder
function Add-Line {
    param([string]$Text = '')
    [void]$report.AppendLine($Text)
}

# --- the session -------------------------------------------------------------------------------

$Exe = Find-Exe $Exe
$exeDir = Split-Path $Exe
$reportDir = Join-Path $exeDir 'diag-reports'
$dumpDir = Join-Path $exeDir 'dumps'
$logDir = Get-LogDir
$sessionStart = Get-Date
$stamp = $sessionStart.ToString('yyyyMMdd-HHmmss')
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
$reportPath = Join-Path $reportDir "win_diag-$stamp.txt"

Write-Host "OpenRB-338 diagnostics: $Runs run(s) of $Exe"

$dumpsNote = ''
if ($EnableDumps) {
    New-Item -ItemType Directory -Force -Path $dumpDir | Out-Null
    Write-Host 'Turning on crash dumps; Windows asks for administrator rights.'
    try {
        [void](Invoke-Elevated @('-SetDumpsFolder', "`"$dumpDir`"", '-DumpType', $DumpType))
    } catch {
        $dumpsNote = "Turning on crash dumps failed: $($_.Exception.Message)"
    }
}
if ($DisableDumps) {
    Write-Host 'Turning off crash dumps; Windows asks for administrator rights.'
    try {
        [void](Invoke-Elevated @('-RemoveDumpsKey'))
    } catch {
        $dumpsNote = "Turning off crash dumps failed: $($_.Exception.Message)"
    }
}

$parts = @()
if ($Demo) { $parts += '--demo' }
if ($ExtraArgs.Trim() -ne '') { $parts += $ExtraArgs.Trim() }
if ($Song -ne '') { $parts += ('"' + $Song + '"') }
$argText = $parts -join ' '

$saved = @{}
foreach ($name in 'RUST_BACKTRACE', 'ORB_LOG', 'WGPU_BACKEND') {
    $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
}
$env:RUST_BACKTRACE = 'full'
$env:ORB_LOG = $LogLevel
if ($Backend -ne '') { $env:WGPU_BACKEND = $Backend }

$cdb = Find-Cdb
$results = @()
try {
    for ($i = 1; $i -le $Runs; $i++) {
        $others = @(Get-Process -Name 'OpenRB-338' -ErrorAction SilentlyContinue).Count
        $errFile = Join-Path $reportDir "run-$stamp-$i.stderr.txt"
        $outFile = Join-Path $reportDir "run-$stamp-$i.stdout.txt"
        $runsBefore = Get-LogNames $logDir 'run'
        $crashesBefore = Get-LogNames $logDir 'crash'
        $start = Get-Date
        $startArgs = @{
            FilePath               = $Exe
            PassThru               = $true
            RedirectStandardError  = $errFile
            RedirectStandardOutput = $outFile
            WorkingDirectory       = $exeDir
        }
        if ($argText -ne '') { $startArgs['ArgumentList'] = $argText }
        try {
            $p = Start-Process @startArgs
        } catch {
            # blocked by SmartScreen, AppLocker or an antivirus, or the file is gone: still a
            # result, and the report must still be written
            $outcome = "could not be started: $($_.Exception.Message)"
            $results += [pscustomobject]@{
                Number    = $i
                Start     = $start
                Outcome   = $outcome
                Detail    = ''
                ExitCode  = $null
                Killed    = $false
                Others    = $others
                ErrFile   = $errFile
                OutFile   = $outFile
                RunLog    = $null
                CrashLogs = @()
            }
            Write-Host "Run ${i}: $outcome"
            continue
        }
        # keeps the exit code readable after the process ends (a quirk of Process)
        $null = $p.Handle
        $frameMs = $null
        $endText = $null
        $runLog = $null
        $closed = $false
        $hang = $false
        $killed = $false
        $notes = @()
        $deadline = $start.AddSeconds($Timeout)
        # until the first frame, the run log's end line, the process's end or the timeout
        while ($true) {
            if ($null -eq $runLog) {
                $runLog = Get-NewLogs $logDir 'run' $runsBefore | Select-Object -First 1
            }
            if ($p.HasExited) { break }
            if ($runLog) {
                $text = Read-Shared $runLog.FullName
                $endText = Get-EndText $text
                if ($endText) { break }
                $m = [regex]::Match($text, '(?m)^\s*(\d+) ms  first frame drawn')
                if ($m.Success) {
                    $frameMs = [int]$m.Groups[1].Value
                    break
                }
            }
            if ((Get-Date) -gt $deadline) { break }
            Start-Sleep -Milliseconds 250
        }
        if ($null -ne $frameMs -and -not $p.WaitForExit(3000)) {
            # the window is up and stayed up for 3 s, unless a panic's box is showing
            $endText = Get-EndText (Read-Shared $runLog.FullName)
            if (-not $endText) {
                $p.Refresh()
                [void]$p.CloseMainWindow()
                $closed = $true
                if ($p.WaitForExit(10000)) {
                    $notes += 'closed by the script'
                } else {
                    Stop-Process -Id $p.Id -Force
                    [void]$p.WaitForExit(5000)
                    $killed = $true
                    $notes += 'did not close when asked; stopped by the script'
                }
            }
        }
        if ($endText -and -not $p.HasExited) {
            # The program wrote its end and is still there: its message box (a start error, a
            # panic) waits for OK, or Windows Error Reporting handles a native crash (its dump
            # and its box). Give that time, then stop it.
            $wait = 5000
            if ($endText -like 'native crash*') { $wait = 60000 }
            if (-not $p.WaitForExit($wait)) {
                Stop-Process -Id $p.Id -Force
                [void]$p.WaitForExit(5000)
                $killed = $true
                $notes += 'its message box was still open; stopped by the script'
            }
        }
        if (-not $p.HasExited) {
            $hang = $true
            if ($cdb) {
                New-Item -ItemType Directory -Force -Path $dumpDir | Out-Null
                $hangDump = Join-Path $dumpDir "hang-$stamp-$i.dmp"
                # the commands go in a file (-cf): Windows PowerShell 5.1 does not escape the
                # quotes of a dump path with spaces inside a -c argument
                $cmdFile = Join-Path $dumpDir "hang-$stamp-$i.cdb.txt"
                $commands = ".dump /ma `"$hangDump`"`r`nqd`r`n"
                [IO.File]::WriteAllText($cmdFile, $commands, (New-Object Text.UTF8Encoding($false)))
                [void](Invoke-Native $cdb @('-pv', '-p', "$($p.Id)", '-cf', $cmdFile))
                Remove-Item -Path $cmdFile -ErrorAction SilentlyContinue
                if (Test-Path $hangDump) { $notes += "dump $hangDump" }
            }
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
            [void]$p.WaitForExit(5000)
            $killed = $true
        }
        $code = $null
        if ($p.HasExited) { $code = $p.ExitCode }
        if ($null -eq $runLog) {
            $runLog = Get-NewLogs $logDir 'run' $runsBefore | Select-Object -First 1
        }
        if ($runLog -and -not $endText) { $endText = Get-EndText (Read-Shared $runLog.FullName) }
        $detail = ''
        if ($closed -and $endText -notlike 'panic*') {
            $outcome = 'started'
            $detail = 'first frame after {0:N1} s' -f ($frameMs / 1000.0)
        } elseif ($hang) {
            $outcome = 'hang'
            $last = Get-LastLine $runLog
            $detail = "no frame after $Timeout s; the run log's last line: $last"
        } else {
            if ($killed -or $null -eq $code) {
                $outcome = "the run log's end: $endText"
            } else {
                $outcome = Get-ExitKind $code
                if ($endText) { $detail = "the run log's end: $endText" }
            }
            if ($null -ne $frameMs) {
                $outcome += (' (after its first frame at {0:N1} s)' -f ($frameMs / 1000.0))
            }
        }
        if ($notes.Count -gt 0) {
            if ($detail -ne '') { $detail += '; ' }
            $detail += ($notes -join '; ')
        }
        $crashLogs = Get-NewLogs $logDir 'crash' $crashesBefore
        $results += [pscustomobject]@{
            Number    = $i
            Start     = $start
            Outcome   = $outcome
            Detail    = $detail
            ExitCode  = $code
            Killed    = $killed
            Others    = $others
            ErrFile   = $errFile
            OutFile   = $outFile
            RunLog    = $runLog
            CrashLogs = $crashLogs
        }
        $line = "Run ${i}: $outcome"
        if ($detail -ne '') { $line += "; $detail" }
        Write-Host $line
        Start-Sleep -Milliseconds 500
    }
} finally {
    foreach ($name in $saved.Keys) {
        [Environment]::SetEnvironmentVariable($name, $saved[$name], 'Process')
    }
}

# --- the report --------------------------------------------------------------------------------

Add-Line 'OpenRB-338 diagnostic report'
Add-Line ('Made: {0} (local time) by tools/win_diag.ps1' -f
    $sessionStart.ToString('yyyy-MM-dd HH:mm:ss'))
Add-Line "Program: $Exe"
$backendText = 'wgpu chooses'
if ($Backend -ne '') { $backendText = $Backend }
$argReport = 'none'
if ($argText -ne '') { $argReport = $argText }
Add-Line ("Settings: $Runs run(s), timeout $Timeout s, graphics backend: $backendText, " +
    "ORB_LOG=$LogLevel, arguments: $argReport")
Add-Line "Log folder: $logDir"
if ($dumpsNote -ne '') { Add-Line $dumpsNote }
Add-Line

Add-Line '== Summary =='
if ($results.Count -eq 0) { Add-Line 'No runs.' }
foreach ($r in $results) {
    $s = "Run $($r.Number): $($r.Outcome)"
    if ($r.Detail -ne '') { $s += "; $($r.Detail)" }
    Add-Line "$s."
}
Add-Line

Add-Line '== System =='
try {
    $os = Get-CimInstance Win32_OperatingSystem
    $cv = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
    $release = $cv.DisplayVersion
    if (-not $release) { $release = $cv.ReleaseId }
    Add-Line ('Windows: {0}, version {1}, build {2}.{3}, {4}' -f $os.Caption, $release,
        $os.BuildNumber, $cv.UBR, $os.OSArchitecture)
    Add-Line ('Last start of Windows: {0}' -f $os.LastBootUpTime.ToString('yyyy-MM-dd HH:mm'))
} catch {
    Add-Line "Windows: unknown ($($_.Exception.Message))"
}
try {
    foreach ($c in Get-CimInstance Win32_Processor) {
        Add-Line ('Processor: {0}, {1} cores, {2} threads' -f $c.Name.Trim(), $c.NumberOfCores,
            $c.NumberOfLogicalProcessors)
    }
    $cs = Get-CimInstance Win32_ComputerSystem
    Add-Line ('Memory: {0:N1} GB; computer: {1} {2}' -f ($cs.TotalPhysicalMemory / 1GB),
        $cs.Manufacturer, $cs.Model)
} catch {
    Add-Line "Processor and memory: unknown ($($_.Exception.Message))"
}
try {
    $n = 0
    foreach ($g in Get-CimInstance Win32_VideoController) {
        $n++
        $date = 'unknown'
        if ($g.DriverDate) { $date = $g.DriverDate.ToString('yyyy-MM-dd') }
        $mode = ''
        if ($g.CurrentHorizontalResolution) {
            $mode = ('; screen {0} by {1} at {2} Hz' -f $g.CurrentHorizontalResolution,
                $g.CurrentVerticalResolution, $g.CurrentRefreshRate)
        }
        Add-Line ('Graphics adapter {0}: {1}; driver version {2}, dated {3}; status {4}{5}' -f
            $n, $g.Name, $g.DriverVersion, $date, $g.Status, $mode)
    }
    if ($n -eq 0) { Add-Line 'Graphics adapters: none found' }
} catch {
    Add-Line "Graphics adapters: unknown ($($_.Exception.Message))"
}
try {
    $endpoints = @(Get-PnpDevice -Class AudioEndpoint -PresentOnly -ErrorAction Stop)
    foreach ($e in $endpoints) {
        Add-Line ('Audio endpoint: {0}; status {1}' -f $e.FriendlyName, $e.Status)
    }
    if ($endpoints.Count -eq 0) { Add-Line 'Audio endpoints: none present' }
} catch {
    Add-Line "Audio endpoints: unknown ($($_.Exception.Message))"
}
$readers = @()
foreach ($pair in @(@('nvda', 'NVDA'), @('jfw', 'JAWS'), @('Narrator', 'Narrator'),
        @('ZoomText', 'ZoomText'))) {
    if (Get-Process -Name $pair[0] -ErrorAction SilentlyContinue) { $readers += $pair[1] }
}
if ($readers.Count -gt 0) {
    Add-Line ('Screen readers running: ' + ($readers -join ', '))
} else {
    Add-Line 'Screen readers running: none found'
}
try {
    $av = @(Get-CimInstance -Namespace root/SecurityCenter2 -ClassName AntivirusProduct `
            -ErrorAction Stop)
    if ($av.Count -gt 0) {
        Add-Line ('Antivirus products: ' + (($av | ForEach-Object { $_.displayName }) -join ', '))
    } else {
        Add-Line 'Antivirus products: none registered'
    }
} catch {
    Add-Line 'Antivirus products: unknown'
}
$vc = Join-Path $env:SystemRoot 'System32\vcruntime140.dll'
if (Test-Path $vc) {
    Add-Line ('Visual C++ runtime: vcruntime140.dll version {0}' -f
        (Get-Item $vc).VersionInfo.FileVersion)
} else {
    Add-Line ('Visual C++ runtime: vcruntime140.dll is MISSING from System32; OpenRB-338 needs ' +
        'it and cannot start without it (install the Microsoft Visual C++ Redistributable, x64)')
}
try {
    $zone = Get-Item -Path $Exe -Stream Zone.Identifier -ErrorAction Stop
    if ($zone) {
        Add-Line ('Downloaded mark: the program file carries the mark of a file from the ' +
            'internet, so Windows SmartScreen may hold it back when it is started from Explorer')
    }
} catch {
    Add-Line 'Downloaded mark: none on the program file'
}
Add-Line ('Program file: {0:N1} MB, changed {1}' -f ((Get-Item $Exe).Length / 1MB),
    (Get-Item $Exe).LastWriteTime.ToString('yyyy-MM-dd HH:mm'))
$key = Get-Item -Path $DumpsKey -ErrorAction SilentlyContinue
if ($key) {
    Add-Line ('Crash dumps: on, into {0} (type {1}, at most {2})' -f
        $key.GetValue('DumpFolder'), $key.GetValue('DumpType'), $key.GetValue('DumpCount'))
} else {
    Add-Line 'Crash dumps: off (turn them on with -EnableDumps)'
}
if ($cdb) { Add-Line "Debugger: $cdb" } else { Add-Line 'Debugger: cdb.exe not found' }
Add-Line

foreach ($r in $results) {
    Add-Line "== Run $($r.Number) =="
    Add-Line ('Started: {0}' -f $r.Start.ToString('HH:mm:ss'))
    $s = "Outcome: $($r.Outcome)"
    if ($r.Detail -ne '') { $s += "; $($r.Detail)" }
    Add-Line $s
    if ($r.Killed) {
        Add-Line 'Exit code: none of its own; the script stopped the process'
    } elseif ($null -ne $r.ExitCode) {
        Add-Line ('Exit code: {0} (0x{1:X8})' -f $r.ExitCode, (ConvertTo-UInt32 $r.ExitCode))
    }
    if ($r.Others -gt 0) {
        Add-Line "Other OpenRB-338 processes running before this run: $($r.Others)"
    }
    $err = ''
    if (Test-Path $r.ErrFile) { $err = (Read-Shared $r.ErrFile).TrimEnd() }
    if ($err -ne '') {
        Add-Line 'Standard error:'
        Add-Line $err
    } else {
        Add-Line 'Standard error: empty'
    }
    if ($r.RunLog) {
        Add-Line "Run log: $($r.RunLog.FullName)"
        $text = (Read-Shared $r.RunLog.FullName).TrimEnd()
        Add-Line $text
        if ($text -notmatch '(?m)^\s*\d+ ms  end: ') {
            Add-Line ('The run log has no end line: the program was stopped, hung, or ended in ' +
                'a way it could not write down (a fail-fast abort).')
        }
    } else {
        Add-Line ('Run log: none was written. The program ended before its first line of code ' +
            'ran (a missing DLL, or blocked by security software), or the log folder could not ' +
            'be written.')
    }
    foreach ($c in $r.CrashLogs) {
        Add-Line "Crash log: $($c.FullName)"
        Add-Line (Read-Shared $c.FullName).TrimEnd()
    }
    Add-Line
    foreach ($f in $r.ErrFile, $r.OutFile) {
        Remove-Item -Path $f -ErrorAction SilentlyContinue
    }
}

Add-Line '== Windows event log =='
try {
    $filter = @{ LogName = 'Application'; StartTime = $sessionStart }
    $events = @(Get-WinEvent -FilterHashtable $filter -ErrorAction Stop |
            Where-Object {
                ($_.ProviderName -eq 'Application Error' -or
                $_.ProviderName -eq 'Application Hang' -or
                $_.ProviderName -eq 'Windows Error Reporting') -and
                $_.Message -match 'OpenRB-338'
            })
} catch {
    $events = @()
}
if ($events.Count -eq 0) {
    Add-Line ('No Application Error, Application Hang or Windows Error Reporting event of ' +
        'OpenRB-338 during the session.')
}
foreach ($e in ($events | Sort-Object TimeCreated)) {
    Add-Line ('{0} {1}, event {2}:' -f $e.TimeCreated.ToString('HH:mm:ss'), $e.ProviderName, $e.Id)
    Add-Line $e.Message.TrimEnd()
    Add-Line
}
Add-Line

Add-Line '== Crash dumps =='
$dumps = @()
if (Test-Path $dumpDir) {
    $dumps = @(Get-ChildItem -Path $dumpDir -Filter '*.dmp' -File |
            Where-Object { $_.LastWriteTime -ge $sessionStart } | Sort-Object LastWriteTime)
}
if ($dumps.Count -eq 0) { Add-Line 'No crash dump of this session.' }
foreach ($d in $dumps) {
    Add-Line ('Dump: {0}, {1:N0} MB' -f $d.FullName, ($d.Length / 1MB))
    if ($cdb) {
        Add-Line 'cdb: !analyze -v and the stack'
        $sym = $exeDir
        if ($env:_NT_SYMBOL_PATH) { $sym = "$exeDir;$env:_NT_SYMBOL_PATH" }
        $out = Invoke-Native $cdb @('-z', $d.FullName, '-y', $sym, '-lines', '-c',
            '!analyze -v; .ecxr; kn 40; q')
        Add-Line $out.TrimEnd()
    }
    Add-Line
}

$utf8 = New-Object System.Text.UTF8Encoding($false)
[IO.File]::WriteAllText($reportPath, $report.ToString(), $utf8)
Write-Host "Report: $reportPath"
