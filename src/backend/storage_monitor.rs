use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::SystemTime;
use sysinfo::{System, RefreshKind, Disks};

use crate::core::{
    DiskMetrics, Metric, MetricType, MetricValue, Monitor, MonitorConfig, MonitorError,
    MonitorState, Result,
};

pub struct StorageMonitor {
    state: Arc<RwLock<MonitorState>>,
    config: Arc<RwLock<MonitorConfig>>,
    #[allow(dead_code)] // Will be used for future platform-specific optimizations
    system: Arc<RwLock<System>>,
    metrics_history: Arc<RwLock<VecDeque<Vec<DiskMetrics>>>>,
    last_update: Arc<RwLock<SystemTime>>,
    previous_io_stats: Arc<RwLock<HashMap<String, IoStats>>>,
}

#[derive(Clone, Debug)]
struct IoStats {
    #[allow(dead_code)] // Used in platform-specific implementations
    read_bytes: u64,
    #[allow(dead_code)] // Used in platform-specific implementations
    write_bytes: u64,
    #[allow(dead_code)] // Used in platform-specific implementations
    timestamp: SystemTime,
}

impl StorageMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MonitorState::Uninitialized)),
            config: Arc::new(RwLock::new(MonitorConfig::default())),
            system: Arc::new(RwLock::new(System::new_with_specifics(RefreshKind::everything()))),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            last_update: Arc::new(RwLock::new(SystemTime::now())),
            previous_io_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn collect_storage_metrics(&self) -> Result<Vec<DiskMetrics>> {
        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh();

        let mut metrics = Vec::new();
        let mut current_io_stats = HashMap::new();
        let now = SystemTime::now();
        let previous_stats = self.previous_io_stats.read();

        for disk in disks.iter() {
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let device_name = disk.name().to_string_lossy().to_string();
            
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space();
            let used_bytes = total_bytes.saturating_sub(available_bytes);
            let usage_percent = if total_bytes > 0 {
                (used_bytes as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };

            // Calculate I/O rates
            let (read_bytes_per_sec, write_bytes_per_sec) = self.calculate_io_rates(
                &device_name,
                &previous_stats,
                &mut current_io_stats,
                now,
            );

            let fs_type = disk.file_system()
                .to_string_lossy()
                .to_string();

            metrics.push(DiskMetrics {
                mount_point,
                device_name: device_name.clone(),
                fs_type,
                total_bytes,
                used_bytes,
                available_bytes,
                usage_percent,
                read_bytes_per_sec,
                write_bytes_per_sec,
                io_operations_per_sec: 0, // Platform-specific, would need additional implementation
            });
        }

        // Update previous I/O stats for next calculation
        *self.previous_io_stats.write() = current_io_stats;

        Ok(metrics)
    }

    fn calculate_io_rates(
        &self,
        _device_name: &str,
        _previous_stats: &HashMap<String, IoStats>,
        _current_stats: &mut HashMap<String, IoStats>,
        _now: SystemTime,
    ) -> (u64, u64) {
        // Platform-specific I/O statistics
        #[cfg(target_os = "linux")]
        {
            if let Ok((read_bytes, write_bytes)) = self.read_linux_io_stats(device_name) {
                let io_stats = IoStats {
                    read_bytes,
                    write_bytes,
                    timestamp: now,
                };

                if let Some(prev_stats) = previous_stats.get(device_name) {
                    if let Ok(duration) = now.duration_since(prev_stats.timestamp) {
                        let secs = duration.as_secs_f64();
                        if secs > 0.0 {
                            let read_rate = ((read_bytes.saturating_sub(prev_stats.read_bytes)) as f64 / secs) as u64;
                            let write_rate = ((write_bytes.saturating_sub(prev_stats.write_bytes)) as f64 / secs) as u64;
                            
                            current_stats.insert(device_name.to_string(), io_stats);
                            return (read_rate, write_rate);
                        }
                    }
                }

                current_stats.insert(device_name.to_string(), io_stats);
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows implementation would use Performance Counters or WMI
            // This is a placeholder
        }

        #[cfg(target_os = "macos")]
        {
            // macOS implementation would use IOKit
            // This is a placeholder
        }

        (0, 0)
    }

    #[cfg(target_os = "linux")]
    fn read_linux_io_stats(&self, device_name: &str) -> Result<(u64, u64)> {
        use std::fs;
        
        // Extract the base device name (e.g., sda from sda1)
        let base_device = device_name.trim_start_matches("/dev/")
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>();

        let stat_path = format!("/sys/block/{}/stat", base_device);
        
        if let Ok(contents) = fs::read_to_string(&stat_path) {
            let parts: Vec<&str> = contents.split_whitespace().collect();
            if parts.len() >= 6 {
                // Format: reads read_sectors writes written_sectors
                let read_sectors = parts[2].parse::<u64>().unwrap_or(0);
                let written_sectors = parts[6].parse::<u64>().unwrap_or(0);
                
                // Convert sectors to bytes (typically 512 bytes per sector)
                let read_bytes = read_sectors * 512;
                let write_bytes = written_sectors * 512;
                
                return Ok((read_bytes, write_bytes));
            }
        }

        Ok((0, 0))
    }

    fn update_history(&self, metrics: Vec<DiskMetrics>) {
        let mut history = self.metrics_history.write();
        let config = self.config.read();
        
        history.push_back(metrics);
        
        // Remove old metrics based on retention policy
        let max_entries = (config.retain_history_seconds * 1000 / config.interval_ms) as usize;
        while history.len() > max_entries {
            history.pop_front();
        }
    }
}

#[async_trait]
impl Monitor for StorageMonitor {
    fn name(&self) -> &str {
        "Storage Monitor"
    }

    fn state(&self) -> MonitorState {
        *self.state.read()
    }

    async fn initialize(&mut self, config: MonitorConfig) -> Result<()> {
        *self.state.write() = MonitorState::Initializing;
        *self.config.write() = config;
        
        // Initialize disk info
        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh();
        
        // Collect initial I/O stats
        let _ = self.collect_storage_metrics()?;
        
        *self.state.write() = MonitorState::Running;
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        match self.state() {
            MonitorState::Running => return Ok(()),
            MonitorState::Uninitialized => {
                return Err(MonitorError::NotInitialized);
            }
            _ => {}
        }
        
        *self.state.write() = MonitorState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.state.write() = MonitorState::Stopped;
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        *self.state.write() = MonitorState::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        *self.state.write() = MonitorState::Running;
        Ok(())
    }

    async fn collect(&mut self) -> Result<Vec<Metric>> {
        if self.state() != MonitorState::Running {
            return Err(MonitorError::NotInitialized);
        }

        let disk_metrics = self.collect_storage_metrics()?;
        self.update_history(disk_metrics.clone());
        *self.last_update.write() = SystemTime::now();

        let mut metrics = Vec::new();
        
        for disk in disk_metrics.iter() {
            // Disk usage percentage
            metrics.push(Metric::new(
                MetricType::DiskUsage,
                MetricValue::Float(disk.usage_percent as f64),
                "%",
            ).with_tag("mount", &disk.mount_point)
             .with_tag("device", &disk.device_name));
            
            // Disk space metrics
            metrics.push(Metric::new(
                MetricType::DiskSpace,
                MetricValue::Unsigned(disk.used_bytes),
                "bytes",
            ).with_tag("mount", &disk.mount_point)
             .with_tag("type", "used"));
            
            metrics.push(Metric::new(
                MetricType::DiskSpace,
                MetricValue::Unsigned(disk.available_bytes),
                "bytes",
            ).with_tag("mount", &disk.mount_point)
             .with_tag("type", "available"));
            
            metrics.push(Metric::new(
                MetricType::DiskSpace,
                MetricValue::Unsigned(disk.total_bytes),
                "bytes",
            ).with_tag("mount", &disk.mount_point)
             .with_tag("type", "total"));
            
            // I/O metrics
            if disk.read_bytes_per_sec > 0 || disk.write_bytes_per_sec > 0 {
                metrics.push(Metric::new(
                    MetricType::DiskIo,
                    MetricValue::Unsigned(disk.read_bytes_per_sec),
                    "bytes/s",
                ).with_tag("mount", &disk.mount_point)
                 .with_tag("operation", "read"));
                
                metrics.push(Metric::new(
                    MetricType::DiskIo,
                    MetricValue::Unsigned(disk.write_bytes_per_sec),
                    "bytes/s",
                ).with_tag("mount", &disk.mount_point)
                 .with_tag("operation", "write"));
            }
        }
        
        Ok(metrics)
    }

    async fn get_current_metrics(&self) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            let mut metrics = Vec::new();
            
            for disk in latest.iter() {
                metrics.push(Metric::new(
                    MetricType::DiskUsage,
                    MetricValue::Float(disk.usage_percent as f64),
                    "%",
                ).with_tag("mount", &disk.mount_point));
                
                metrics.push(Metric::new(
                    MetricType::DiskIo,
                    MetricValue::Unsigned(disk.read_bytes_per_sec + disk.write_bytes_per_sec),
                    "bytes/s",
                ).with_tag("mount", &disk.mount_point));
            }
            
            Ok(metrics)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_historical_metrics(&self, _duration_seconds: u64) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        let mut metrics = Vec::new();
        
        for disk_list in history.iter() {
            for disk in disk_list.iter() {
                metrics.push(Metric::new(
                    MetricType::DiskUsage,
                    MetricValue::Float(disk.usage_percent as f64),
                    "%",
                ).with_tag("mount", &disk.mount_point));
            }
        }
        
        Ok(metrics)
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, 
            "disk_usage" | "disk_space" | "disk_io" | 
            "disk_read" | "disk_write"
        )
    }
}