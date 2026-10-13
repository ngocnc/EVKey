# Unregisters the EVKey TSF text service and deletes installed copies that are not locked.
$ErrorActionPreference = 'Stop'
$clsid = '{B5F8D54F-6BAD-46D4-8DBB-FFE03BF4D2F4}'
$logPath = Join-Path $env:TEMP 'evkey-uninstall.log'

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin = (New-Object Security.Principal.WindowsPrincipal $identity).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Remove-Item $logPath -ErrorAction SilentlyContinue
    $self = $MyInvocation.MyCommand.Path
    $p = Start-Process powershell -Verb RunAs -Wait -PassThru -ArgumentList @(
        '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$self`"")
    if (Test-Path $logPath) { Get-Content $logPath }
    else { Write-Host "Elevated uninstall produced no log (UAC declined?). Exit code: $($p.ExitCode)" }
    exit $p.ExitCode
}

try {
    Start-Transcript -Path $logPath -Force | Out-Null
    $regKey = "HKLM:\SOFTWARE\Classes\CLSID\$clsid\InprocServer32"
    $cur = (Get-ItemProperty $regKey -ErrorAction SilentlyContinue).'(default)'
    if ($cur -and (Test-Path $cur)) {
        Write-Host "Unregistering: $cur"
        $p = Start-Process regsvr32.exe -ArgumentList '/s', '/u', "`"$cur`"" -Wait -PassThru
        if ($p.ExitCode -ne 0) { Write-Warning "regsvr32 /u exit $($p.ExitCode)" }
    } else {
        Write-Host 'No registered DLL found at InprocServer32.'
    }
    Remove-Item "HKLM:\SOFTWARE\Classes\CLSID\$clsid" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item "HKLM:\SOFTWARE\Microsoft\CTF\TIP\$clsid" -Recurse -Force -ErrorAction SilentlyContinue

    # If the user's default input override points at EVKey, clear it so Windows falls back to en-US.
    $ovr = Get-WinDefaultInputMethodOverride -ErrorAction SilentlyContinue
    if ($ovr -and ($ovr.InputMethodTip -like "*$clsid*")) {
        Set-WinDefaultInputMethodOverride -InputTip '0409:00000409'
        Write-Host 'Reset default input override from EVKey to 0409:00000409.'
    }

    $root = 'C:\Program Files\EVKey'
    $left = @()
    if (Test-Path $root) {
        Get-ChildItem $root -Directory | ForEach-Object {
            Remove-Item $_.FullName -Recurse -Force -ErrorAction SilentlyContinue
            if (Test-Path $_.FullName) { $left += $_.FullName }
        }
        if (-not $left -and -not (Get-ChildItem $root -Force)) {
            Remove-Item $root -Force -ErrorAction SilentlyContinue
        }
    }
    if ($left) {
        Write-Host 'Locked, not deleted (close the apps using them, then delete manually):'
        $left | ForEach-Object { Write-Host "  $_" }
    } else {
        Write-Host 'Deleted all installed copies.'
    }
    $code = 0
} catch {
    Write-Host "UNINSTALL FAILED: $_"
    $code = 1
} finally {
    Stop-Transcript | Out-Null
}
exit $code
