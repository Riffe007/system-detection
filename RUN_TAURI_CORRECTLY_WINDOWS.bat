@echo off
echo ===================================
echo TAURI LAUNCHER - WINDOWS VERSION
echo ===================================
echo.

REM Check if running from VS Code terminal
if defined TERM_PROGRAM (
    if "%TERM_PROGRAM%"=="vscode" (
        echo WARNING: You may be running from VS Code terminal.
        echo If you encounter issues, try running from Windows Terminal or CMD directly.
        echo.
    )
)

REM Kill any existing processes
echo Stopping any existing Tauri processes...
taskkill /F /IM system-monitor.exe 2>nul
taskkill /F /IM node.exe /FI "WINDOWTITLE eq *tauri dev*" 2>nul
timeout /t 2 >nul

echo.
echo Checking for required tools...

REM Check for Rust/Cargo
where cargo >nul 2>&1
if errorlevel 1 (
    echo ERROR: Cargo not found! Please install Rust from https://rustup.rs/
    echo.
    pause
    exit /b 1
)
echo [OK] Cargo found

REM Check for Node.js
where node >nul 2>&1
if errorlevel 1 (
    echo ERROR: Node.js not found! Please install Node.js from https://nodejs.org/
    echo.
    pause
    exit /b 1
)
echo [OK] Node.js found

REM Check for pnpm
where pnpm >nul 2>&1
if errorlevel 1 (
    echo ERROR: pnpm not found! Installing pnpm...
    npm install -g pnpm
    if errorlevel 1 (
        echo Failed to install pnpm!
        pause
        exit /b 1
    )
)
echo [OK] pnpm found

echo.
echo Starting Tauri development server...
echo ===================================
echo.
echo IMPORTANT: A new window will open - that's the Tauri app!
echo - The Tauri window shows REAL system data
echo - Browser at localhost:5173 shows MOCK data
echo.

REM Run Tauri
cd /d "%~dp0"
pnpm run tauri dev

if errorlevel 1 (
    echo.
    echo ERROR: Failed to start Tauri!
    echo.
    echo Common fixes:
    echo 1. Make sure you have installed dependencies: pnpm install
    echo 2. Try running: pnpm run tauri dev directly
    echo 3. Check for WebView2 installation (required on Windows)
    echo.
    pause
)