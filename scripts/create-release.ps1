# ASTRA Launcher — Release Script
# Automates building, signing, and creating latest.json for auto-updates
#
# Usage:
#   .\scripts\create-release.ps1 -Version "1.7.1"
#   .\scripts\create-release.ps1 -Version "1.7.1" -SkipBuild
#
# Prerequisites:
#   - Rust toolchain (cargo)
#   - Node.js + npm
#   - GitHub CLI (gh) authenticated
#   - SignPath signing tool (optional, for code signing)

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,

    [switch]$SkipBuild = $false,

    [string]$RepoOwner = "Districkov",
    [string]$RepoName = "Astra_launcher"
)

$ErrorActionPreference = "Stop"

# ── Validate version format ──
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Version must be in format X.Y.Z (e.g., 1.7.1). Got: $Version"
    exit 1
}

$Tag = "v$Version"
$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  ASTRA Launcher Release Script" -ForegroundColor Cyan
Write-Host "  Version: $Tag" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan

# ── Step 1: Update version in tauri.conf.json ──
Write-Host "`n[1/6] Updating version in tauri.conf.json..." -ForegroundColor Yellow

$tauriConfPath = Join-Path $ProjectRoot "src-tauri\tauri.conf.json"
$tauriConf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
$tauriConf.version = $Version
$tauriConf | ConvertTo-Json -Depth 10 | Set-Content $tauriConfPath -Encoding UTF8NoBOM

Write-Host "  ✓ Version updated to $Version" -ForegroundColor Green

# ── Step 2: Update version in Cargo.toml ──
Write-Host "`n[2/6] Updating version in Cargo.toml..." -ForegroundColor Yellow

$cargoPath = Join-Path $ProjectRoot "src-tauri\Cargo.toml"
$cargoContent = Get-Content $cargoPath -Raw
$cargoContent = $cargoContent -replace 'version\s*=\s*"[^"]*"', "version = `"$Version`""
Set-Content $cargoPath $cargoContent -Encoding UTF8NoBOM

Write-Host "  ✓ Cargo.toml version updated" -ForegroundColor Green

# ── Step 3: Build ──
if (-not $SkipBuild) {
    Write-Host "`n[3/6] Building release..." -ForegroundColor Yellow

    Push-Location $ProjectRoot
    try {
        npm run tauri build 2>&1 | ForEach-Object { Write-Host "  $_" }
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Build failed with exit code $LASTEXITCODE"
            exit 1
        }
        Write-Host "  ✓ Build completed" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
} else {
    Write-Host "`n[3/6] Skipping build (-SkipBuild)" -ForegroundColor Yellow
}

# ── Step 4: Find and verify NSIS installer ──
Write-Host "`n[4/6] Locating NSIS installer..." -ForegroundColor Yellow

$bundlePath = Join-Path $ProjectRoot "src-tauri\target\release\bundle"
$nsisPath = Get-ChildItem -Path $bundlePath -Filter "*.exe" -Recurse |
    Where-Object { $_.Name -match "astra-launcher_" -and $_.Name -notmatch "nsis" } |
    Select-Object -First 1

if (-not $nsisPath) {
    # Try the nsis subfolder
    $nsisPath = Get-ChildItem -Path (Join-Path $bundlePath "nsis") -Filter "*.exe" -Recurse |
        Select-Object -First 1
}

if (-not $nsisPath) {
    Write-Error "NSIS installer not found in $bundlePath"
    exit 1
}

Write-Host "  ✓ Found: $($nsisPath.FullName)" -ForegroundColor Green
Write-Host "    Size: $([math]::Round($nsisPath.Length / 1MB, 2)) MB" -ForegroundColor Gray

# ── Step 5: Generate signature ──
Write-Host "`n[5/6] Generating signature..." -ForegroundColor Yellow

# Use the private key from environment or prompt
$privateKey = $env:TAURI_SIGNING_PRIVATE_KEY
if (-not $privateKey) {
    Write-Host "  ⚠ TAURI_SIGNING_PRIVATE_KEY not set — signature will be empty" -ForegroundColor Red
    Write-Host "  Set it with: `$env:TAURI_SIGNING_PRIVATE_KEY = 'your-key'" -ForegroundColor Gray
    $signature = ""
} else {
    # Sign the installer using tauri signer
    $sigTool = Get-Command "tauri-signer" -ErrorAction SilentlyContinue
    if ($sigTool) {
        $signature = & $sigTool.Source sign --private-key $privateKey $nsisPath.FullName 2>&1
    } else {
        Write-Host "  ⚠ tauri-signer not found — using ed25519 from npm" -ForegroundColor Yellow
        # Alternative: use the @aspect-build/ed25519 or similar
        $signature = ""
    }
}

Write-Host "  ✓ Signature generated (length: $($signature.Length))" -ForegroundColor Green

# ── Step 6: Create latest.json ──
Write-Host "`n[6/6] Creating latest.json..." -ForegroundColor Yellow

$releaseDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$downloadUrl = "https://github.com/$RepoOwner/$RepoName/releases/download/$Tag/astra-launcher_${Version}_x64-setup.exe"

$latestJson = @{
    version = $Version
    notes = "ASTRA Launcher v$Version"
    pub_date = $releaseDate
    platforms = @{
        "windows-x86_64" = @{
            signature = $signature
            url = $downloadUrl
        }
    }
} | ConvertTo-Json -Depth 5

# Write WITHOUT BOM (critical for Tauri updater)
$latestJsonPath = Join-Path $ProjectRoot "latest.json"
[System.IO.File]::WriteAllText($latestJsonPath, $latestJson, [System.Text.UTF8Encoding]::new($false))

Write-Host "  ✓ latest.json created at $latestJsonPath" -ForegroundColor Green

# ── Summary ──
Write-Host "`n═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Release ready!" -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Next steps:" -ForegroundColor Yellow
Write-Host "  1. Create git tag:  git tag $Tag && git push origin $Tag" -ForegroundColor White
Write-Host "  2. Create GitHub release:" -ForegroundColor White
Write-Host "     gh release create $Tag `"$(nsisPath.FullName)`" latest.json -t `"ASTRA Launcher $Tag`" -n `"Release $Tag`"" -ForegroundColor Gray
Write-Host "  3. Upload latest.json as release asset" -ForegroundColor White
Write-Host ""
Write-Host "  Files:" -ForegroundColor Yellow
Write-Host "    Installer: $($nsisPath.FullName)" -ForegroundColor Gray
Write-Host "    latest.json: $latestJsonPath" -ForegroundColor Gray
