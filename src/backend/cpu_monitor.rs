use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use sysinfo::{System, CpuRefreshKind, RefreshKind};

use crate::core::{
    CpuMetrics, Metric, MetricType, MetricValue, Monitor, MonitorConfig, MonitorError,
    MonitorState, Result,
};

/// CPU monitoring implementation
/// 
/// Monitors CPU usage, frequency, temperature, load average, and per-core metrics.
/// 
/// # Features
/// 
/// - Overall CPU usage percentage
/// - Per-core usage tracking
/// - CPU frequency monitoring
/// - Temperature sensing (Linux only)
/// - Load average (1, 5, 15 minutes)
/// - Process count tracking
/// - Context switches and interrupts (Linux only)
/// 
/// # Example
/// 
/// ```rust,no_run
/// use system_monitor::backend::CpuMonitor;
/// use system_monitor::core::{Monitor, MonitorConfig};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut monitor = CpuMonitor::new();
/// monitor.initialize(MonitorConfig::default()).await?;
/// 
/// let metrics = monitor.collect().await?;
/// for metric in metrics {
///     println!("{}: {:?}", metric.metric_type, metric.value);
/// }
/// # Ok(())
/// # }
/// ```
pub struct CpuMonitor {
    state: Arc<RwLock<MonitorState>>,
    config: Arc<RwLock<MonitorConfig>>,
    system: Arc<RwLock<System>>,
    metrics_history: Arc<RwLock<VecDeque<CpuMetrics>>>,
    last_update: Arc<RwLock<SystemTime>>,
}

impl CpuMonitor {
    /// Creates a new CPU monitor instance
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MonitorState::Uninitialized)),
            config: Arc::new(RwLock::new(MonitorConfig::default())),
            system: Arc::new(RwLock::new(System::new_with_specifics(RefreshKind::everything()))),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            last_update: Arc::new(RwLock::new(SystemTime::now())),
        }
    }

    fn collect_cpu_metrics(&self) -> Result<CpuMetrics> {
        let mut system = self.system.write();
        system.refresh_cpu_specifics(CpuRefreshKind::everything());
        system.refresh_processes();

        let global_cpu = system.global_cpu_info();
        let cpus = system.cpus();
        
        let per_core_usage: Vec<f32> = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
        
        let load_avg = System::load_average();
        let load_average = [load_avg.one as f32, load_avg.five as f32, load_avg.fifteen as f32];

        let processes: Vec<_> = system.processes().values().collect();
        let processes_running = processes.iter().filter(|p| {
            matches!(p.status().to_string().as_str(), "Run" | "Running")
        }).count();

        // Get CPU temperature from sensors
        let temperature_celsius = {
            let sensors = super::sensors::SensorsManager::new();
            sensors.read_cpu_temperature().ok().flatten()
        };

        Ok(CpuMetrics {
            usage_percent: global_cpu.cpu_usage(),
            frequency_mhz: global_cpu.frequency(),
            temperature_celsius,
            load_average,
            per_core_usage,
            processes_running,
            processes_total: processes.len(),
            context_switches: self.read_context_switches().unwrap_or(0),
            interrupts: self.read_interrupts().unwrap_or(0),
        })
    }

    fn read_context_switches(&self) -> Option<u64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
                for line in stat.lines() {
                    if line.starts_with("ctxt") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return parts[1].parse::<u64>().ok();
                        }
                    }
                }
            }
        }
        None
    }

    fn read_interrupts(&self) -> Option<u64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
                for line in stat.lines() {
                    if line.starts_with("intr") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return parts[1].parse::<u64>().ok();
                        }
                    }
                }
            }
        }
        None
    }

    fn update_history(&self, metrics: CpuMetrics) {
        let mut history = self.metrics_history.write();
        let config = self.config.read();
        
        history.push_back(metrics);
        
        // Remove old metrics based on retention policy
        let cutoff_time = SystemTime::now() - Duration::from_secs(config.retain_history_seconds);
        let now = SystemTime::now();
        
        while history.len() > 0 {
            let age_secs = now.duration_since(*self.last_update.read()).unwrap_or_default().as_secs();
            if age_secs > config.retain_history_seconds {
                history.pop_front();
            } else {
                break;
            }
        }
    }
}

#[async_trait]
impl Monitor for CpuMonitor {
    fn name(&self) -> &str {
        "CPU Monitor"
    }

    fn state(&self) -> MonitorState {
        *self.state.read()
    }

    async fn initialize(&mut self, config: MonitorConfig) -> Result<()> {
        *self.state.write() = MonitorState::Initializing;
        *self.config.write() = config;
        
        // Initialize system info
        let mut system = self.system.write();
        system.refresh_all();
        
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

        let cpu_metrics = self.collect_cpu_metrics()?;
        self.update_history(cpu_metrics.clone());
        *self.last_update.write() = SystemTime::now();

        let mut metrics = Vec::new();
        
        metrics.push(Metric::new(
            MetricType::CpuUsage,
            MetricValue::Float(cpu_metrics.usage_percent as f64),
            "%",
        ));
        
        metrics.push(Metric::new(
            MetricType::CpuFrequency,
            MetricValue::Unsigned(cpu_metrics.frequency_mhz),
            "MHz",
        ));
        
        metrics.push(Metric::new(
            MetricType::ProcessCount,
            MetricValue::Integer(cpu_metrics.processes_total as i64),
            "count",
        ).with_tag("type", "total"));
        
        metrics.push(Metric::new(
            MetricType::ProcessCount,
            MetricValue::Integer(cpu_metrics.processes_running as i64),
            "count",
        ).with_tag("type", "running"));
        
        // Add per-core usage metrics
        for (i, usage) in cpu_metrics.per_core_usage.iter().enumerate() {
            metrics.push(Metric::new(
                MetricType::CpuUsage,
                MetricValue::Float(*usage as f64),
                "%",
            ).with_tag("core", i.to_string()));
        }
        
        Ok(metrics)
    }

    async fn get_current_metrics(&self) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            let mut metrics = Vec::new();
            
            metrics.push(Metric::new(
                MetricType::CpuUsage,
                MetricValue::Float(latest.usage_percent as f64),
                "%",
            ));
            
            Ok(metrics)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_historical_metrics(&self, duration_seconds: u64) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        let cutoff_time = SystemTime::now() - Duration::from_secs(duration_seconds);
        
        let mut metrics = Vec::new();
        
        for cpu_metrics in history.iter() {
            metrics.push(Metric::new(
                MetricType::CpuUsage,
                MetricValue::Float(cpu_metrics.usage_percent as f64),
                "%",
            ));
        }
        
        Ok(metrics)
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, "cpu_usage" | "cpu_frequency" | "per_core_usage" | "process_count")
    }
}
