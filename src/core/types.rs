use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub architecture: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub total_memory: u64,
    pub boot_time: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub frequency_mhz: u64,
    pub temperature_celsius: Option<f32>,
    pub load_average: [f32; 3],
    pub per_core_usage: Vec<f32>,
    pub processes_running: usize,
    pub processes_total: usize,
    pub context_switches: u64,
    pub interrupts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub usage_percent: f32,
    pub swap_usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub name: String,
    pub driver_version: String,
    pub temperature_celsius: f32,
    pub usage_percent: f32,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_usage_percent: f32,
    pub power_watts: f32,
    pub fan_speed_percent: Option<f32>,
    pub clock_mhz: u32,
    pub memory_clock_mhz: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub mount_point: String,
    pub device_name: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub io_operations_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interface_name: String,
    pub is_up: bool,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors_sent: u64,
    pub errors_received: u64,
    pub speed_mbps: Option<u64>,
    pub bytes_sent_rate: u64,
    pub bytes_received_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_usage_percent: f32,
    pub memory_bytes: u64,
    pub memory_percent: f32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub status: String,
    pub threads: u32,
    pub start_time: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: SystemTime,
    pub system_info: SystemInfo,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub gpus: Vec<GpuMetrics>,
    pub disks: Vec<DiskMetrics>,
    pub networks: Vec<NetworkMetrics>,
    pub top_processes: Vec<ProcessMetrics>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MonitoringInterval {
    pub cpu: Duration,
    pub memory: Duration,
    pub gpu: Duration,
    pub disk: Duration,
    pub network: Duration,
    pub process: Duration,
}

impl Default for MonitoringInterval {
    fn default() -> Self {
        Self {
            cpu: Duration::from_millis(500),
            memory: Duration::from_secs(1),
            gpu: Duration::from_secs(1),
            disk: Duration::from_secs(2),
            network: Duration::from_secs(1),
            process: Duration::from_secs(2),
        }
    }
}