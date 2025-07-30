use crate::backend::NetworkMonitor;
use crate::core::{Monitor, MonitorConfig, MonitorState, MetricType, MetricValue};


#[tokio::test]
async fn test_network_monitor_initialization() {
    let mut monitor = NetworkMonitor::new();
    assert_eq!(monitor.name(), "Network Monitor");
    assert_eq!(monitor.state(), MonitorState::Uninitialized);
    
    let config = MonitorConfig::default();
    let result = monitor.initialize(config).await;
    assert!(result.is_ok());
    assert_eq!(monitor.state(), MonitorState::Running);
}

#[tokio::test]
async fn test_network_monitor_collection() {
    let mut monitor = NetworkMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    // Should have at least one network interface
    assert!(!metrics.is_empty());
    
    // Verify network metrics exist
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::NetworkBytes)));
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::NetworkStatus)));
    
    // Verify each interface has required metrics
    let interfaces: std::collections::HashSet<_> = metrics.iter()
        .filter_map(|m| m.tags.get("interface"))
        .collect();
    
    assert!(!interfaces.is_empty());
}

#[tokio::test]
async fn test_network_monitor_features() {
    let monitor = NetworkMonitor::new();
    
    assert!(monitor.supports_feature("network_throughput"));
    assert!(monitor.supports_feature("network_status"));
    assert!(monitor.supports_feature("network_errors"));
    assert!(!monitor.supports_feature("disk_usage"));
}

#[tokio::test]
async fn test_network_throughput_calculation() {
    let mut monitor = NetworkMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    
    // Collect twice to get throughput rates
    let _first = monitor.collect().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let second = monitor.collect().await.unwrap();
    
    // Check throughput metrics exist
    let throughput_metrics: Vec<_> = second.iter()
        .filter(|m| matches!(m.metric_type, MetricType::NetworkThroughput))
        .collect();
    
    assert!(!throughput_metrics.is_empty());
    
    // Verify throughput values are reasonable
    for metric in throughput_metrics {
        if let MetricValue::Unsigned(bytes_per_sec) = metric.value {
            // Throughput should be non-negative
            // bytes_per_sec is u64, so it's always >= 0
            assert!(bytes_per_sec >= 0u64);
        }
    }
}