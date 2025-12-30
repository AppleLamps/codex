# Codex Web UI Development Startup Script (Windows PowerShell)
# This script sets up the environment and starts the development server.

$ErrorActionPreference = "Stop"

$ScriptDir = $PSScriptRoot
$CodexBinary = Join-Path $ScriptDir "codex-rs\target\debug\codex.exe"
$CodexWebDir = Join-Path $ScriptDir "codex-web"

Write-Host ""
Write-Host "================================================================"
Write-Host "                    CODEX WEB UI                                "
Write-Host "================================================================"
Write-Host ""

# Check if codex binary exists
if (-not (Test-Path $CodexBinary)) {
    Write-Host "X Codex binary not found at: $CodexBinary"
    Write-Host ""
    Write-Host "Building codex binary..."
    Write-Host ""
    Push-Location (Join-Path $ScriptDir "codex-rs")
    cargo build -p codex-cli
    Pop-Location
    Write-Host ""
}

Write-Host "[OK] Codex binary: $CodexBinary"

# Check login status
Write-Host ""
Write-Host "Checking authentication status..."
$ErrorActionPreference = "SilentlyContinue"
& $CodexBinary login status *> $null
$loginSuccess = $LASTEXITCODE -eq 0
$ErrorActionPreference = "Stop"

if (-not $loginSuccess) {
    Write-Host ""
    Write-Host "================================================================"
    Write-Host "                 AUTHENTICATION REQUIRED                        "
    Write-Host "================================================================"
    Write-Host ""
    Write-Host "Codex requires authentication. Choose one of the following:"
    Write-Host ""
    Write-Host "  Option 1: Login with API Key"
    Write-Host "    `$env:OPENAI_API_KEY='your-api-key'"
    Write-Host "    echo `$env:OPENAI_API_KEY | & '$CodexBinary' login --with-api-key"
    Write-Host ""
    Write-Host "  Option 2: Interactive Device Login"
    Write-Host "    & '$CodexBinary' login --device-auth"
    Write-Host ""
    Write-Host "After authenticating, run this script again."
    exit 1
}

Write-Host "[OK] Authentication: OK"

# Check if node_modules exists
if (-not (Test-Path (Join-Path $CodexWebDir "node_modules"))) {
    Write-Host ""
    Write-Host "Installing dependencies..."
    Push-Location $CodexWebDir
    npm install
    Pop-Location
}

Write-Host "[OK] Dependencies: OK"
Write-Host ""
Write-Host "Starting development server..."
Write-Host "----------------------------------------------------------------"
Write-Host ""

# Export the codex binary path
$env:CODEX_PATH = $CodexBinary

# Start the Next.js development server
Push-Location $CodexWebDir
npm run dev
