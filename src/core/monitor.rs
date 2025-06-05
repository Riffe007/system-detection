use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::{Metric, MonitorError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub enabled: bool,
    pub interval_ms: u64,
    pub retain_history_seconds: u64,
    pub alert_thresholds: std::collections::HashMap<String, f64>,
    pub max_processes: Option<usize>,
    pub top_processes_count: Option<usize>,
    pub include_loopback: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 1000,
            retain_history_seconds: 3600,
            alert_thresholds: std::collections::HashMap::new(),
            max_processes: Some(100),
            top_processes_count: Some(10),
            include_loopback: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorState {
    Uninitialized,
    Initializing,
    Running,
    Paused,
    Stopped,
    Error,
}

#[async_trait]
pub trait Monitor: Send + Sync {
    fn name(&self) -> &str;
    
    fn state(&self) -> MonitorState;
    
    async fn initialize(&mut self, config: MonitorConfig) -> Result<()>;
    
    async fn start(&mut self) -> Result<()>;
    
    async fn stop(&mut self) -> Result<()>;
    
    async fn pause(&mut self) -> Result<()>;
    
    async fn resume(&mut self) -> Result<()>;
    
    async fn collect(&mut self) -> Result<Vec<Metric>>;
    
    async fn get_current_metrics(&self) -> Result<Vec<Metric>>;
    
    async fn get_historical_metrics(&self, duration_seconds: u64) -> Result<Vec<Metric>>;
    
    fn supports_feature(&self, feature: &str) -> bool;
}

pub type SharedMonitor = Arc<RwLock<Box<dyn Monitor>>>;

#[derive(Clone)]
pub struct MonitorManager {
    monitors: Arc<RwLock<std::collections::HashMap<String, SharedMonitor>>>,
}

impl MonitorManager {
    pub fn new() -> Self {
        Self {
            monitors: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn register_monitor(&self, name: String, monitor: Box<dyn Monitor>) -> Result<()> {
        let mut monitors = self.monitors.write().await;
        
        if monitors.contains_key(&name) {
            return Err(MonitorError::InvalidConfig(format!(
                "Monitor '{}' already registered",
                name
            )));
        }
        
        monitors.insert(name, Arc::new(RwLock::new(monitor)));
        Ok(())
    }

    pub async fn unregister_monitor(&self, name: &str) -> Result<()> {
        let mut monitors = self.monitors.write().await;
        
        if let Some(monitor) = monitors.remove(name) {
            let mut m = monitor.write().await;
            m.stop().await?;
        }
        
        Ok(())
    }

    pub async fn get_monitor(&self, name: &str) -> Option<SharedMonitor> {
        let monitors = self.monitors.read().await;
        monitors.get(name).cloned()
    }

    pub async fn start_all(&self) -> Result<()> {
        let monitors = self.monitors.read().await;
        
        for (_, monitor) in monitors.iter() {
            let mut m = monitor.write().await;
            m.start().await?;
        }
        
        Ok(())
    }

    pub async fn stop_all(&self) -> Result<()> {
        let monitors = self.monitors.read().await;
        
        for (_, monitor) in monitors.iter() {
            let mut m = monitor.write().await;
            m.stop().await?;
        }
        
        Ok(())
    }

    pub async fn collect_all_metrics(&self) -> Result<std::collections::HashMap<String, Vec<Metric>>> {
        let monitors = self.monitors.read().await;
        let mut all_metrics = std::collections::HashMap::new();
        
        for (name, monitor) in monitors.iter() {
            let mut m = monitor.write().await;
            match m.collect().await {
                Ok(metrics) => {
                    all_metrics.insert(name.clone(), metrics);
                }
                Err(e) => {
                    tracing::error!("Failed to collect metrics from {}: {}", name, e);
                }
            }
        }
        
        Ok(all_metrics)
    }
}