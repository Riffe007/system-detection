use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::{System, Disks, Networks, ProcessStatus};

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
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics_callback: Arc::new(RwLock::new(None)),
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

    pub async fn collect_metrics(&self) -> Result<SystemMetrics, String> {
        println!("=== collect_metrics called ===");
        let mut sys = self.system.write().await;
        println!("Refreshing system data...");
        sys.refresh_all();
        println!("System data refreshed");
        
        // Drop the write lock before calling get_system_info to avoid deadlock
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
        
        let load_avg = System::load_average();
        
        let cpu_metrics = CpuMetrics {
            usage_percent: cpu_usage,
            frequency_mhz: sys.cpus().first()
                .map(|cpu| cpu.frequency())
                .unwrap_or(0),
            per_core_usage,
            temperature: None, // Would need sensors crate
            load_average: [load_avg.one as f32, load_avg.five as f32, load_avg.fifteen as f32],
            processes_total: sys.processes().len(),
            processes_running: sys.processes().values()
                .filter(|p| matches!(p.status(), ProcessStatus::Run))
                .count(),
            context_switches: 0, // Not available in sysinfo 
            interrupts: 0, // Not available in sysinfo
        };
        
        // Memory metrics
        let memory_metrics = MemoryMetrics {
            total_bytes: sys.total_memory() * 1024,
            used_bytes: sys.used_memory() * 1024,
            available_bytes: sys.available_memory() * 1024,
            cached_bytes: 0, // Not available in sysinfo
            swap_total_bytes: sys.total_swap() * 1024,
            swap_used_bytes: sys.used_swap() * 1024,
            usage_percent: (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0,
            swap_usage_percent: if sys.total_swap() > 0 {
                (sys.used_swap() as f32 / sys.total_swap() as f32) * 100.0
            } else {
                0.0
            },
        };
        
        // Disk metrics
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
                read_bytes_per_sec: 0, // Not available in sysinfo
                write_bytes_per_sec: 0, // Not available in sysinfo
                io_operations_per_sec: 0, // Not available in sysinfo
            })
            .collect();
        
        // Network metrics
        let networks = Networks::new_with_refreshed_list();
        let network_metrics: Vec<NetworkMetrics> = networks.iter()
            .map(|(name, data)| NetworkMetrics {
                interface_name: name.clone(),
                is_up: true, // Assume up if we can get data
                mac_address: String::new(), // Not available in sysinfo
                ip_addresses: vec![], // Not available in sysinfo
                bytes_sent: data.total_transmitted(),
                bytes_received: data.total_received(),
                packets_sent: data.total_packets_transmitted(),
                packets_received: data.total_packets_received(),
                errors_sent: 0, // Not available in sysinfo
                errors_received: 0, // Not available in sysinfo
                speed_mbps: None, // Not available in sysinfo
                bytes_sent_rate: 0, // Not available in sysinfo
                bytes_received_rate: 0, // Not available in sysinfo
            })
            .collect();
        
        // Top processes
        let mut processes: Vec<ProcessMetrics> = sys.processes().values()
            .map(|process| ProcessMetrics {
                pid: process.pid().as_u32(),
                name: process.name().to_string(),
                cpu_usage_percent: process.cpu_usage(),
                memory_bytes: process.memory() * 1024,
                memory_percent: (process.memory() as f32 / sys.total_memory() as f32) * 100.0,
                disk_read_bytes: 0, // Not available in sysinfo
                disk_write_bytes: 0, // Not available in sysinfo
                status: format!("{:?}", process.status()),
                threads: 1, // Default to 1 thread since sysinfo doesn't provide thread count easily
                start_time: chrono::Utc::now().timestamp().to_string(), // Not available in sysinfo
            })
            .collect();
        
        processes.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());
        processes.truncate(10);
        
        // GPU metrics (empty for now since sysinfo doesn't provide GPU data)
        let gpu_metrics: Vec<GpuMetrics> = vec![];
        
        println!("Creating SystemMetrics object...");
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now().timestamp().to_string(),
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