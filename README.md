# 🔧 Rust System Monitor

An async, modular system metrics collector built in Rust.  
Collects CPU, memory, GPU, and storage information using a clean architecture and extensible backend. Designed as a lightweight, open-source foundation for system diagnostics, embedded monitoring tools, or performance dashboards.

---

## ✨ Features

- **Written in Rust** for speed, safety, and low-level control.
- **Asynchronous** metric collection via [`tokio`](https://tokio.rs/).
- **Modular architecture** with a central `SystemMonitor` interface.
- **Clean CLI output** for real-time system diagnostics.
- **Ready for extension** into daemons, web APIs, or logging services.
- **Open Source** under the MIT License.

---

## 📦 Modules

This project is structured into reusable system-specific components:

| Module            | Responsibility                    |
|-------------------|------------------------------------|
| `cpu_monitor.rs`   | Detects CPU usage and core status |
| `memory_monitor.rs`| Retrieves memory usage metrics    |
| `gpu_monitor.rs`   | Queries GPU statistics            |
| `storage_monitor.rs`| Collects disk usage information |
| `logger.rs`        | Optional hook for logging output  |
| `system_monitor.rs`| Aggregates all subsystems into a unified metric snapshot |

---

## 🚀 Getting Started

### ✅ Requirements

- Rust (latest stable version)
- Cargo

### 🔨 Build & Run

Clone the repository:

```bash
git clone https://github.com/Riffe/system-detection.git
cd rust-system-monitor
```
## Build the project
```bash
cargo build --release
```
## Run the monitor
```bash
cargo run --release
```
### This will print a snapshot of the current system metrics to the console:
```bash
CPU Usage: 12.5%
Memory Used: 4.9 GB
GPU Temp: 61.2 °C
Storage Used: 250 GB / 1 TB
```
Output will vary depending on your operating system and available hardware.

# 🔧 Architecture Overview
The main runtime is powered by %tokio:%
```rust
fn main() {
    let rt = Runtime::new().unwrap();
    let metrics = rt.block_on(SystemMonitor::collect_metrics());

    for (key, value) in metrics.iter() {
        println!("{}: {}", key, value);
    }
}
```
The SystemMonitor delegates to subsystem modules and returns a HashMap<String, String> containing unified metric data.
# 🧠 Why This Project?
  - This is part of my systems engineering portfolio and reflects my work in:
  - Designing low-level, high-performance monitoring systems
  - Modular and scalable Rust architectures
  - Cross-platform observability tooling
  - Real-time data systems and performance-aware applications
While this project is open-source, I also develop more advanced proprietary monitoring systems with predictive diagnostics and quantum-inspired state modeling. This lightweight tool is a distilled showcase of clean design and practical insight.

# 🪪 License
This project is licensed under the MIT License.
You're free to use, modify, and distribute it — just keep the license notice intact.

# 🙋 Contributing
Pull requests and suggestions are welcome!
Open an issue or fork the project and submit a PR — let’s build something great.

# 📫 Contact
Want to connect or collaborate on systems-level tools, optimization research, or AI infrastructure?

Reach out at timothy@riffeandassociates.com or visit https://www.riffe.tech 
```vbnet

---

### 🔏 MIT LICENSE File (`LICENSE`)

I'll also prep the text for the MIT License to include in the repo:

```text
MIT License

Copyright (c) 2025 [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights  
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell  
copies of the Software, and to permit persons to whom the Software is  
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in  
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR  
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,  
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE  
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER  
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING  
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS  
IN THE SOFTWARE.
```

