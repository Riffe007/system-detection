use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::{System, Disks, Networks, ProcessStatus};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics_callback: Arc::new(RwLock::new(None)),
            previous_network_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_metrics_callback<F>(&mut self, callback: F)
    where
        F: Fn(SystemMetrics) + Send + Sync + 'static,
    {
        *self.metrics_callback.write().await = Some(Box::new(callback));
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, String> {
        println!("=== get_system_info called in monitoring service ===");
        
        let mut sys = self.system.write().await;
        println!("Refreshing system info...");
        sys.refresh_all();
        println!("System info refreshed successfully");
        
        let info = os_info::get();
        println!("OS info retrieved: {} {}", info.os_type(), info.version());
        
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        println!("Hostname: {}", hostname);
        
        let cpu_brand = sys.cpus().first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_default();
        println!("CPU Brand: {}", cpu_brand);
        
        let cpu_cores = sys.physical_core_count().unwrap_or(0);
        let cpu_threads = sys.cpus().len();
        println!("CPU Cores: {}, Threads: {}", cpu_cores, cpu_threads);
        
        let total_memory = sys.total_memory();
        println!("Total Memory: {} MB", total_memory / 1024 / 1024);
        
        Ok(SystemInfo {
            hostname,
            os_name: info.os_type().to_string(),
            os_version: info.version().to_string(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_brand,
            cpu_cores,
            cpu_threads,
            total_memory,
            boot_time: System::boot_time() as i64,
        })
    }

    async fn get_gpu_metrics(&self) -> Vec<GpuMetrics> {
        let mut gpus = Vec::new();
        
        // Try to get NVIDIA GPU info if available
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
                                    driver_version: nvml.sys_driver_version().unwrap_or_default(),
                                    temperature_celsius: temperature as f32,
                                    usage_percent: utilization.gpu as f32,
                                    memory_total_bytes: memory.total,
                                    memory_used_bytes: memory.used,
                                    memory_usage_percent,
                                    power_watts: device.power_usage().unwrap_or(0) as f32 / 1000.0,
                                    fan_speed_percent: device.fan_speed(0).ok().map(|speed| speed as f32),
                                    clock_mhz: device.max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Gpu).unwrap_or(0) as f32,
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
        let disk_metrics: Vec<DiskMetrics> = disks.iter()
            .map(|disk| DiskMetrics {
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                device_name: disk.name().to_string_lossy().to_string(),
                fs_type: disk.file_system().to_string_lossy().to_string(),
                total_bytes: disk.total_space(),
                used_bytes: disk.total_space() - disk.available_space(),
                available_bytes: disk.available_space(),
                usage_percent: ((disk.total_space() - disk.available_space()) as f32 / disk.total_space() as f32) * 100.0,
                read_bytes_per_sec: 0, // sysinfo doesn't provide this in current version
                write_bytes_per_sec: 0, // sysinfo doesn't provide this in current version
                io_operations_per_sec: 0, // sysinfo doesn't provide this in current version
            })
            .collect();
        
        disk_metrics
    }

    async fn get_network_metrics(&self) -> Vec<NetworkMetrics> {
        let networks = Networks::new_with_refreshed_list();
        let mut network_metrics = Vec::new();
        let mut previous_stats = self.previous_network_stats.write().await;
        
        for (name, data) in networks.iter() {
            let current_sent = data.total_transmitted();
            let current_received = data.total_received();
            
            let (sent_rate, received_rate) = if let Some((prev_sent, prev_received)) = previous_stats.get(name) {
                let sent_diff = current_sent.saturating_sub(*prev_sent);
                let received_diff = current_received.saturating_sub(*prev_received);
                (sent_diff, received_diff)
            } else {
                (0, 0)
            };
            
            previous_stats.insert(name.clone(), (current_sent, current_received));
            
            network_metrics.push(NetworkMetrics {
                interface_name: name.clone(),
                is_up: true, // Assume up if we can get data
                mac_address: String::new(), // Would need additional system calls
                ip_addresses: vec![], // Would need additional system calls
                bytes_sent: current_sent,
                bytes_received: current_received,
                packets_sent: data.total_packets_transmitted(),
                packets_received: data.total_packets_received(),
                errors_sent: 0, // Would need additional system calls
                errors_received: 0, // Would need additional system calls
                speed_mbps: None, // Would need additional system calls
                bytes_sent_rate: sent_rate,
                bytes_received_rate: received_rate,
            });
        }
        
        network_metrics
    }

    pub async fn collect_metrics(&self) -> Result<SystemMetrics, String> {
        println!("=== collect_metrics called ===");
        let mut sys = self.system.write().await;
        println!("Refreshing system data...");
        sys.refresh_all();
        println!("System data refreshed");
        
        // Drop the write lock before calling other methods to avoid deadlock
        drop(sys);
        
        println!("Getting system info for metrics...");
        let system_info = self.get_system_info().await?;
        println!("System info retrieved for metrics");
        
        // Get fresh system data for metrics
        let sys = self.system.read().await;
        
        // CPU metrics
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let per_core_usage: Vec<f32> = sys.cpus().iter()
            .map(|cpu| cpu.cpu_usage())
            .collect();
        
        println!("CPU usage - Global: {}%, Per-thread: {:?}", cpu_usage, per_core_usage);
        println!("CPU count: {}", sys.cpus().len());
        println!("Physical cores: {}", sys.physical_core_count().unwrap_or(0));
        
        let load_avg = System::load_average();
        println!("Load average - 1min: {}, 5min: {}, 15min: {}", load_avg.one, load_avg.five, load_avg.fifteen);
        
        // On Windows, load average might not be available, so we'll use a fallback
        let load_average = if load_avg.one == 0.0 && load_avg.five == 0.0 && load_avg.fifteen == 0.0 {
            // Fallback: calculate a simple load based on CPU usage
            let avg_cpu = per_core_usage.iter().sum::<f32>() / per_core_usage.len() as f32;
            let load_factor = avg_cpu / 100.0;
            [load_factor, load_factor * 0.8, load_factor * 0.6]
        } else {
            [load_avg.one as f32, load_avg.five as f32, load_avg.fifteen as f32]
        };
        
        let cpu_metrics = CpuMetrics {
            usage_percent: cpu_usage,
            frequency_mhz: sys.cpus().first()
                .map(|cpu| cpu.frequency())
                .unwrap_or(0),
            per_core_usage,
            temperature: None, // Would need sensors crate
            load_average,
            processes_total: sys.processes().len(),
            processes_running: sys.processes().values()
                .filter(|p| matches!(p.status(), ProcessStatus::Run))
                .count(),
            context_switches: 0, // Would need additional system calls
            interrupts: 0, // Would need additional system calls
        };
        
        // Memory metrics
        let memory_metrics = MemoryMetrics {
            total_bytes: sys.total_memory() * 1024,
            used_bytes: sys.used_memory() * 1024,
            available_bytes: sys.available_memory() * 1024,
            cached_bytes: 0, // Would need additional system calls
            swap_total_bytes: sys.total_swap() * 1024,
            swap_used_bytes: sys.used_swap() * 1024,
            usage_percent: (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0,
            swap_usage_percent: if sys.total_swap() > 0 {
                (sys.used_swap() as f32 / sys.total_swap() as f32) * 100.0
            } else {
                0.0
            },
        };
        
        // Get enhanced metrics
        let disk_metrics = self.get_disk_metrics().await;
        let network_metrics = self.get_network_metrics().await;
        let gpu_metrics = self.get_gpu_metrics().await;
        
        // Top processes
        let mut processes: Vec<ProcessMetrics> = sys.processes().values()
            .map(|process| ProcessMetrics {
                pid: process.pid().as_u32(),
                name: process.name().to_string(),
                cpu_usage_percent: process.cpu_usage(),
                memory_bytes: process.memory() * 1024,
                memory_percent: (process.memory() as f32 / sys.total_memory() as f32) * 100.0,
                disk_read_bytes: 0, // Would need additional system calls
                disk_write_bytes: 0, // Would need additional system calls
                status: format!("{:?}", process.status()),
                threads: 1, // Would need additional system calls
                start_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
            })
            .collect();
        
        processes.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());
        processes.truncate(10);
        
        println!("Creating SystemMetrics object...");
        let metrics = SystemMetrics {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
            system_info,
            cpu: cpu_metrics,
            memory: memory_metrics,
            gpus: gpu_metrics,
            disks: disk_metrics,
            networks: network_metrics,
            top_processes: processes,
        };
        println!("SystemMetrics created successfully with {} processes", metrics.top_processes.len());
        println!("Returning metrics from collect_metrics");
        Ok(metrics)
    }
    
    pub async fn start_monitoring(&self) {
        println!("=== start_monitoring called in monitoring service ===");
        let system = self.system.clone();
        let callback = self.metrics_callback.clone();
        
        println!("Spawning monitoring task...");
        tokio::spawn(async move {
            println!("Monitoring task started, setting up interval...");
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            println!("Monitoring interval set to 1 second");
            
            loop {
                interval.tick().await;
                
                let service = MonitoringService {
                    system: system.clone(),
                    metrics_callback: callback.clone(),
                    previous_network_stats: Arc::new(RwLock::new(HashMap::new())),
                };
                
                match service.collect_metrics().await {
                    Ok(metrics) => {
                        if let Some(cb) = callback.read().await.as_ref() {
                            println!("Calling metrics callback with {} processes", metrics.top_processes.len());
                            cb(metrics);
                        } else {
                            println!("No metrics callback set");
                        }
                    }
                    Err(e) => {
                        println!("Error collecting metrics: {}", e);
                    }
                }
            }
        });
        println!("Monitoring task spawned successfully");
    }
}