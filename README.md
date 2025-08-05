# 🔧 Rust System Monitor

A **real-time, cross-platform system monitoring application** built with **Tauri v2** (Rust backend + React frontend).  
Collects comprehensive CPU, memory, GPU, disk, network, and process metrics using a clean architecture and extensible backend. Designed as a lightweight, open-source foundation for system diagnostics, embedded monitoring tools, or performance dashboards.

---

## ✨ Features

- **Written in Rust** for speed, safety, and low-level control
- **Tauri v2** for cross-platform desktop applications
- **Real-time monitoring** with 1-second update intervals
- **GPU monitoring** with NVIDIA NVML support (temperature, usage, memory, power)
- **Interactive dashboard** with draggable, customizable widgets
- **Live charts** and historical data visualization
- **Dark/light theme** support
- **Responsive design** that adapts to different screen sizes
- **Process monitoring** with top processes by CPU/memory usage
- **Network monitoring** with real-time bandwidth tracking
- **Disk monitoring** with I/O rate tracking
- **Open Source** under the MIT License

---

## 🚀 Getting Started

### ✅ Requirements

- **Rust** (latest stable version)
- **Node.js** (v18 or later)
- **pnpm** (recommended) or npm
- **NVIDIA drivers** (for GPU monitoring - optional)

### 🔨 Build & Run

Clone the repository:

```bash
git clone https://github.com/Riffe/system-detection.git
cd system-detection
```

Install dependencies:

```bash
pnpm install
```

Build and run in development mode:

```bash
pnpm run tauri dev
```

Build for production:

```bash
pnpm run tauri build
```

### 🎯 Running the Application

**Important**: This application requires the Tauri runtime and cannot run in a regular browser. The application will show an error if launched in a browser environment.

When running correctly, you'll see:
- A **native application window** (not a browser tab)
- **Real system data** (no mock data)
- **Live updating metrics** every second
- **Interactive dashboard** with draggable widgets

---

## 📊 Monitoring Capabilities

### CPU Monitoring
- **Real-time usage** percentage
- **Per-core usage** visualization
- **Frequency** monitoring
- **Load average** (1, 5, 15 minute)
- **Process count** (running/total)

### Memory Monitoring
- **Total/used/available** memory
- **Swap usage** monitoring
- **Memory pressure** indicators
- **Real-time charts**

### GPU Monitoring (NVIDIA)
- **GPU usage** percentage
- **Memory usage** and capacity
- **Temperature** monitoring
- **Power draw** in watts
- **Clock speeds** (core/memory)
- **Fan speed** (if available)
- **Multi-GPU** support

### Disk Monitoring
- **Space usage** by mount point
- **I/O rates** (read/write bytes per second)
- **File system** information
- **Real-time bandwidth** tracking

### Network Monitoring
- **Interface status** and statistics
- **Bandwidth usage** (bytes sent/received per second)
- **Packet statistics**
- **Network interface** details

### Process Monitoring
- **Top processes** by CPU usage
- **Memory usage** per process
- **Process status** and details
- **Real-time updates**

---

## 🏗️ Architecture Overview

### Backend (Rust/Tauri)
- **`src-tauri/src/monitoring.rs`**: Core monitoring service using `sysinfo` crate
- **Async metrics collection**: Tokio-based background monitoring
- **Event-driven updates**: Real-time metrics pushed to frontend
- **GPU monitoring**: NVIDIA NVML integration for GPU metrics
- **Rate calculation**: Real-time I/O and network rate tracking

### Frontend (React/TypeScript)
- **Modern React 18** with hooks and functional components
- **Tailwind CSS** for styling with dark/light theme support
- **Recharts** for data visualization
- **Drag-and-drop dashboard** using `@dnd-kit`
- **Responsive grid layout** with customizable widgets

### Key Components
- **`AppWrapper.tsx`**: Main application logic and Tauri integration
- **`DraggableDashboard.tsx`**: Interactive dashboard with widget management
- **Monitor Components**: Individual monitoring widgets (CPU, Memory, GPU, etc.)
- **`useMetricsHistory.ts`**: Custom hook for historical data tracking

---

## 🔧 Configuration

### GPU Monitoring
GPU monitoring requires:
1. **NVIDIA GPU** with supported drivers
2. **NVML library** (included via `nvml-wrapper` crate)
3. **Feature flag enabled** (enabled by default)

To disable GPU monitoring:
```toml
# In Cargo.toml
[features]
default = []  # Remove "nvidia"
```

### Dashboard Customization
- **Drag and drop** widgets to reorder
- **Resize widgets** (small, medium, large, full-width)
- **Show/hide widgets** in edit mode
- **Layout persistence** (saved to localStorage)

---

## 🧪 Testing

Run all tests:

```bash
pnpm run test:all
```

Run specific test suites:

```bash
# Frontend tests
pnpm run test

# Rust backend tests
pnpm run test:rust

# Test coverage
pnpm run test:coverage
```

---

## 🐛 Troubleshooting

### Common Issues

1. **"Tauri environment not detected"**
   - Ensure you're running through Tauri, not in a browser
   - Use `pnpm run tauri dev` to launch

2. **GPU monitoring not working**
   - Verify NVIDIA drivers are installed
   - Check that `nvml-wrapper` feature is enabled
   - Ensure you have an NVIDIA GPU

3. **High CPU usage**
   - The app updates every second by default
   - Consider reducing update frequency for older systems

4. **Permission errors**
   - The app needs system access for monitoring
   - Grant necessary permissions when prompted

### Debug Mode
Enable debug logging by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug pnpm run tauri dev
```

---

## 🪪 License

This project is licensed under the MIT License.
You're free to use, modify, and distribute it — just keep the license notice intact.

---

## 🙋 Contributing

Pull requests and suggestions are welcome!
Open an issue or fork the project and submit a PR — let's build something great.

### Development Guidelines
- **No mock data**: All features must work with real system data
- **Real-time updates**: All metrics should update live
- **Cross-platform**: Ensure compatibility across Windows, macOS, and Linux
- **Performance**: Optimize for minimal system impact

---

## 📫 Contact

Want to connect or collaborate on systems-level tools, optimization research, or AI infrastructure?

Reach out at timothy@riffeandassociates.com or visit https://www.riffe.tech 
```

