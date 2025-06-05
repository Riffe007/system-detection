use crate::backend::GpuMonitor;
use crate::core::{Monitor, MonitorConfig, MonitorState, MetricType};
use rstest::*;

#[tokio::test]
async fn test_gpu_monitor_initialization() {
    let mut monitor = GpuMonitor::new();
    assert_eq!(monitor.name(), "GPU Monitor");
    assert_eq!(monitor.state(), MonitorState::Uninitialized);
    
    let config = MonitorConfig::default();
    let result = monitor.initialize(config).await;
    assert!(result.is_ok());
    assert_eq!(monitor.state(), MonitorState::Running);
}

#[tokio::test]
async fn test_gpu_monitor_collection() {
    let mut monitor = GpuMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    // GPU metrics might be empty if no GPU is detected
    // But the collection should not fail
    assert!(metrics.is_empty() || metrics.iter().any(|m| 
        matches!(m.metric_type, 
            MetricType::GpuUsage | 
            MetricType::GpuTemperature | 
            MetricType::GpuMemoryUsage |
            MetricType::GpuPower |
            MetricType::GpuFanSpeed
        )
    ));
}

#[tokio::test]
async fn test_gpu_monitor_features() {
    let monitor = GpuMonitor::new();
    
    assert!(monitor.supports_feature("gpu_usage"));
    assert!(monitor.supports_feature("gpu_temperature"));
    assert!(monitor.supports_feature("gpu_memory"));
    assert!(monitor.supports_feature("gpu_power"));
    assert!(monitor.supports_feature("gpu_clock"));
    assert!(monitor.supports_feature("gpu_fan_speed"));
    assert!(!monitor.supports_feature("cpu_usage"));
}

#[cfg(feature = "nvidia")]
#[tokio::test]
async fn test_nvidia_gpu_detection() {
    let mut monitor = GpuMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    
    // This test will only pass if NVIDIA GPU is present
    // We check that initialization doesn't panic
    let metrics = monitor.collect().await;
    assert!(metrics.is_ok());
}