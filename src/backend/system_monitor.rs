use crate::backend::{cpu_monitor::CpuMonitor, gpu_monitor::GpuMonitor, memory_monitor::MemoryMonitor, storage_monitor::StorageMonitor};
use crate::core::{Monitor, MonitorConfig, MetricType, MetricValue};
use std::collections::HashMap;


/// Struct for monitoring system resources
pub struct SystemMonitor;

impl SystemMonitor {
    /// Collects system metrics asynchronously
    pub async fn collect_metrics() -> HashMap<String, String> {
        let mut metrics = HashMap::new();

        // Initialize monitors
        let mut cpu_monitor = CpuMonitor::new();
        let mut memory_monitor = MemoryMonitor::new();
        let mut storage_monitor = StorageMonitor::new();
        let mut gpu_monitor = GpuMonitor::new();

        // Initialize with default config
        let config = MonitorConfig::default();
        let _ = cpu_monitor.initialize(config.clone()).await;
        let _ = memory_monitor.initialize(config.clone()).await;
        let _ = storage_monitor.initialize(config.clone()).await;
        let _ = gpu_monitor.initialize(config.clone()).await;

        // Collect CPU metrics
        if let Ok(cpu_metrics) = cpu_monitor.collect().await {
            for metric in cpu_metrics {
                match metric.metric_type {
                    MetricType::CpuUsage => {
                        if metric.tags.is_empty() {  // Global CPU usage
                            if let MetricValue::Float(usage) = metric.value {
                                metrics.insert("CPU Usage".to_string(), format!("{:.2}%", usage));
                            }
                        }
                    }
                    MetricType::CpuFrequency => {
                        if let MetricValue::Unsigned(freq) = metric.value {
                            metrics.insert("CPU Frequency".to_string(), format!("{} MHz", freq));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Collect Memory metrics
        if let Ok(memory_metrics) = memory_monitor.collect().await {
            let mut total_bytes = 0u64;
            let mut used_bytes = 0u64;
            let mut available_bytes = 0u64;
            let mut usage_percent = 0.0f64;

            for metric in memory_metrics {
                match metric.metric_type {
                    MetricType::MemoryUsage => {
                        if let Some(tag_type) = metric.tags.get("type") {
                            if let MetricValue::Unsigned(bytes) = metric.value {
                                match tag_type.as_str() {
                                    "total" => total_bytes = bytes,
                                    "used" => used_bytes = bytes,
                                    _ => {}
                                }
                            }
                        } else if let MetricValue::Float(percent) = metric.value {
                            usage_percent = percent;
                        }
                    }
                    MetricType::MemoryAvailable => {
                        if let MetricValue::Unsigned(bytes) = metric.value {
                            available_bytes = bytes;
                        }
                    }
                    _ => {}
                }
            }

            // Convert bytes to GB
            let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

            metrics.insert("Memory Total".to_string(), format!("{:.2} GB", total_gb));
            metrics.insert("Memory Available".to_string(), format!("{:.2} GB", available_gb));
            metrics.insert("Memory Used".to_string(), format!("{:.2} GB", used_gb));
            metrics.insert("Memory Usage %".to_string(), format!("{:.2}%", usage_percent));
        }

        // Collect Storage metrics
        if let Ok(storage_metrics) = storage_monitor.collect().await {
            let mut total_bytes = 0u64;
            let mut available_bytes = 0u64;

            for metric in storage_metrics {
                match metric.metric_type {
                    MetricType::DiskSpace => {
                        if let Some(tag_type) = metric.tags.get("type") {
                            if let MetricValue::Unsigned(bytes) = metric.value {
                                match tag_type.as_str() {
                                    "total" => total_bytes += bytes,
                                    "available" => available_bytes += bytes,
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Convert bytes to GB
            let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

            metrics.insert("Storage Total".to_string(), format!("{:.2} GB", total_gb));
            metrics.insert("Storage Available".to_string(), format!("{:.2} GB", available_gb));
        }

        // Collect GPU metrics
        if let Ok(gpu_metrics) = gpu_monitor.collect().await {
            for metric in gpu_metrics {
                match metric.metric_type {
                    MetricType::GpuUsage => {
                        if let MetricValue::Float(usage) = metric.value {
                            let gpu_name = metric.tags.get("name").unwrap_or(&"GPU".to_string()).clone();
                            metrics.insert(format!("{} Usage", gpu_name), format!("{:.2}%", usage));
                        }
                    }
                    MetricType::GpuMemoryUsage => {
                        if let MetricValue::Float(usage) = metric.value {
                            let gpu_name = metric.tags.get("name").unwrap_or(&"GPU".to_string()).clone();
                            metrics.insert(format!("{} Memory Usage", gpu_name), format!("{:.2}%", usage));
                        }
                    }
                    MetricType::GpuTemperature => {
                        if let MetricValue::Float(temp) = metric.value {
                            let gpu_name = metric.tags.get("name").unwrap_or(&"GPU".to_string()).clone();
                            metrics.insert(format!("{} Temperature", gpu_name), format!("{:.1}°C", temp));
                        }
                    }
                    _ => {}
                }
            }
        }

        metrics
    }
}