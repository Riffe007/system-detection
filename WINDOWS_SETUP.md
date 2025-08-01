# Windows Setup Guide for System Monitor

## Prerequisites

Before running the System Monitor on Windows, ensure you have:

1. **Node.js** (v18 or higher)
   - Download from: https://nodejs.org/
   - Verify: `node --version`

2. **Rust** (latest stable)
   - Download from: https://rustup.rs/
   - Run the installer and follow prompts
   - Verify: `cargo --version`

3. **Microsoft Edge WebView2** (Usually pre-installed on Windows 10/11)
   - If missing, download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
   - Required for Tauri applications on Windows

4. **pnpm** (Will be auto-installed if missing)
   - Or install manually: `npm install -g pnpm`

## Quick Start

### Option 1: Batch Script (Easiest)
1. Double-click `RUN_TAURI_CORRECTLY_WINDOWS.bat`
2. The script will check dependencies and start the app
3. A new window will open with your system information

### Option 2: PowerShell Script (More Features)
1. Right-click on `RUN_TAURI_CORRECTLY_WINDOWS.ps1`
2. Select "Run with PowerShell"
3. If you see an execution policy error:
   - Open PowerShell as Administrator
   - Run: `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`
   - Try again

### Option 3: Manual Command
1. Open Command Prompt or PowerShell
2. Navigate to the project directory
3. Run:
   ```cmd
   pnpm install
   pnpm run tauri dev
   ```

## Important Notes

### Understanding the Two Windows

When you run the application:
1. **Browser Tab** (http://localhost:5173) - Shows MOCK data (fake system info)
2. **Tauri Window** - Shows REAL system data from your computer

**Always look at the Tauri window for real system information!**

### Troubleshooting

#### "cargo not found" Error
- Install Rust from https://rustup.rs/
- Restart your terminal after installation

#### "WebView2 not found" Error
- Download and install WebView2 from Microsoft
- Most Windows 10/11 systems have it pre-installed

#### Blank Window
1. Press F12 in the Tauri window to open Developer Tools
2. Check the Console tab for errors
3. Report any red error messages

#### Windows Defender Issues
- Windows Defender might block the first run
- Click "More info" → "Run anyway" if prompted

#### VS Code Terminal Issues
- If running from VS Code's integrated terminal fails
- Use Windows Terminal or Command Prompt instead

## Visual Indicators

The app shows whether it's using real or mock data:

- **Window Title**: 
  - "System Monitor (Tauri)" = Real data ✓
  - "System Monitor (Mock Data)" = Fake data ✗

- **Warning Banner**:
  - Yellow banner visible = Mock data
  - No banner = Real data

## Development Tips

1. **Hot Reload**: Changes to the frontend automatically reload
2. **Backend Changes**: Rust code changes trigger a recompile (takes ~30 seconds)
3. **Console Logs**: Press F12 in the Tauri window to see debug output

## Common Windows-Specific Issues

1. **Long Path Names**: Windows has a 260 character path limit
   - Enable long paths: https://docs.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation
   - Or move project closer to drive root (e.g., C:\Projects\)

2. **Antivirus Software**: May slow down or block compilation
   - Add project folder to exclusions
   - Especially important for Rust compilation

3. **Permission Issues**: 
   - Run as regular user (not Administrator) unless specified
   - Ensure you have write permissions in the project directory

## Getting Real System Data

To verify you're seeing real data:
1. Check CPU name matches your actual processor
2. Check RAM amount matches your system
3. Check hostname matches your computer name

If you see "Mock Intel Core i9" or "MockOS", you're looking at the wrong window!