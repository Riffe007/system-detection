#!/bin/bash

echo "=== Tauri Standalone Runner ==="
echo "This will run the Tauri app with real system data"
echo ""

# Kill any existing processes
pkill -f "system-monitor" || true
pkill -f "tauri dev" || true
sleep 2

# Create a wrapper script that completely isolates from snap
cat > /tmp/tauri-runner.sh << 'EOF'
#!/bin/bash
# Clear ALL environment variables
env -i \
  HOME="$HOME" \
  USER="$USER" \
  PATH="/home/ubuntu/.cargo/bin:/home/ubuntu/.nvm/versions/node/v22.17.1/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin" \
  LD_LIBRARY_PATH="/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu" \
  WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 \
  WEBKIT_FORCE_SANDBOX=0 \
  DISPLAY="$DISPLAY" \
  XAUTHORITY="$XAUTHORITY" \
  /bin/bash -c 'cd /home/ubuntu/system-detection && pnpm run tauri dev'
EOF

chmod +x /tmp/tauri-runner.sh

echo "Starting Tauri in completely clean environment..."
echo "This may take 2-3 minutes to compile on first run."
echo ""
echo "IMPORTANT: Look for a NEW WINDOW to open - that's the Tauri app with real data!"
echo "The browser at localhost:5173 will always show mock data."
echo ""

# Run the wrapper script
/tmp/tauri-runner.sh