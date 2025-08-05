@echo off
echo ========================================
echo    System Monitor - Windows Launcher
echo ========================================
echo.

REM Check if we're in the right directory
if not exist "package.json" (
    echo Error: package.json not found.
    echo Please run this script from the project root directory.
    pause
    exit /b 1
)

REM Check if pnpm is available
pnpm --version >nul 2>&1
if errorlevel 1 (
    echo Error: pnpm is not installed.
    echo Please install pnpm first: npm install -g pnpm
    pause
    exit /b 1
)

REM Check if Rust is available
cargo --version >nul 2>&1
if errorlevel 1 (
    echo Error: Rust/Cargo is not installed.
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo Dependencies check passed.
echo.
echo Starting System Monitor...
echo This will open a native application window with real system data.
echo.

REM Run Tauri in development mode
pnpm run tauri dev

echo.
echo Application closed.
pause 