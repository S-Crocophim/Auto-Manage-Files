<#
.SYNOPSIS
Menjalankan quality gate yang sama dengan CI.

.PARAMETER Installer
Membangun installer NSIS unsigned sebagai pengganti smoke build executable.

.PARAMETER SkipDesktopBuild
Hanya menjalankan pemeriksaan versi, frontend, format, Clippy, dan Rust test.
#>
param(
    [switch]$Installer,
    [switch]$SkipDesktopBuild
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$CargoCommand = Get-Command cargo -ErrorAction SilentlyContinue

if (-not $CargoCommand) {
    $CargoCandidate = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (-not (Test-Path -LiteralPath $CargoCandidate)) {
        throw "Cargo tidak ditemukan di PATH atau $CargoCandidate"
    }
    $Cargo = $CargoCandidate
}
else {
    $Cargo = $CargoCommand.Source
}

if ($Installer -and $SkipDesktopBuild) {
    throw "-Installer dan -SkipDesktopBuild tidak dapat dipakai bersamaan."
}

Push-Location $ProjectRoot
try {
    node scripts/check-version.mjs
    if ($LASTEXITCODE -ne 0) { throw "Pemeriksaan versi gagal" }

    npm audit --audit-level=high
    if ($LASTEXITCODE -ne 0) { throw "Dependency audit gagal" }

    npm run check
    if ($LASTEXITCODE -ne 0) { throw "Quality gate frontend gagal" }

    & $Cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
    if ($LASTEXITCODE -ne 0) { throw "cargo fmt gagal" }

    & $Cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw "cargo clippy gagal" }

    & $Cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked
    if ($LASTEXITCODE -ne 0) { throw "cargo test gagal" }

    if ($SkipDesktopBuild) {
        Write-Host "Desktop smoke build dilewati." -ForegroundColor Yellow
    }
    elseif ($Installer) {
        npm run tauri -- build --bundles nsis --ci --no-sign -- --locked
        if ($LASTEXITCODE -ne 0) { throw "Build installer NSIS gagal" }
    }
    else {
        npm run tauri -- build --no-bundle --ci -- --locked
        if ($LASTEXITCODE -ne 0) { throw "Tauri production smoke build gagal" }
    }
}
finally {
    Pop-Location
}

Write-Host "Semua pemeriksaan lulus." -ForegroundColor Green
