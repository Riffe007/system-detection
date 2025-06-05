use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use sysinfo::{System, SystemExt};

use crate::core::{
    MemoryMetrics, Metric, MetricType, MetricValue, Monitor, MonitorConfig, MonitorError,
    MonitorState, Result,
};

pub struct MemoryMonitor {
    state: Arc<RwLock<MonitorState>>,
    config: Arc<RwLock<MonitorConfig>>,
    system: Arc<RwLock<System>>,
    metrics_history: Arc<RwLock<VecDeque<MemoryMetrics>>>,
    last_update: Arc<RwLock<SystemTime>>,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MonitorState::Uninitialized)),
            config: Arc::new(RwLock::new(MonitorConfig::default())),
            system: Arc::new(RwLock::new(System::new_all())),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            last_update: Arc::new(RwLock::new(SystemTime::now())),
        }
    }

    fn collect_memory_metrics(&self) -> Result<MemoryMetrics> {
        let mut system = self.system.write();
        system.refresh_memory();

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = system.available_memory();
        let total_swap = system.total_swap();
        let used_swap = system.used_swap();

        let usage_percent = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        let swap_usage_percent = if total_swap > 0 {
            (used_swap as f32 / total_swap as f32) * 100.0
        } else {
            0.0
        };

        Ok(MemoryMetrics {
            total_bytes: total_memory * 1024, // Convert KB to bytes
            used_bytes: used_memory * 1024,
            available_bytes: available_memory * 1024,
            cached_bytes: 0, // Platform-specific, will implement later
            swap_total_bytes: total_swap * 1024,
            swap_used_bytes: used_swap * 1024,
            usage_percent,
            swap_usage_percent,
        })
    }

    fn update_history(&self, metrics: MemoryMetrics) {
        let mut history = self.metrics_history.write();
        let config = self.config.read();
        
        history.push_back(metrics);
        
        // Remove old metrics based on retention policy
        let max_entries = (config.retain_history_seconds * 1000 / config.interval_ms) as usize;
        while history.len() > max_entries {
            history.pop_front();
        }
    }
}

#[async_trait]
impl Monitor for MemoryMonitor {
    fn name(&self) -> &str {
        "Memory Monitor"
    }

    fn state(&self) -> MonitorState {
        *self.state.read()
    }

    async fn initialize(&mut self, config: MonitorConfig) -> Result<()> {
        *self.state.write() = MonitorState::Initializing;
        *self.config.write() = config;
        
        // Initialize system info
        let mut system = self.system.write();
        system.refresh_memory();
        
        *self.state.write() = MonitorState::Running;
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        match self.state() {
            MonitorState::Running => return Ok(()),
            MonitorState::Uninitialized => {
                return Err(MonitorError::NotInitialized);
            }
            _ => {}
        }
        
        *self.state.write() = MonitorState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.state.write() = MonitorState::Stopped;
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        *self.state.write() = MonitorState::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        *self.state.write() = MonitorState::Running;
        Ok(())
    }

    async fn collect(&mut self) -> Result<Vec<Metric>> {
        if self.state() != MonitorState::Running {
            return Err(MonitorError::NotInitialized);
        }

        let memory_metrics = self.collect_memory_metrics()?;
        self.update_history(memory_metrics.clone());
        *self.last_update.write() = SystemTime::now();

        let mut metrics = Vec::new();
        
        metrics.push(Metric::new(
            MetricType::MemoryUsage,
            MetricValue::Float(memory_metrics.usage_percent as f64),
            "%",
        ));
        
        metrics.push(Metric::new(
            MetricType::MemoryAvailable,
            MetricValue::Unsigned(memory_metrics.available_bytes),
            "bytes",
        ));
        
        metrics.push(Metric::new(
            MetricType::SwapUsage,
            MetricValue::Float(memory_metrics.swap_usage_percent as f64),
            "%",
        ));
        
        // Add detailed memory metrics
        metrics.push(Metric::new(
            MetricType::MemoryUsage,
            MetricValue::Unsigned(memory_metrics.used_bytes),
            "bytes",
        ).with_tag("type", "used"));
        
        metrics.push(Metric::new(
            MetricType::MemoryUsage,
            MetricValue::Unsigned(memory_metrics.total_bytes),
            "bytes",
        ).with_tag("type", "total"));
        
        Ok(metrics)
    }

    async fn get_current_metrics(&self) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            let mut metrics = Vec::new();
            
            metrics.push(Metric::new(
                MetricType::MemoryUsage,
                MetricValue::Float(latest.usage_percent as f64),
                "%",
            ));
            
            metrics.push(Metric::new(
                MetricType::MemoryAvailable,
                MetricValue::Unsigned(latest.available_bytes),
                "bytes",
            ));
            
            Ok(metrics)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_historical_metrics(&self, duration_seconds: u64) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        let mut metrics = Vec::new();
        
        for memory_metrics in history.iter() {
            metrics.push(Metric::new(
                MetricType::MemoryUsage,
                MetricValue::Float(memory_metrics.usage_percent as f64),
                "%",
            ));
        }
        
        Ok(metrics)
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, "memory_usage" | "memory_available" | "swap_usage")
    }
}
