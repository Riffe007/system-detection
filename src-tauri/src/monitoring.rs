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
pub struct DiskMetrics {
    pub mount_point: String,
    pub device_name: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interface_name: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_usage_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: i64,
    pub system_info: SystemInfo,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
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
        let mut sys = self.system.write().await;
        sys.refresh_all();
        
        let info = os_info::get();
        
        Ok(SystemInfo {
            hostname: hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            os_name: info.os_type().to_string(),
            os_version: info.version().to_string(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_brand: sys.cpus().first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_default(),
            cpu_cores: sys.physical_core_count().unwrap_or(0),
            cpu_threads: sys.cpus().len(),
            total_memory: sys.total_memory(),
            boot_time: System::boot_time() as i64,
        })
    }

    pub async fn collect_metrics(&self) -> Result<SystemMetrics, String> {
        let mut sys = self.system.write().await;
        sys.refresh_all();
        
        let system_info = self.get_system_info().await?;
        
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
            })
            .collect();
        
        // Network metrics
        let networks = Networks::new_with_refreshed_list();
        let network_metrics: Vec<NetworkMetrics> = networks.iter()
            .map(|(name, data)| NetworkMetrics {
                interface_name: name.clone(),
                bytes_sent: data.total_transmitted(),
                bytes_received: data.total_received(),
                packets_sent: data.total_packets_transmitted(),
                packets_received: data.total_packets_received(),
            })
            .collect();
        
        // Top processes
        let mut processes: Vec<ProcessMetrics> = sys.processes().values()
            .map(|process| ProcessMetrics {
                pid: process.pid().as_u32(),
                name: process.name().to_string(),
                cpu_usage_percent: process.cpu_usage(),
                memory_bytes: process.memory() * 1024,
            })
            .collect();
        
        processes.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());
        processes.truncate(10);
        
        Ok(SystemMetrics {
            timestamp: chrono::Utc::now().timestamp(),
            system_info,
            cpu: cpu_metrics,
            memory: memory_metrics,
            disks: disk_metrics,
            networks: network_metrics,
            top_processes: processes,
        })
    }
    
    pub async fn start_monitoring(&self) {
        let system = self.system.clone();
        let callback = self.metrics_callback.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                let service = MonitoringService {
                    system: system.clone(),
                    metrics_callback: callback.clone(),
                };
                
                if let Ok(metrics) = service.collect_metrics().await {
                    if let Some(cb) = callback.read().await.as_ref() {
                        cb(metrics);
                    }
                }
            }
        });
    }
}