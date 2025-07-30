use crate::backend::ProcessMonitor;
use crate::core::{Monitor, MonitorConfig, MonitorState, MetricType, MetricValue};


#[tokio::test]
async fn test_process_monitor_initialization() {
    let mut monitor = ProcessMonitor::new();
    assert_eq!(monitor.name(), "Process Monitor");
    assert_eq!(monitor.state(), MonitorState::Uninitialized);
    
    let config = MonitorConfig::default();
    let result = monitor.initialize(config).await;
    assert!(result.is_ok());
    assert_eq!(monitor.state(), MonitorState::Running);
}

#[tokio::test]
async fn test_process_monitor_collection() {
    let mut monitor = ProcessMonitor::new();
    let config = MonitorConfig::default();
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    // Should have process metrics
    assert!(!metrics.is_empty());
    
    // Verify process metrics exist
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::ProcessCpu)));
    assert!(metrics.iter().any(|m| matches!(m.metric_type, MetricType::ProcessMemory)));
    
    // Verify process data is valid
    for metric in metrics.iter() {
        match metric.metric_type {
            MetricType::ProcessCpu => {
                if let MetricValue::Float(cpu) = metric.value {
                    assert!(cpu >= 0.0);
                }
            }
            MetricType::ProcessMemory => {
                if let MetricValue::Unsigned(memory) = metric.value {
                    assert!(memory > 0);
                }
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_process_monitor_features() {
    let monitor = ProcessMonitor::new();
    
    assert!(monitor.supports_feature("process_list"));
    assert!(monitor.supports_feature("process_cpu"));
    assert!(monitor.supports_feature("process_memory"));
    assert!(!monitor.supports_feature("gpu_usage"));
}

#[tokio::test]
async fn test_top_processes_limit() {
    let mut monitor = ProcessMonitor::new();
    let config = MonitorConfig {
        top_processes_count: Some(5),
        ..Default::default()
    };
    
    monitor.initialize(config).await.unwrap();
    let metrics = monitor.collect().await.unwrap();
    
    // Count unique PIDs
    let pids: std::collections::HashSet<_> = metrics.iter()
        .filter_map(|m| m.tags.get("pid"))
        .collect();
    
    // Should not exceed the configured limit
    assert!(pids.len() <= 5);
}