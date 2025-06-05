use crate::services::MonitoringService;
use crate::core::SystemInfo;

#[tokio::test]
async fn test_monitoring_service_lifecycle() {
    let service = MonitoringService::new();
    
    // Initialize service
    let init_result = service.initialize().await;
    assert!(init_result.is_ok());
    
    // Get system info
    let system_info = service.get_system_info().await;
    assert!(system_info.is_some());
    
    // Start monitoring
    let start_result = service.start().await;
    assert!(start_result.is_ok());
    
    // Subscribe to metrics
    let mut receiver = service.subscribe();
    
    // Wait for metrics
    tokio::select! {
        metrics = receiver.recv() => {
            assert!(metrics.is_ok());
            let metrics = metrics.unwrap();
            assert_eq!(metrics.system_info.hostname, system_info.unwrap().hostname);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
            panic!("Timeout waiting for metrics");
        }
    }
    
    // Stop monitoring
    let stop_result = service.stop().await;
    assert!(stop_result.is_ok());
}

#[tokio::test]
async fn test_system_info_collection() {
    let service = MonitoringService::new();
    service.initialize().await.unwrap();
    
    let info = service.get_system_info().await;
    assert!(info.is_some());
    
    let info = info.unwrap();
    assert!(!info.hostname.is_empty());
    assert!(!info.os_name.is_empty());
    assert!(!info.cpu_brand.is_empty());
    assert!(info.cpu_cores > 0);
    assert!(info.cpu_threads > 0);
    assert!(info.total_memory > 0);
}

#[tokio::test]
async fn test_metrics_collection_consistency() {
    let service = MonitoringService::new();
    service.initialize().await.unwrap();
    service.start().await.unwrap();
    
    let mut receiver = service.subscribe();
    
    // Collect multiple metrics
    let mut metrics_samples = Vec::new();
    for _ in 0..3 {
        if let Ok(metrics) = receiver.recv().await {
            metrics_samples.push(metrics);
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    
    assert!(metrics_samples.len() >= 2);
    
    // Verify consistency across samples
    for window in metrics_samples.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        
        // System info should be consistent
        assert_eq!(prev.system_info.hostname, curr.system_info.hostname);
        assert_eq!(prev.system_info.cpu_cores, curr.system_info.cpu_cores);
        
        // Timestamp should increase
        assert!(curr.timestamp > prev.timestamp);
    }
    
    service.stop().await.unwrap();
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    use mockall::mock;
    
    // Example of how to use mocks for testing
    mock! {
        MonitoringServiceMock {
            async fn initialize(&self) -> Result<(), String>;
            async fn start(&self) -> Result<(), String>;
            async fn stop(&self) -> Result<(), String>;
        }
    }
    
    #[tokio::test]
    async fn test_mock_service() {
        let mut mock = MockMonitoringServiceMock::new();
        
        mock.expect_initialize()
            .times(1)
            .returning(|| Ok(()));
            
        mock.expect_start()
            .times(1)
            .returning(|| Ok(()));
            
        mock.expect_stop()
            .times(1)
            .returning(|| Ok(()));
        
        assert!(mock.initialize().await.is_ok());
        assert!(mock.start().await.is_ok());
        assert!(mock.stop().await.is_ok());
    }
}