#!/bin/bash

echo "=== Debugging Tauri System Detection ==="
echo ""

# Step 1: Check if we can access system info directly
echo "1. Testing direct system access..."
echo "-----------------------------------"

# Create a simple Python test
cat > /tmp/test_system.py << 'EOF'
import platform
import psutil
import socket

print("System Information Test:")
print(f"  Hostname: {socket.gethostname()}")
print(f"  Platform: {platform.system()} {platform.release()}")
print(f"  CPU Count: {psutil.cpu_count()}")
print(f"  Memory: {psutil.virtual_memory().total / (1024**3):.1f} GB")
print(f"  CPU Usage: {psutil.cpu_percent(interval=1)}%")
EOF

python3 /tmp/test_system.py 2>/dev/null || echo "Python test failed - psutil not installed"

echo ""
echo "2. Checking Tauri window status..."
echo "-----------------------------------"

# Check if any Tauri window exists
xwininfo -root -tree 2>/dev/null | grep -E '"[^"]+".*\(' | grep -v -E "(Firefox|Chrome|Terminal)" | head -10

echo ""
echo "3. Questions to verify:"
echo "-----------------------------------"
echo "When you run 'pnpm run tauri dev':"
echo ""
echo "a) Do you see a NEW WINDOW open? (Not the browser)"
echo "   - If YES: That's the Tauri window - what does it show?"
echo "   - If NO: The Tauri window is not opening (snap issue)"
echo ""
echo "b) In the terminal output, do you see:"
echo "   - '=== Tauri App Setup ===' ?"
echo "   - '=== get_system_info called ===' ?"
echo "   - Any error messages?"
echo ""
echo "c) Are you looking at:"
echo "   - Firefox/Chrome at localhost:5173? (This is ALWAYS mock data)"
echo "   - A separate application window? (This would be the real Tauri app)"
echo ""

echo "4. Let's check the console output..."
echo "-----------------------------------"
echo "If the Tauri window IS open:"
echo "1. Right-click in the window"
echo "2. Select 'Inspect Element' or press F12"
echo "3. Go to the Console tab"
echo "4. Look for these messages:"
echo "   - 'TAURI DETECTED - NOT USING MOCK SERVICE'"
echo "   - 'USING MOCK SERVICE - NOT REAL SYSTEM DATA'"
echo ""
echo "Which message do you see?"