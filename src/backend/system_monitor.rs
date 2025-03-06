use crate::backend::{cpu_monitor::CpuMonitor, gpu_monitor::GpuMonitor, memory_monitor::MemoryMonitor, storage_monitor::StorageMonitor};
use std::collections::HashMap;
use tokio::task;

/// Struct for monitoring system resources
pub struct SystemMonitor;

impl SystemMonitor {
    /// Collects system metrics asynchronously
    pub async fn collect_metrics() -> HashMap<String, String> {
        let cpu_usage_map = task::spawn_blocking(|| CpuMonitor::get_usage()).await.unwrap();
        let (total_memory, available_memory, used_memory) = task::spawn_blocking(|| MemoryMonitor::get_usage()).await.unwrap();
        let (total_storage, available_storage) = task::spawn_blocking(|| StorageMonitor::get_usage()).await.unwrap();

        let mut metrics = HashMap::new();

        // ✅ Extracts CPU usage properly
        if let Some(cpu_usage) = cpu_usage_map.get("CPU Usage") {
            metrics.insert("CPU Usage".to_string(), format!("{:.2}%", cpu_usage));
        }

        // ✅ Memory is now displayed in **GB** correctly
        metrics.insert("Memory Total".to_string(), format!("{} GB", total_memory)); 
        metrics.insert("Memory Available".to_string(), format!("{} GB", available_memory)); 
        metrics.insert("Memory Used".to_string(), format!("{} GB", used_memory));
        metrics.insert("Memory Usage %".to_string(), format!("{:.2}%", (used_memory as f32 / total_memory as f32) * 100.0));

        // ✅ Storage metrics are robust
        metrics.insert("Storage Total".to_string(), format!("{} GB", total_storage)); 
        metrics.insert("Storage Available".to_string(), format!("{} GB", available_storage)); 

        // ✅ GPU information included
        if let Some(gpu_info) = GpuMonitor::detect() {
            for (key, value) in gpu_info {
                metrics.insert(key, value);
            }
        }

        metrics
    }
}
