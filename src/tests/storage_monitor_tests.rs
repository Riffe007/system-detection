use crate::backend::StorageMonitor;
use crate::core::{Monitor, MonitorConfig, MonitorState, MetricType, MetricValue};
use rstest::*;

#[tokio::test]
async fn test_storage_monitor_initialization() {
    let mut monitor = StorageMonitor::new();
    assert_eq!(monitor.name(), "Storage Monitor");
    assert_eq!(monitor.state(), MonitorState::Uninitialized);
    
    let config = MonitorConfig::default();
    let result = monitor.initialize(config).await;
    assert!(result.is_ok());
    assert_eq!(monitor.state(), MonitorState::Running);
}

#[tokio::test]
async fn test_storage_monitor_collection() {
    let mut monitor = StorageMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    // Should have at least one disk
    assert!(!metrics.is_empty());
    
    // Verify disk metrics exist
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::DiskUsage)));
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::DiskSpace)));
    
    // Verify disk usage percentages are valid
    for metric in metrics.iter() {
        if matches!(metric.metric_type, MetricType::DiskUsage) {
            if let MetricValue::Float(usage) = metric.value {
                assert!(usage >= 0.0 && usage <= 100.0);
            }
        }
    }
}

#[tokio::test]
async fn test_storage_monitor_features() {
    let monitor = StorageMonitor::new();
    
    assert!(monitor.supports_feature("disk_usage"));
    assert!(monitor.supports_feature("disk_space"));
    assert!(monitor.supports_feature("disk_io"));
    assert!(!monitor.supports_feature("memory_usage"));
}

#[tokio::test]
async fn test_disk_space_consistency() {
    let mut monitor = StorageMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    // Group metrics by mount point
    let mut disk_stats = std::collections::HashMap::new();
    
    for metric in metrics.iter() {
        if let Some(mount) = metric.tags.get("mount") {
            let entry = disk_stats.entry(mount.clone()).or_insert((0u64, 0u64, 0u64));
            
            if matches!(metric.metric_type, MetricType::DiskSpace) {
                if let Some(space_type) = metric.tags.get("type") {
                    if let MetricValue::Unsigned(bytes) = metric.value {
                        match space_type.as_str() {
                            "total" => entry.0 = bytes,
                            "used" => entry.1 = bytes,
                            "available" => entry.2 = bytes,
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    
    // Verify consistency for each disk
    for (mount, (total, used, available)) in disk_stats {
        assert!(total > 0, "Disk {} has zero total space", mount);
        assert!(used <= total, "Disk {} used space exceeds total", mount);
        assert!(available <= total, "Disk {} available space exceeds total", mount);
    }
}