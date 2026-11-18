# Adds or removes one folder in the PATH for the OpenRB-338 installer and uninstaller (task-111,
# task-237): the user's (HKCU\Environment\Path, scope user, the default) or the machine's
# (HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment\Path, scope machine, for
# the All users install, which runs elevated).
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File path.ps1 add|remove DIR [user|machine]
#
# Why a script and not NSIS code: NSIS strings are 1024 characters (a longer PATH would be cut
# and corrupted), and .NET's SetEnvironmentVariable turns the REG_EXPAND_SZ value into REG_SZ
# (the %USERPROFILE% in it would stop expanding). This reads the value unexpanded and writes it
# back as REG_EXPAND_SZ. The installer broadcasts WM_SETTINGCHANGE afterwards.
#
# Every entry that is not ours, empty ones included, stays exactly as it is. add appends DIR
# (after a ';' unless the value is empty or already ends with one) if it is not in the value;
# remove drops DIR's entry, and with it the ';' after it, or, when it was the last entry, only
# the entry itself (the ';' before it stays). So add then remove gives back the value, except
# that a value that had no ';' at its end gets one.
# Exit code 0: done (or nothing to do); 1: failed.
param(
    [Parameter(Mandatory = $true)][ValidateSet('add', 'remove')][string]$Action,
    [Parameter(Mandatory = $true)][string]$Dir,
    [ValidateSet('user', 'machine')][string]$Scope = 'user'
)
$ErrorActionPreference = 'Stop'
$key = $null
try {
    $dir = $Dir.TrimEnd('\')
    if ($Scope -eq 'machine') {
        # All users install (needs an elevated process): the machine's PATH, in the 64-bit view
        $hive = [Microsoft.Win32.RegistryKey]::OpenBaseKey(
            [Microsoft.Win32.RegistryHive]::LocalMachine, [Microsoft.Win32.RegistryView]::Registry64)
        $key = $hive.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment', $true)
    } else {
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    }
    $old = [string]$key.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    $entries = @($old.Split(';'))
    $isOurs = { param($e) $e -ne '' -and $e.TrimEnd('\') -ieq $dir }
    $found = @($entries | Where-Object { & $isOurs $_ }).Count -gt 0
    $new = $old
    if ($Action -eq 'add' -and -not $found) {
        if ($old -eq '' -or $old.EndsWith(';')) { $new = $old + $dir } else { $new = $old + ';' + $dir }
    } elseif ($Action -eq 'remove' -and $found) {
        $kept = [System.Collections.Generic.List[string]]::new()
        $lastWasOurs = $false
        foreach ($e in $entries) {
            $lastWasOurs = & $isOurs $e
            if (-not $lastWasOurs) { $kept.Add($e) }
        }
        $new = $kept -join ';'
        # the last entry was ours: its separator stays (the join dropped it)
        if ($lastWasOurs -and $kept.Count -gt 0) { $new += ';' }
    }
    if ($new -cne $old) {
        $key.SetValue('Path', $new, [Microsoft.Win32.RegistryValueKind]::ExpandString)
    }
} catch {
    Write-Error $_
    exit 1
} finally {
    if ($null -ne $key) { $key.Close() }
}
