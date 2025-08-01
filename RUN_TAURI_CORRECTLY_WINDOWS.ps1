# Tauri Launcher for Windows - PowerShell Version
# Run this script if the .bat file doesn't work properly

Write-Host "===================================" -ForegroundColor Cyan
Write-Host "TAURI LAUNCHER - WINDOWS VERSION" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""

# Check if running with proper execution policy
$executionPolicy = Get-ExecutionPolicy
if ($executionPolicy -eq "Restricted") {
    Write-Host "ERROR: PowerShell execution policy is restricted!" -ForegroundColor Red
    Write-Host "To fix this, run PowerShell as Administrator and execute:" -ForegroundColor Yellow
    Write-Host "Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser" -ForegroundColor Green
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

# Function to check if a command exists
function Test-Command {
    param($Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

# Kill existing processes
Write-Host "Stopping any existing Tauri processes..." -ForegroundColor Yellow
Get-Process -Name "system-monitor" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "node" -ErrorAction SilentlyContinue | Where-Object {
    $_.MainWindowTitle -like "*tauri dev*"
} | Stop-Process -Force
Start-Sleep -Seconds 2

Write-Host ""
Write-Host "Checking for required tools..." -ForegroundColor Yellow

# Check for Rust/Cargo
if (-not (Test-Command "cargo")) {
    Write-Host "ERROR: Cargo not found!" -ForegroundColor Red
    Write-Host "Please install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}
Write-Host "[OK] Cargo found" -ForegroundColor Green

# Check for Node.js
if (-not (Test-Command "node")) {
    Write-Host "ERROR: Node.js not found!" -ForegroundColor Red
    Write-Host "Please install Node.js from: https://nodejs.org/" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}
Write-Host "[OK] Node.js found" -ForegroundColor Green

# Check for pnpm
if (-not (Test-Command "pnpm")) {
    Write-Host "WARNING: pnpm not found! Installing pnpm..." -ForegroundColor Yellow
    npm install -g pnpm
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install pnpm!" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
}
Write-Host "[OK] pnpm found" -ForegroundColor Green

# Check for WebView2 (Windows specific requirement)
$webview2 = Get-ItemProperty -Path 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue
if (-not $webview2) {
    Write-Host ""
    Write-Host "WARNING: WebView2 might not be installed!" -ForegroundColor Yellow
    Write-Host "Tauri requires WebView2 on Windows. If the app fails to start:" -ForegroundColor Yellow
    Write-Host "Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "Starting Tauri development server..." -ForegroundColor Green
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "IMPORTANT: A new window will open - that's the Tauri app!" -ForegroundColor Yellow
Write-Host "- The Tauri window shows REAL system data" -ForegroundColor Green
Write-Host "- Browser at localhost:5173 shows MOCK data" -ForegroundColor Red
Write-Host ""

# Change to script directory
Set-Location -Path $PSScriptRoot

# Clear any problematic environment variables
$env:WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS = $null
$env:WEBKIT_FORCE_SANDBOX = $null

# Run Tauri
try {
    pnpm run tauri dev
} catch {
    Write-Host ""
    Write-Host "ERROR: Failed to start Tauri!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Common fixes:" -ForegroundColor Yellow
    Write-Host "1. Make sure you have installed dependencies: pnpm install" -ForegroundColor White
    Write-Host "2. Try running: pnpm run tauri dev directly" -ForegroundColor White
    Write-Host "3. Check for WebView2 installation (required on Windows)" -ForegroundColor White
    Write-Host "4. Make sure Windows Defender isn't blocking the app" -ForegroundColor White
    Write-Host ""
    Read-Host "Press Enter to exit"
}