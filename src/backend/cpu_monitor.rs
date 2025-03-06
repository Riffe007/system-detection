use sysinfo::{System, SystemExt, CpuExt, ProcessExt};
use std::collections::HashMap;

/// Struct for CPU Monitoring
pub struct CpuMonitor;

impl CpuMonitor {
    /// Get per-process & per-core CPU usage
    pub fn get_usage() -> HashMap<String, f32> {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let mut cpu_usage = HashMap::new();
        let total_cpu_usage = sys.global_cpu_info().cpu_usage();
        cpu_usage.insert("Total CPU".to_string(), total_cpu_usage);

        for (pid, process) in sys.processes() {
            cpu_usage.insert(format!("PID {} - {}", pid, process.name()), process.cpu_usage());
        }

        cpu_usage
    }
}
