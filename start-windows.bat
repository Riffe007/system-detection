@echo off
echo ========================================
echo    System Monitor - Windows Launcher
echo ========================================
echo.

REM Check if we're in the correct directory
if not exist "package.json" (
    echo Error: package.json not found.
    echo Please run this script from the project root directory.
    pause
    exit /b 1
)

REM Clear Vite cache to avoid permission issues
if exist "node_modules\.vite" (
    echo Clearing Vite cache...
    rmdir /s /q "node_modules\.vite" 2>nul
)

REM Clear npm cache
echo Clearing npm cache...
call npm cache clean --force

REM Install dependencies if needed
if not exist "node_modules" (
    echo Installing dependencies...
    call npm install
)

REM Start the Tauri application
echo Starting Tauri development server...
echo This will open a native application window with real system data.
echo.
call npm run tauri dev

echo.
echo Application closed.
pause 