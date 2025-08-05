# System Monitor - Windows PowerShell Launcher
Write-Host "Starting System Monitor Application..." -ForegroundColor Green
Write-Host ""

# Check if we're in the correct directory
if (-not (Test-Path "package.json")) {
    Write-Host "Error: package.json not found. Please run this script from the project root directory." -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Clear Vite cache to avoid permission issues
if (Test-Path "node_modules\.vite") {
    Write-Host "Clearing Vite cache..." -ForegroundColor Yellow
    try {
        Remove-Item -Recurse -Force "node_modules\.vite" -ErrorAction Stop
        Write-Host "Vite cache cleared successfully." -ForegroundColor Green
    } catch {
        Write-Host "Warning: Could not clear Vite cache. Continuing..." -ForegroundColor Yellow
    }
}

# Clear npm cache
Write-Host "Clearing npm cache..." -ForegroundColor Yellow
try {
    npm cache clean --force
    Write-Host "npm cache cleared successfully." -ForegroundColor Green
} catch {
    Write-Host "Warning: Could not clear npm cache. Continuing..." -ForegroundColor Yellow
}

# Install dependencies if needed
if (-not (Test-Path "node_modules")) {
    Write-Host "Installing dependencies..." -ForegroundColor Yellow
    try {
        npm install
        Write-Host "Dependencies installed successfully." -ForegroundColor Green
    } catch {
        Write-Host "Error: Failed to install dependencies." -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
}

# Start the Tauri application
Write-Host "Starting Tauri development server..." -ForegroundColor Green
Write-Host "This will open a native application window with real system data." -ForegroundColor Cyan
Write-Host ""

try {
    npm run tauri dev
} catch {
    Write-Host "Error: Failed to start Tauri application." -ForegroundColor Red
    Write-Host "Please check the error messages above." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Application closed." -ForegroundColor Green
Read-Host "Press Enter to exit" 