use crate::backend::MemoryMonitor;
use crate::core::{Monitor, MonitorConfig, MonitorState, MetricType, MetricValue};
use rstest::*;

#[tokio::test]
async fn test_memory_monitor_initialization() {
    let mut monitor = MemoryMonitor::new();
    assert_eq!(monitor.name(), "Memory Monitor");
    assert_eq!(monitor.state(), MonitorState::Uninitialized);
    
    let config = MonitorConfig::default();
    let result = monitor.initialize(config).await;
    assert!(result.is_ok());
    assert_eq!(monitor.state(), MonitorState::Running);
}

#[tokio::test]
async fn test_memory_monitor_collection() {
    let mut monitor = MemoryMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    // Verify memory usage metrics exist
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::MemoryUsage)));
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::MemoryAvailable)));
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::SwapUsage)));
    
    // Verify memory values are reasonable
    for metric in metrics.iter() {
        match &metric.metric_type {
            MetricType::MemoryUsage => {
                if metric.tags.is_empty() {
                    if let MetricValue::Float(usage) = metric.value {
                        assert!(usage >= 0.0 && usage <= 100.0);
                    }
                }
            }
            MetricType::MemoryAvailable => {
                if let MetricValue::Unsigned(bytes) = metric.value {
                    assert!(bytes > 0);
                }
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_memory_monitor_features() {
    let monitor = MemoryMonitor::new();
    
    assert!(monitor.supports_feature("memory_usage"));
    assert!(monitor.supports_feature("memory_available"));
    assert!(monitor.supports_feature("swap_usage"));
    assert!(!monitor.supports_feature("cpu_usage"));
}

#[tokio::test]
async fn test_memory_metrics_consistency() {
    let mut monitor = MemoryMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    let mut total_memory = 0u64;
    let mut used_memory = 0u64;
    let mut available_memory = 0u64;
    
    for metric in metrics.iter() {
        if let Some(mem_type) = metric.tags.get("type") {
            if let MetricValue::Unsigned(bytes) = metric.value {
                match mem_type.as_str() {
                    "total" => total_memory = bytes,
                    "used" => used_memory = bytes,
                    _ => {}
                }
            }
        } else if matches!(metric.metric_type, MetricType::MemoryAvailable) {
            if let MetricValue::Unsigned(bytes) = metric.value {
                available_memory = bytes;
            }
        }
    }
    
    // Verify memory values make sense
    assert!(total_memory > 0);
    assert!(used_memory > 0);
    assert!(available_memory > 0);
    assert!(used_memory <= total_memory);
    assert!(available_memory <= total_memory);
}