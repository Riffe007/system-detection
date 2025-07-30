use crate::backend::CpuMonitor;
use crate::core::{Monitor, MonitorConfig, MonitorState, MetricType};
// Removed unused rstest import
use std::time::Duration;

#[tokio::test]
async fn test_cpu_monitor_initialization() {
    let mut monitor = CpuMonitor::new();
    assert_eq!(monitor.name(), "CPU Monitor");
    assert_eq!(monitor.state(), MonitorState::Uninitialized);
    
    let config = MonitorConfig::default();
    let result = monitor.initialize(config).await;
    assert!(result.is_ok());
    assert_eq!(monitor.state(), MonitorState::Running);
}

#[tokio::test]
async fn test_cpu_monitor_lifecycle() {
    let mut monitor = CpuMonitor::new();
    let config = MonitorConfig::default();
    
    // Initialize
    monitor.initialize(config).await.unwrap();
    assert_eq!(monitor.state(), MonitorState::Running);
    
    // Pause
    monitor.pause().await.unwrap();
    assert_eq!(monitor.state(), MonitorState::Paused);
    
    // Resume
    monitor.resume().await.unwrap();
    assert_eq!(monitor.state(), MonitorState::Running);
    
    // Stop
    monitor.stop().await.unwrap();
    assert_eq!(monitor.state(), MonitorState::Stopped);
}

#[tokio::test]
async fn test_cpu_monitor_collection() {
    let mut monitor = CpuMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    
    // Collect metrics
    let metrics = monitor.collect().await.unwrap();
    
    // Verify we have CPU usage metric
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::CpuUsage)));
    
    // Verify we have CPU frequency metric
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::CpuFrequency)));
    
    // Verify we have process count metrics
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::ProcessCount)));
    
    // Verify per-core metrics exist
    let per_core_metrics: Vec<_> = metrics.iter()
        .filter(|m| matches!(m.metric_type, MetricType::CpuUsage) && m.tags.contains_key("core"))
        .collect();
    assert!(!per_core_metrics.is_empty());
}

#[tokio::test]
async fn test_cpu_monitor_features() {
    let monitor = CpuMonitor::new();
    
    assert!(monitor.supports_feature("cpu_usage"));
    assert!(monitor.supports_feature("cpu_frequency"));
    assert!(monitor.supports_feature("per_core_usage"));
    assert!(monitor.supports_feature("process_count"));
    assert!(!monitor.supports_feature("gpu_usage"));
}

#[tokio::test]
async fn test_cpu_monitor_historical_metrics() {
    let mut monitor = CpuMonitor::new();
    let config = MonitorConfig {
        interval_ms: 100,
        retain_history_seconds: 10,
        ..Default::default()
    };
    
    monitor.initialize(config).await.unwrap();
    
    // Collect metrics multiple times
    for _ in 0..3 {
        monitor.collect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Get historical metrics
    let historical = monitor.get_historical_metrics(5).await.unwrap();
    assert!(!historical.is_empty());
}

#[tokio::test]
async fn test_cpu_usage_bounds() {
    let mut monitor = CpuMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    for metric in metrics.iter() {
        if let MetricType::CpuUsage = metric.metric_type {
            if let crate::core::MetricValue::Float(usage) = metric.value {
                assert!(usage >= 0.0 && usage <= 100.0);
            }
        }
    }
}