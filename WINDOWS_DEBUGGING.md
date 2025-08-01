# Windows Debugging Guide

## If the app is stuck on "Loading System Monitor..."

### 1. Open Developer Console
In the Tauri window (not browser), press **F12** and check:

- **Console tab**: Look for errors (red text)
- Look for these specific messages:
  - "Starting initialization..."
  - "Tauri v2 Detection Service"
  - "Setting loading to false"

### 2. Common Windows Issues

#### A. Windows Defender Blocking
- Windows Defender may block Tauri commands
- Check Windows Security notifications
- Add the app to exclusions if needed

#### B. WebView2 Issues
- The app might be stuck initializing WebView2
- Check if other Tauri/Electron apps work
- Reinstall WebView2 if needed

#### C. Large System Detection
If you have a high-core-count CPU (16+ cores):
- The app might be overwhelmed processing core data
- Check console for "per_core_usage" errors

### 3. Quick Fixes

#### Fix 1: Force Reload
In the Tauri window:
- Press `Ctrl+R` to reload
- Press `Ctrl+Shift+R` for hard reload

#### Fix 2: Check Tauri Communication
In the developer console, type:
```javascript
// Check if Tauri is available
window.__TAURI__

// Try calling a command directly
await window.__TAURI__.core.invoke('get_system_info')
```

If this returns data, the backend works but the frontend is stuck.

#### Fix 3: Clear localStorage
In the console:
```javascript
localStorage.clear()
location.reload()
```

### 4. Diagnostic Commands

Run these in the Tauri window's console:

```javascript
// Check initialization state
console.log('Document ready state:', document.readyState);
console.log('Root element:', document.getElementById('root'));

// Check React mounting
console.log('React root:', document.getElementById('root')?._reactRootContainer);

// Force set loading to false (emergency fix)
if (window.setAppLoading) {
  window.setAppLoading(false);
}
```

### 5. Performance Issues

For high-performance systems:
- CPU with 32+ cores
- 64GB+ RAM
- Multiple GPUs

The app now includes:
- Throttled updates (100ms minimum between renders)
- Simplified view for 32+ core systems
- Optimized grid layouts

### 6. Report Issues

If still stuck, gather this info:
1. Screenshot of the console errors
2. Your system specs:
   - Windows version
   - CPU model and core count
   - RAM amount
3. Output of console commands above

## Emergency Workaround

If the app won't load at all, create this file:

**emergency-fix.html**
```html
<!DOCTYPE html>
<html>
<head>
    <title>System Monitor Emergency Mode</title>
</head>
<body>
    <h1>Emergency System Info</h1>
    <div id="info"></div>
    <script>
        async function test() {
            try {
                const info = await window.__TAURI__.core.invoke('get_system_info');
                document.getElementById('info').innerText = JSON.stringify(info, null, 2);
            } catch (e) {
                document.getElementById('info').innerText = 'Error: ' + e;
            }
        }
        if (window.__TAURI__) {
            test();
        } else {
            document.getElementById('info').innerText = 'Tauri not available';
        }
    </script>
</body>
</html>
```

Then navigate to this file in the Tauri window to test if the backend works.