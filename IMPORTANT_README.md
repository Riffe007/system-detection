# Understanding Tauri Dev Mode

## The Problem

When you run `pnpm run tauri dev`, two things happen:
1. A web server starts at http://localhost:5173 (Vite dev server)
2. A native Tauri window SHOULD open (but it's crashing on your system)

**You've been looking at #1 in Firefox, which will ALWAYS show mock data!**

## Why Mock Data in Browser?

The browser at localhost:5173 shows mock data because:
- It's a regular web browser (Firefox)
- It has no access to system APIs
- `window.__TAURI__` is not available in regular browsers
- The app correctly detects this and falls back to mock data

## The Real Tauri Window

The actual Tauri window (which would show real data):
- Is a native application window
- Has `window.__TAURI__` injected
- Can access system information through Rust backend
- **But it's crashing due to snap/glibc conflicts on your Ubuntu system**

## The Error

```
symbol lookup error: /snap/core20/current/lib/x86_64-linux-gnu/libpthread.so.0: undefined symbol: __libc_pthread_init
```

This happens because your system uses snap packages, which have isolated library environments that conflict with the Tauri app.

## Solutions

### Option 1: Run the standalone script
```bash
./run-tauri-standalone.sh
```
This attempts to bypass snap conflicts and open the real Tauri window.

### Option 2: Build for production
```bash
pnpm run tauri build
# Then run the built app from src-tauri/target/release/system-monitor
```

### Option 3: Use a non-snap environment
Consider using a Ubuntu system without snap packages, or uninstalling the snap version of your IDE/terminal.

## How to Verify It's Working

When Tauri is working correctly:
1. A **separate window** will open (not in your browser)
2. The window title will say "System Monitor (Tauri)"
3. NO yellow warning banner about mock data
4. You'll see real CPU, memory, and system metrics

## Visual Indicators I've Added

- **In Browser**: Yellow banner saying "MOCK DATA MODE"
- **In Tauri Window**: No warning banner, real system data
- **Window Title**: Shows "(Mock Data)" or "(Tauri)"
- **Console Logs**: Clear messages about which mode is active