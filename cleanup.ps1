# System Monitor Cleanup Script
# Removes unnecessary files to clean up the project

Write-Host "🧹 Starting System Monitor cleanup..." -ForegroundColor Green

# Remove duplicate project directory
if (Test-Path "system-monitor") {
    Write-Host "Removing duplicate system-monitor directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force "system-monitor"
}

# Remove empty files
$emptyFiles = @(
    "src/storage/cache.rs",
    "src/storage/database.rs", 
    "src/storage/vector_storage.rs",
    "src/inference/llama_wrapper.h",
    "src/inference/quantization.cpp",
    "src/inference/quantization.h",
    "src/inference/inference_engine.cpp",
    "src/inference/inference_engine.h",
    "src/inference/llama_wrapper.cpp",
    "config.json",
    "CMakeList.txt"
)

foreach ($file in $emptyFiles) {
    if (Test-Path $file) {
        Write-Host "Removing empty file: $file" -ForegroundColor Yellow
        Remove-Item $file
    }
}

# Remove log files
$logFiles = @(
    "tauri.log",
    "tauri-run.log", 
    "tauri-test.log",
    "tauri-dev.log",
    "tauri-new.log",
    "tauri-clean.log",
    "dev_output.log"
)

foreach ($file in $logFiles) {
    if (Test-Path $file) {
        Write-Host "Removing log file: $file" -ForegroundColor Yellow
        Remove-Item $file
    }
}

# Remove redundant scripts
$redundantScripts = @(
    "run-tauri-standalone.sh",
    "run-tauri-no-snap.sh",
    "run-tauri-final.sh", 
    "run-tauri-clean.sh",
    "run-dev.sh",
    "run-direct.sh",
    "run-compiled-app.sh",
    "run-clean.sh",
    "debug-tauri.sh",
    "diagnose.sh",
    "test-tauri.sh",
    "test-backend.sh",
    "test-tauri.html",
    "start-tauri.sh"
)

foreach ($script in $redundantScripts) {
    if (Test-Path $script) {
        Write-Host "Removing redundant script: $script" -ForegroundColor Yellow
        Remove-Item $script
    }
}

# Remove backup files
if (Test-Path "Cargo-backup.toml") {
    Write-Host "Removing backup file: Cargo-backup.toml" -ForegroundColor Yellow
    Remove-Item "Cargo-backup.toml"
}

Write-Host "✅ Cleanup completed!" -ForegroundColor Green
Write-Host "📁 Project is now clean and organized" -ForegroundColor Cyan 