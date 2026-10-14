# Builds dist/EVKey-Setup-<version>.exe with Inno Setup.
# Usage: scripts/build-installer.ps1 [-NoBuild]
#   -NoBuild  skip `cargo build --release` (uses existing target/release binaries)
param([switch]$NoBuild)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$cargoToml = Get-Content (Join-Path $root 'Cargo.toml') -Raw
if ($cargoToml -notmatch '(?ms)\[workspace\.package\].*?^version\s*=\s*"([^"]+)"') {
    throw 'Workspace version not found in Cargo.toml [workspace.package]'
}
$version = $Matches[1]

if (-not $NoBuild) {
    cargo build --release -p evkey-app
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
}

$bin = Join-Path $root 'target\release'
if (-not (Test-Path (Join-Path $bin 'evkey.exe'))) { throw "Missing $bin\evkey.exe (run without -NoBuild or build first)" }

$iscc = @(
    (Get-Command iscc -ErrorAction SilentlyContinue).Source,
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) { throw 'Inno Setup 6 (ISCC.exe) not found. Install: winget install --id JRSoftware.InnoSetup -e' }

$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
& $iscc "/DAppVersion=$version" "/DBinDir=$bin" "/DOutDir=$dist" (Join-Path $root 'installer\evkey.iss')
if ($LASTEXITCODE -ne 0) { throw "ISCC failed with exit code $LASTEXITCODE" }

$out = Join-Path $dist "EVKey-Setup-$version.exe"
$hash = (Get-FileHash $out -Algorithm SHA256).Hash
Write-Host "Installer: $out"
Write-Host "SHA256:    $hash"
