use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::{System, Disks, Networks, ProcessStatus};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Import the high-performance monitoring system
pub mod high_perf_monitor;
use high_perf_monitor::{HighPerfMonitoringService, HighPerfMetrics};

// Import ultra-performance monitoring system
pub mod ultra_perf_monitor;
use ultra_perf_monitor::{UltraPerfMonitoringService, UltraPerfMetrics};

// Import kernel-level monitoring
pub mod kernel_monitor;
pub mod linux_ebpf;
pub mod windows_etw;

use kernel_monitor::{KernelMonitor, KernelMetrics};

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
    pub boot_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub frequency_mhz: u64,
    pub per_core_usage: Vec<f32>,
    pub temperature: Option<f32>,
    pub load_average: [f32; 3],
    pub processes_total: usize,
    pub processes_running: usize,
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
    pub clock_mhz: f32,
    pub memory_clock_mhz: f32,
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
    pub start_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: String,
    pub system_info: SystemInfo,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub gpus: Vec<GpuMetrics>,
    pub disks: Vec<DiskMetrics>,
    pub networks: Vec<NetworkMetrics>,
    pub top_processes: Vec<ProcessMetrics>,
}

pub struct MonitoringService {
    system: Arc<RwLock<System>>,
    metrics_callback: Arc<RwLock<Option<Box<dyn Fn(SystemMetrics) + Send + Sync>>>>,
    previous_network_stats: Arc<RwLock<HashMap<String, (u64, u64)>>>,
    // High-performance monitoring system
    high_perf_service: Option<HighPerfMonitoringService>,
    high_perf_callback: Arc<RwLock<Option<Box<dyn Fn(HighPerfMetrics) + Send + Sync>>>>,
    // Ultra-performance monitoring system
    ultra_perf_service: Option<UltraPerfMonitoringService>,
    ultra_perf_callback: Arc<RwLock<Option<Box<dyn Fn(UltraPerfMetrics) + Send + Sync>>>>,
    // Kernel-level monitoring system
    kernel_monitor: Option<KernelMonitor>,
    kernel_callback: Arc<RwLock<Option<Box<dyn Fn(KernelMetrics) + Send + Sync>>>>,
}

impl MonitoringService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics_callback: Arc::new(RwLock::new(None)),
            previous_network_stats: Arc::new(RwLock::new(HashMap::new())),
            high_perf_service: None,
            high_perf_callback: Arc::new(RwLock::new(None)),
            ultra_perf_service: None,
            ultra_perf_callback: Arc::new(RwLock::new(None)),
            kernel_monitor: None,
            kernel_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub fn new_with_high_perf(update_interval_ms: u64) -> Self {
        let mut service = Self::new();
        service.high_perf_service = Some(HighPerfMonitoringService::new(update_interval_ms));
        service.ultra_perf_service = Some(UltraPerfMonitoringService::new(update_interval_ms));
        service
    }

    pub async fn set_metrics_callback<F>(&mut self, callback: F)
    where
        F: Fn(SystemMetrics) + Send + Sync + 'static,
    {
        *self.metrics_callback.write().await = Some(Box::new(callback));
    }

    pub async fn set_high_perf_callback<F>(&mut self, callback: F)
    where
        F: Fn(HighPerfMetrics) + Send + Sync + 'static,
    {
        *self.high_perf_callback.write().await = Some(Box::new(callback));
    }

    pub async fn set_ultra_perf_callback<F>(&mut self, callback: F)
    where
        F: Fn(UltraPerfMetrics) + Send + Sync + 'static,
    {
        *self.ultra_perf_callback.write().await = Some(Box::new(callback));
    }

    pub fn start_high_perf_monitoring(&mut self) {
        if let Some(service) = &self.high_perf_service {
            service.start();
        }
    }

    pub fn start_ultra_perf_monitoring(&mut self) {
        if let Some(service) = &self.ultra_perf_service {
            service.start();
        }
    }

    #[allow(dead_code)]
    pub fn stop_high_perf_monitoring(&mut self) {
        if let Some(service) = &self.high_perf_service {
            service.stop();
        }
    }

    pub fn stop_ultra_perf_monitoring(&mut self) {
        if let Some(service) = &self.ultra_perf_service {
            service.stop();
        }
    }

    pub fn get_high_perf_metrics(&self) -> Option<HighPerfMetrics> {
        self.high_perf_service.as_ref()?.get_latest_metrics()
    }

    pub fn get_ultra_perf_metrics(&self) -> Option<UltraPerfMetrics> {
        self.ultra_perf_service.as_ref()?.get_latest_metrics()
    }

    pub async fn set_kernel_callback<F>(&mut self, callback: F)
    where
        F: Fn(KernelMetrics) + Send + Sync + 'static,
    {
        *self.kernel_callback.write().await = Some(Box::new(callback));
    }

    pub fn start_kernel_monitoring(&mut self) -> Result<(), kernel_monitor::KernelMonitorError> {
        if self.kernel_monitor.is_none() {
            self.kernel_monitor = Some(KernelMonitor::new()?);
        }
        
        if let Some(monitor) = &mut self.kernel_monitor {
            monitor.start()?;
        }
        
        Ok(())
    }

    pub fn stop_kernel_monitoring(&mut self) {
        if let Some(monitor) = &mut self.kernel_monitor {
            monitor.stop();
        }
    }

    pub fn get_kernel_metrics(&self) -> Option<KernelMetrics> {
        self.kernel_monitor.as_ref()?.get_latest_metrics()
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, String> {
        let system = self.system.read().await;
        
        let hostname = hostname::get()
            .map_err(|e| format!("Failed to get hostname: {}", e))?
            .to_string_lossy()
            .to_string();

        let os_info = os_info::get();
        let cpu_info = system.global_cpu_info();

        Ok(SystemInfo {
            hostname,
            os_name: os_info.os_type().to_string(),
            os_version: os_info.version().to_string(),
            kernel_version: os_info.edition().unwrap_or("Unknown").to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_brand: cpu_info.brand().to_string(),
            cpu_cores: system.cpus().len(),
            cpu_threads: system.cpus().len(),
            total_memory: system.total_memory(),
            boot_time: sysinfo::System::boot_time() as i64,
        })
    }

    async fn get_gpu_metrics(&self) -> Vec<GpuMetrics> {
        let mut gpus = Vec::new();
        
        #[cfg(feature = "nvidia")]
        {
            if let Ok(nvml) = nvml_wrapper::Nvml::init() {
                if let Ok(device_count) = nvml.device_count() {
                    for i in 0..device_count {
                        if let Ok(device) = nvml.device_by_index(i) {
                            if let (Ok(name), Ok(memory), Ok(utilization), Ok(temperature)) = (
                                device.name(),
                                device.memory_info(),
                                device.utilization_rates(),
                                device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                            ) {
                                let memory_usage_percent = if memory.total > 0 {
                                    (memory.used as f32 / memory.total as f32) * 100.0
                                } else {
                                    0.0
                                };
                                
                                gpus.push(GpuMetrics {
                                    name: name.trim().to_string(),
                                    driver_version: "Unknown".to_string(), // TODO: Implement driver version detection
                                    temperature_celsius: temperature as f32,
                                    usage_percent: utilization.gpu as f32,
                                    memory_total_bytes: memory.total,
                                    memory_used_bytes: memory.used,
                                    memory_usage_percent,
                                    power_watts: device.power_usage().unwrap_or(0) as f32 / 1000.0,
                                    fan_speed_percent: device.fan_speed(0).ok().map(|speed| speed as f32),
                                    clock_mhz: device.max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics).unwrap_or(0) as f32,
                                    memory_clock_mhz: device.max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory).unwrap_or(0) as f32,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        gpus
    }

    async fn get_disk_metrics(&self) -> Vec<DiskMetrics> {
        let disks = Disks::new_with_refreshed_list();
        
        disks.iter().map(|disk| {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0 {
                (used_space as f32 / total_space as f32) * 100.0
            } else {
                0.0
            };

            DiskMetrics {
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                device_name: disk.name().to_string_lossy().to_string(),
                fs_type: disk.file_system().to_string_lossy().to_string(),
                total_bytes: total_space,
                used_bytes: used_space,
                available_bytes: available_space,
                usage_percent,
                read_bytes_per_sec: 0, // TODO: Add I/O rate calculation
                write_bytes_per_sec: 0,
                io_operations_per_sec: 0,
            }
        }).collect()
    }

    async fn get_network_metrics(&self) -> Vec<NetworkMetrics> {
        let networks = Networks::new_with_refreshed_list();
        let mut network_metrics = Vec::new();
        
        for (name, data) in networks.iter() {
            let current_sent = data.total_transmitted();
            let current_received = data.total_received();
            
            let (sent_rate, received_rate) = {
                let mut stats = self.previous_network_stats.write().await;
                if let Some((prev_sent, prev_received)) = stats.get(name) {
                    let sent_diff = current_sent.saturating_sub(*prev_sent);
                    let received_diff = current_received.saturating_sub(*prev_received);
                    stats.insert(name.clone(), (current_sent, current_received));
                    (sent_diff, received_diff)
                } else {
                    stats.insert(name.clone(), (current_sent, current_received));
                    (0, 0)
                }
            };

            network_metrics.push(NetworkMetrics {
                interface_name: name.clone(),
                is_up: true, // TODO: Implement is_up detection
                mac_address: "Unknown".to_string(), // TODO: Implement MAC address detection
                ip_addresses: Vec::new(), // TODO: Implement IP address detection
                bytes_sent: current_sent,
                bytes_received: current_received,
                packets_sent: data.total_packets_transmitted(),
                packets_received: data.total_packets_received(),
                errors_sent: data.total_errors_on_transmitted(),
                errors_received: data.total_errors_on_received(),
                speed_mbps: None, // TODO: Add speed detection
                bytes_sent_rate: sent_rate,
                bytes_received_rate: received_rate,
            });
        }
        
        network_metrics
    }

    pub async fn collect_metrics(&self) -> Result<SystemMetrics, String> {
        let system = self.system.read().await;
        
        // CPU metrics
        let cpu_usage = system.global_cpu_info().cpu_usage();
        let per_core_usage: Vec<f32> = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        
        let raw_frequency = system.cpus().first()
            .map(|cpu| cpu.frequency())
            .unwrap_or(0);
        println!("Raw CPU frequency from sysinfo: {} Hz", raw_frequency);
        let frequency_mhz = raw_frequency / 1_000_000;
        
        let cpu_metrics = CpuMetrics {
            usage_percent: cpu_usage,
            frequency_mhz, // Now correctly in MHz
            per_core_usage,
            temperature: None,
            load_average: {
                let load = sysinfo::System::load_average();
                [load.one as f32, load.five as f32, load.fifteen as f32]
            },
            processes_total: system.processes().len(),
            processes_running: system.processes().values()
                .filter(|p| matches!(p.status(), ProcessStatus::Run))
                .count(),
            context_switches: 0,
            interrupts: 0,
        };

        // Memory metrics
        let memory_metrics = MemoryMetrics {
            total_bytes: system.total_memory(),
            used_bytes: system.used_memory(),
            available_bytes: system.available_memory(),
            cached_bytes: 0, // Would need additional system calls
            swap_total_bytes: system.total_swap(),
            swap_used_bytes: system.used_swap(),
            usage_percent: (system.used_memory() as f32 / system.total_memory() as f32) * 100.0,
            swap_usage_percent: if system.total_swap() > 0 {
                (system.used_swap() as f32 / system.total_swap() as f32) * 100.0
            } else {
                0.0
            },
        };

        // GPU metrics
        let gpu_metrics = self.get_gpu_metrics().await;

        // Disk metrics
        let disk_metrics = self.get_disk_metrics().await;

        // Network metrics
        let network_metrics = self.get_network_metrics().await;

        // Process metrics (top processes by CPU usage)
        let mut processes: Vec<ProcessMetrics> = system.processes()
            .iter()
            .map(|(pid, process)| {
                ProcessMetrics {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    cpu_usage_percent: process.cpu_usage(),
                    memory_bytes: process.memory(),
                    memory_percent: (process.memory() as f32 / system.total_memory() as f32) * 100.0,
                    disk_read_bytes: 0, // Would need additional system calls
                    disk_write_bytes: 0, // Would need additional system calls
                    status: format!("{:?}", process.status()),
                    threads: 1, // TODO: Implement thread count detection
                    start_time: process.start_time().to_string(),
                }
            })
            .collect();

        processes.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());
        processes.truncate(20);

        let system_info = self.get_system_info().await?;

        Ok(SystemMetrics {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            system_info,
            cpu: cpu_metrics,
            memory: memory_metrics,
            gpus: gpu_metrics,
            disks: disk_metrics,
            networks: network_metrics,
            top_processes: processes,
        })
    }

    pub async fn start_monitoring(&self) {
        // Start high-performance monitoring
        if let Some(service) = &self.high_perf_service {
            service.start();
        }

        // Start ultra-performance monitoring
        if let Some(service) = &self.ultra_perf_service {
            service.start();
        }

        // Start kernel-level monitoring
        // Note: Kernel monitoring is started when the service is created
        // The monitor is already running if it exists

        // Set up callbacks for high-performance metrics
        if let Some(service) = &self.high_perf_service {
            let callback = self.high_perf_callback.clone();
            let receiver = service.subscribe();
            
            tokio::spawn(async move {
                while let Ok(metrics) = receiver.recv() {
                    if let Some(cb) = &*callback.read().await {
                        cb(metrics);
                    }
                }
            });
        }

        // Set up callbacks for ultra-performance metrics
        if let Some(service) = &self.ultra_perf_service {
            let callback = self.ultra_perf_callback.clone();
            let receiver = service.subscribe();
            
            tokio::spawn(async move {
                while let Ok(metrics) = receiver.recv() {
                    if let Some(cb) = &*callback.read().await {
                        cb(metrics);
                    }
                }
            });
        }

        // Set up callbacks for kernel metrics
        // Note: Kernel monitoring callbacks are handled separately
        // to avoid lifetime issues with the monitor reference
    }
}