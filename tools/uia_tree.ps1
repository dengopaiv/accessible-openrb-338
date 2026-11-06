<#
.SYNOPSIS
    What a screen reader can see of OpenRB-338's window: its UI Automation tree, summarized.

.DESCRIPTION
    Starts OpenRB-338 (by default in --demo mode, which plays no sound and needs no song), waits
    for its main window, asks Windows UI Automation for the window's elements and prints:
    the window's name, the number of elements, the elements grouped by control type and by
    whether they have a name, how many can take the keyboard focus (and how many of those have
    a name), and the names. Then it closes the program it started (and only that one).

    The first UI Automation request wakes AccessKit up inside the program; the element handed
    out before that is a placeholder, so the script asks again with a fresh element until the
    tree is there. This is the measurement of docs/accessibility.md section 1.1.

    Windows only (UI Automation). It needs no screen reader running. The window shows for a few
    seconds and takes the focus; it reads the settings of a throwaway file, so the user's
    ui.toml is not touched.

.PARAMETER Exe
    The OpenRB-338 program. Default: target\release\OpenRB-338.exe under the repository.

.PARAMETER Arguments
    The program's arguments. Default: --demo --size 624x510.

.PARAMETER Dump
    Also print every element: its control type, its name and whether it can take the focus.

.PARAMETER ScreenReader
    The screen reader mode setting the program starts with, written into the throwaway
    settings file: auto (the default: on while Windows reports a screen reader running, as
    NVDA does), on (the tree a screen reader user gets, with the section groups of task-188)
    or off (the tree with the mode off, also on a machine with NVDA running).

.EXAMPLE
    cargo build --release -p orb-ui
    pwsh tools/uia_tree.ps1

.EXAMPLE
    pwsh tools/uia_tree.ps1 -ScreenReader on

.EXAMPLE
    pwsh tools/uia_tree.ps1 -Arguments 'C:\songs\a.rbs' -Dump
#>
param(
    [string]$Exe = (Join-Path $PSScriptRoot '..\target\release\OpenRB-338.exe'),
    [string[]]$Arguments = @('--demo', '--size', '624x510'),
    [switch]$Dump,
    [ValidateSet('auto', 'on', 'off')]
    [string]$ScreenReader = 'auto'
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Exe)) {
    throw "no program at $Exe (build it: cargo build --release -p orb-ui)"
}
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public static class OrbUiaWin {
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32", CharSet = CharSet.Unicode)]
  static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  // The visible top-level window of class `cls` that process `pid` owns, or zero.
  public static IntPtr Visible(uint pid, string cls) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p != pid || !IsWindowVisible(h)) return true;
      var b = new StringBuilder(256); GetClassName(h, b, 256);
      if (b.ToString() != cls) return true;
      found = h; return false;
    }, IntPtr.Zero);
    return found;
  }
}
'@

$config = Join-Path ([IO.Path]::GetTempPath()) "orb-uia-tree-$PID.toml"
$oldConfig = $env:ORB_UI_CONFIG
$env:ORB_UI_CONFIG = $config
if ($ScreenReader -ne 'auto') {
    # UTF-8 without a byte order mark, in any PowerShell
    [IO.File]::WriteAllText($config, "screen_reader = `"$ScreenReader`"`n")
}
$p = Start-Process -FilePath $Exe -ArgumentList $Arguments -PassThru
try {
    # The main window is winit's visible "Window Class" window. The process also has a hidden
    # window of that class and title, and windows of wgpu, the IME and winit's event target;
    # none of those is it. It turns visible only after the first frame is drawn (about 10 s on
    # a first start).
    $h = [IntPtr]::Zero
    $deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $deadline -and $h -eq [IntPtr]::Zero) {
        Start-Sleep -Milliseconds 250
        if ($p.HasExited) { throw "OpenRB-338 exited with code $($p.ExitCode)" }
        $h = [OrbUiaWin]::Visible([uint32]$p.Id, 'Window Class')
    }
    if ($h -eq [IntPtr]::Zero) { throw 'the main window did not show within 30 s' }

    $ae = [System.Windows.Automation.AutomationElement]
    $scope = [System.Windows.Automation.TreeScope]::Descendants
    $all = [System.Windows.Automation.Condition]::TrueCondition
    # The first request activates AccessKit and gets the placeholder.
    $null = $ae::FromHandle($h).FindAll($scope, $all)
    $root = $null
    $found = $null
    for ($t = 0; $t -lt 10; $t++) {
        Start-Sleep -Milliseconds 500
        $root = $ae::FromHandle($h)
        $found = $root.FindAll($scope, $all)
        if ($found.Count -gt 0) { break }
    }

    $rows = foreach ($e in $found) {
        $c = $e.Current
        [pscustomobject]@{
            Type      = $c.ControlType.ProgrammaticName -replace '^ControlType\.', ''
            Name      = $c.Name
            Named     = [bool]$c.Name
            Focusable = $c.IsKeyboardFocusable
        }
    }
    "window: '$($root.Current.Name)'"
    "elements: $(@($rows).Count)"
    'by type, named:'
    $rows | Group-Object Type, Named | Sort-Object Count -Descending |
        ForEach-Object { '{0,6}  {1}' -f $_.Count, $_.Name }
    $focusable = @($rows | Where-Object Focusable)
    "can take the focus: $($focusable.Count), of them with a name: " +
        @($focusable | Where-Object Named).Count
    'names: ' + ((@($rows | Where-Object Named) | ForEach-Object Name) -join ' | ')
    if ($Dump) {
        ''
        $rows | Format-Table Type, Focusable, Name -AutoSize | Out-String -Width 200
    }
}
finally {
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
    $env:ORB_UI_CONFIG = $oldConfig
    Remove-Item -LiteralPath $config -ErrorAction SilentlyContinue
}
