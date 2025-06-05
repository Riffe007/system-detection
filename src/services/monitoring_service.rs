use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

use crate::backend::{
    CpuMonitor, MemoryMonitor, GpuMonitor, StorageMonitor, NetworkMonitor, ProcessMonitor
};
use crate::core::{
    MonitorConfig, MonitorManager, MonitoringInterval, Result, SystemMetrics, SystemInfo,
    CpuMetrics, MemoryMetrics, GpuMetrics, DiskMetrics, NetworkMetrics, ProcessMetrics,
    Metric, MetricType, MetricValue,
};

pub struct MonitoringService {
    manager: Arc<MonitorManager>,
    metrics_sender: broadcast::Sender<SystemMetrics>,
    monitoring_interval: Arc<RwLock<MonitoringInterval>>,
    system_info: Arc<RwLock<Option<SystemInfo>>>,
    is_running: Arc<RwLock<bool>>,
}

impl MonitoringService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        
        Self {
            manager: Arc::new(MonitorManager::new()),
            metrics_sender: tx,
            monitoring_interval: Arc::new(RwLock::new(MonitoringInterval::default())),
            system_info: Arc::new(RwLock::new(None)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Initialize system info
        let system_info = self.collect_system_info().await?;
        *self.system_info.write().await = Some(system_info);

        // Register all monitors
        self.manager.register_monitor(
            "cpu".to_string(),
            Box::new(CpuMonitor::new()),
        ).await?;

        self.manager.register_monitor(
            "memory".to_string(),
            Box::new(MemoryMonitor::new()),
        ).await?;

        self.manager.register_monitor(
            "gpu".to_string(),
            Box::new(GpuMonitor::new()),
        ).await?;

        self.manager.register_monitor(
            "storage".to_string(),
            Box::new(StorageMonitor::new()),
        ).await?;

        self.manager.register_monitor(
            "network".to_string(),
            Box::new(NetworkMonitor::new()),
        ).await?;

        self.manager.register_monitor(
            "process".to_string(),
            Box::new(ProcessMonitor::new()),
        ).await?;

        // Initialize all monitors with default config
        let config = MonitorConfig::default();
        
        for monitor_name in ["cpu", "memory", "gpu", "storage", "network", "process"] {
            if let Some(monitor) = self.manager.get_monitor(monitor_name).await {
                let mut monitor = monitor.write().await;
                monitor.initialize(config.clone()).await?;
            }
        }

        Ok(())
    }

    async fn collect_system_info(&self) -> Result<SystemInfo> {
        use os_info;
        use sysinfo::{System, SystemExt, CpuExt};
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let info = os_info::get();
        let cpu_info = sys.global_cpu_info();
        
        Ok(SystemInfo {
            hostname: hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            os_name: info.os_type().to_string(),
            os_version: info.version().to_string(),
            kernel_version: sys.kernel_version().unwrap_or_default(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_brand: cpu_info.brand().to_string(),
            cpu_cores: sys.physical_core_count().unwrap_or(0),
            cpu_threads: sys.cpus().len(),
            total_memory: sys.total_memory() * 1024, // Convert KB to bytes
            boot_time: std::time::SystemTime::now() - Duration::from_secs(sys.uptime()),
        })
    }

    pub async fn start(&self) -> Result<()> {
        *self.is_running.write().await = true;
        
        // Start all monitors
        self.manager.start_all().await?;
        
        // Start collection loop
        let manager = self.manager.clone();
        let sender = self.metrics_sender.clone();
        let system_info = self.system_info.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(500));
            
            loop {
                interval.tick().await;
                
                if !*is_running.read().await {
                    break;
                }
                
                if let Err(e) = Self::collect_and_broadcast(
                    &manager, 
                    &sender, 
                    &system_info,
                ).await {
                    tracing::error!("Failed to collect metrics: {}", e);
                }
            }
        });
        
        Ok(())
    }

    async fn collect_and_broadcast(
        manager: &Arc<MonitorManager>,
        sender: &broadcast::Sender<SystemMetrics>,
        system_info: &Arc<RwLock<Option<SystemInfo>>>,
    ) -> Result<()> {
        let all_metrics = manager.collect_all_metrics().await?;
        
        if let Some(info) = system_info.read().await.clone() {
            // Parse collected metrics into structured format
            let mut cpu_metrics = CpuMetrics::default();
            let mut memory_metrics = MemoryMetrics::default();
            let mut gpu_metrics = Vec::new();
            let mut disk_metrics = Vec::new();
            let mut network_metrics = Vec::new();
            let mut process_metrics = Vec::new();

            // Process CPU metrics
            if let Some(metrics) = all_metrics.get("cpu") {
                for metric in metrics {
                    match metric.metric_type {
                        MetricType::CpuUsage => {
                            if metric.tags.is_empty() {
                                if let MetricValue::Float(v) = metric.value {
                                    cpu_metrics.usage_percent = v as f32;
                                }
                            } else if let Some(core_str) = metric.tags.get("core") {
                                if let Ok(core_idx) = core_str.parse::<usize>() {
                                    if let MetricValue::Float(v) = metric.value {
                                        if core_idx >= cpu_metrics.per_core_usage.len() {
                                            cpu_metrics.per_core_usage.resize(core_idx + 1, 0.0);
                                        }
                                        cpu_metrics.per_core_usage[core_idx] = v as f32;
                                    }
                                }
                            }
                        }
                        MetricType::CpuFrequency => {
                            if let MetricValue::Unsigned(v) = metric.value {
                                cpu_metrics.frequency_mhz = v;
                            }
                        }
                        MetricType::ProcessCount => {
                            if let Some(t) = metric.tags.get("type") {
                                if let MetricValue::Integer(v) = metric.value {
                                    match t.as_str() {
                                        "total" => cpu_metrics.processes_total = v as usize,
                                        "running" => cpu_metrics.processes_running = v as usize,
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Process Memory metrics
            if let Some(metrics) = all_metrics.get("memory") {
                for metric in metrics {
                    match metric.metric_type {
                        MetricType::MemoryUsage => {
                            if metric.tags.is_empty() {
                                if let MetricValue::Float(v) = metric.value {
                                    memory_metrics.usage_percent = v as f32;
                                }
                            } else if let Some(t) = metric.tags.get("type") {
                                if let MetricValue::Unsigned(v) = metric.value {
                                    match t.as_str() {
                                        "used" => memory_metrics.used_bytes = v,
                                        "total" => memory_metrics.total_bytes = v,
                                        _ => {}
                                    }
                                }
                            }
                        }
                        MetricType::MemoryAvailable => {
                            if let MetricValue::Unsigned(v) = metric.value {
                                memory_metrics.available_bytes = v;
                            }
                        }
                        MetricType::SwapUsage => {
                            if let MetricValue::Float(v) = metric.value {
                                memory_metrics.swap_usage_percent = v as f32;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Process GPU metrics
            if let Some(metrics) = all_metrics.get("gpu") {
                let mut gpu_map = std::collections::HashMap::new();
                
                for metric in metrics {
                    if let Some(gpu_id) = metric.tags.get("gpu") {
                        let gpu = gpu_map.entry(gpu_id.clone()).or_insert_with(|| {
                            GpuMetrics {
                                name: metric.tags.get("name").cloned().unwrap_or_default(),
                                driver_version: String::new(),
                                temperature_celsius: 0.0,
                                usage_percent: 0.0,
                                memory_total_bytes: 0,
                                memory_used_bytes: 0,
                                memory_usage_percent: 0.0,
                                power_watts: 0.0,
                                fan_speed_percent: None,
                                clock_mhz: 0,
                                memory_clock_mhz: 0,
                            }
                        });
                        
                        match metric.metric_type {
                            MetricType::GpuUsage => {
                                if let MetricValue::Float(v) = metric.value {
                                    gpu.usage_percent = v as f32;
                                }
                            }
                            MetricType::GpuTemperature => {
                                if let MetricValue::Float(v) = metric.value {
                                    gpu.temperature_celsius = v as f32;
                                }
                            }
                            MetricType::GpuMemoryUsage => {
                                if let MetricValue::Float(v) = metric.value {
                                    gpu.memory_usage_percent = v as f32;
                                }
                            }
                            MetricType::GpuPower => {
                                if let MetricValue::Float(v) = metric.value {
                                    gpu.power_watts = v as f32;
                                }
                            }
                            MetricType::GpuFanSpeed => {
                                if let MetricValue::Float(v) = metric.value {
                                    gpu.fan_speed_percent = Some(v as f32);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                gpu_metrics.extend(gpu_map.into_values());
            }

            // Process Disk metrics
            if let Some(metrics) = all_metrics.get("storage") {
                let mut disk_map = std::collections::HashMap::new();
                
                for metric in metrics {
                    if let Some(mount) = metric.tags.get("mount") {
                        let disk = disk_map.entry(mount.clone()).or_insert_with(|| {
                            DiskMetrics {
                                mount_point: mount.clone(),
                                device_name: metric.tags.get("device").cloned().unwrap_or_default(),
                                fs_type: String::new(),
                                total_bytes: 0,
                                used_bytes: 0,
                                available_bytes: 0,
                                usage_percent: 0.0,
                                read_bytes_per_sec: 0,
                                write_bytes_per_sec: 0,
                                io_operations_per_sec: 0,
                            }
                        });
                        
                        match metric.metric_type {
                            MetricType::DiskUsage => {
                                if let MetricValue::Float(v) = metric.value {
                                    disk.usage_percent = v as f32;
                                }
                            }
                            MetricType::DiskSpace => {
                                if let Some(t) = metric.tags.get("type") {
                                    if let MetricValue::Unsigned(v) = metric.value {
                                        match t.as_str() {
                                            "used" => disk.used_bytes = v,
                                            "available" => disk.available_bytes = v,
                                            "total" => disk.total_bytes = v,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            MetricType::DiskIo => {
                                if let Some(op) = metric.tags.get("operation") {
                                    if let MetricValue::Unsigned(v) = metric.value {
                                        match op.as_str() {
                                            "read" => disk.read_bytes_per_sec = v,
                                            "write" => disk.write_bytes_per_sec = v,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                disk_metrics.extend(disk_map.into_values());
            }

            // Process Network metrics
            if let Some(metrics) = all_metrics.get("network") {
                let mut net_map = std::collections::HashMap::new();
                
                for metric in metrics {
                    if let Some(iface) = metric.tags.get("interface") {
                        let net = net_map.entry(iface.clone()).or_insert_with(|| {
                            NetworkMetrics {
                                interface_name: iface.clone(),
                                is_up: false,
                                mac_address: String::from("00:00:00:00:00:00"),
                                ip_addresses: Vec::new(),
                                bytes_sent: 0,
                                bytes_received: 0,
                                packets_sent: 0,
                                packets_received: 0,
                                errors_sent: 0,
                                errors_received: 0,
                                speed_mbps: None,
                                bytes_sent_rate: 0,
                                bytes_received_rate: 0,
                            }
                        });
                        
                        match metric.metric_type {
                            MetricType::NetworkThroughput => {
                                if let Some(dir) = metric.tags.get("direction") {
                                    if let MetricValue::Unsigned(v) = metric.value {
                                        match dir.as_str() {
                                            "sent" => net.bytes_sent_rate = v,
                                            "received" => net.bytes_received_rate = v,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            MetricType::NetworkBytes => {
                                if let Some(dir) = metric.tags.get("direction") {
                                    if let MetricValue::Unsigned(v) = metric.value {
                                        match dir.as_str() {
                                            "sent" => net.bytes_sent = v,
                                            "received" => net.bytes_received = v,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            MetricType::NetworkStatus => {
                                if let MetricValue::Boolean(v) = metric.value {
                                    net.is_up = v;
                                }
                            }
                            MetricType::NetworkSpeed => {
                                if let MetricValue::Unsigned(v) = metric.value {
                                    net.speed_mbps = Some(v);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                network_metrics.extend(net_map.into_values());
            }

            // Process Process metrics
            if let Some(metrics) = all_metrics.get("process") {
                let mut top_processes = Vec::new();
                
                for metric in metrics {
                    if let Some(pid_str) = metric.tags.get("pid") {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            if let Some(name) = metric.tags.get("name") {
                                let mut process = ProcessMetrics {
                                    pid,
                                    name: name.clone(),
                                    cpu_usage_percent: 0.0,
                                    memory_bytes: 0,
                                    memory_percent: 0.0,
                                    disk_read_bytes: 0,
                                    disk_write_bytes: 0,
                                    status: String::from("Running"),
                                    threads: 1,
                                    start_time: std::time::SystemTime::now(),
                                };
                                
                                match metric.metric_type {
                                    MetricType::ProcessCpu => {
                                        if let MetricValue::Float(v) = metric.value {
                                            process.cpu_usage_percent = v as f32;
                                        }
                                    }
                                    MetricType::ProcessMemory => {
                                        if let MetricValue::Unsigned(v) = metric.value {
                                            process.memory_bytes = v;
                                        }
                                    }
                                    _ => {}
                                }
                                
                                if let Some(existing) = top_processes.iter_mut().find(|p| p.pid == pid) {
                                    if process.cpu_usage_percent > 0.0 {
                                        existing.cpu_usage_percent = process.cpu_usage_percent;
                                    }
                                    if process.memory_bytes > 0 {
                                        existing.memory_bytes = process.memory_bytes;
                                    }
                                } else if process.cpu_usage_percent > 0.0 || process.memory_bytes > 0 {
                                    top_processes.push(process);
                                }
                            }
                        }
                    }
                }
                
                // Sort by CPU usage and take top 10
                top_processes.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());
                top_processes.truncate(10);
                process_metrics = top_processes;
            }

            // Build SystemMetrics from collected data
            let metrics = SystemMetrics {
                timestamp: std::time::SystemTime::now(),
                system_info: info,
                cpu: cpu_metrics,
                memory: memory_metrics,
                gpus: gpu_metrics,
                disks: disk_metrics,
                networks: network_metrics,
                top_processes: process_metrics,
            };
            
            // Send metrics to subscribers
            let _ = sender.send(metrics);
        }
        
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        *self.is_running.write().await = false;
        self.manager.stop_all().await?;
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemMetrics> {
        self.metrics_sender.subscribe()
    }

    pub async fn set_monitoring_interval(&self, interval: MonitoringInterval) {
        *self.monitoring_interval.write().await = interval;
    }

    pub async fn get_system_info(&self) -> Option<SystemInfo> {
        self.system_info.read().await.clone()
    }
}

// Add Default implementations for metrics types
impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            usage_percent: 0.0,
            frequency_mhz: 0,
            temperature_celsius: None,
            load_average: [0.0; 3],
            per_core_usage: Vec::new(),
            processes_running: 0,
            processes_total: 0,
            context_switches: 0,
            interrupts: 0,
        }
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            cached_bytes: 0,
            swap_total_bytes: 0,
            swap_used_bytes: 0,
            usage_percent: 0.0,
            swap_usage_percent: 0.0,
        }
    }
}