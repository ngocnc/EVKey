# Installs the EVKey TSF text service. Expects `cargo build -p evkey-tsf --release` output.
# Copies evkey_tsf.dll into a new versioned folder (the old DLL may be locked by running apps),
# unregisters the previously registered DLL, then registers the new one.
$ErrorActionPreference = 'Stop'
$clsid = '{B5F8D54F-6BAD-46D4-8DBB-FFE03BF4D2F4}'
$logPath = Join-Path $env:TEMP 'evkey-install.log'

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin = (New-Object Security.Principal.WindowsPrincipal $identity).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Remove-Item $logPath -ErrorAction SilentlyContinue
    $self = $MyInvocation.MyCommand.Path
    $p = Start-Process powershell -Verb RunAs -Wait -PassThru -ArgumentList @(
        '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$self`"")
    if (Test-Path $logPath) { Get-Content $logPath }
    else { Write-Host "Elevated install produced no log (UAC declined?). Exit code: $($p.ExitCode)" }
    exit $p.ExitCode
}

try {
    Start-Transcript -Path $logPath -Force | Out-Null
    $repo = Split-Path -Parent $PSScriptRoot
    $src = Join-Path $repo 'target/release/evkey_tsf.dll'
    if (-not (Test-Path $src)) { throw "Missing $src. Run: cargo build -p evkey-tsf --release" }

    $regKey = "HKLM:\SOFTWARE\Classes\CLSID\$clsid\InprocServer32"
    $prev = (Get-ItemProperty $regKey -ErrorAction SilentlyContinue).'(default)'
    if ($prev -and (Test-Path $prev)) {
        Write-Host "Unregistering previous: $prev"
        $p = Start-Process regsvr32.exe -ArgumentList '/s', '/u', "`"$prev`"" -Wait -PassThru
        if ($p.ExitCode -ne 0) { Write-Warning "regsvr32 /u exit $($p.ExitCode) (continuing)" }
    }

    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $dir = Join-Path 'C:\Program Files\EVKey' $stamp
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    $dll = Join-Path $dir 'evkey_tsf.dll'
    Copy-Item $src $dll
    Write-Host "Copied to $dll"

    $p = Start-Process regsvr32.exe -ArgumentList '/s', "`"$dll`"" -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw "regsvr32 failed with exit code $($p.ExitCode)" }
    $now = (Get-ItemProperty $regKey).'(default)'
    Write-Host "Registered. InprocServer32 = $now"

    Write-Host ''
    Write-Host 'Switch to EVKey: Win+Space or the language bar, pick "EVKey" (en-US).'
    Write-Host 'For tests: scripts/activate.ps1'
    Write-Host 'Log: %LOCALAPPDATA%\EVKey\tsf.log (only when %LOCALAPPDATA%\EVKey\debug exists)'
    $code = 0
} catch {
    Write-Host "INSTALL FAILED: $_"
    $code = 1
} finally {
    Stop-Transcript | Out-Null
}
exit $code
