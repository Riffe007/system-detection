use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use sysinfo::{System, RefreshKind, ProcessRefreshKind, Process, Pid, ProcessStatus, DiskUsage};

use crate::core::{
    ProcessMetrics, Metric, MetricType, MetricValue, Monitor, MonitorConfig, MonitorError,
    MonitorState, Result,
};

pub struct ProcessMonitor {
    state: Arc<RwLock<MonitorState>>,
    config: Arc<RwLock<MonitorConfig>>,
    system: Arc<RwLock<System>>,
    metrics_history: Arc<RwLock<VecDeque<Vec<ProcessMetrics>>>>,
    last_update: Arc<RwLock<SystemTime>>,
    process_cpu_history: Arc<RwLock<HashMap<u32, f32>>>,
    sort_by: Arc<RwLock<ProcessSortBy>>,
    filter: Arc<RwLock<ProcessFilter>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessSortBy {
    Cpu,
    Memory,
    Name,
    Pid,
}

#[derive(Debug, Clone)]
pub struct ProcessFilter {
    pub min_cpu_percent: f32,
    pub min_memory_bytes: u64,
    pub name_pattern: Option<String>,
    pub include_system: bool,
}

impl Default for ProcessFilter {
    fn default() -> Self {
        Self {
            min_cpu_percent: 0.0,
            min_memory_bytes: 0,
            name_pattern: None,
            include_system: true,
        }
    }
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MonitorState::Uninitialized)),
            config: Arc::new(RwLock::new(MonitorConfig::default())),
            system: Arc::new(RwLock::new(System::new_with_specifics(RefreshKind::everything()))),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            last_update: Arc::new(RwLock::new(SystemTime::now())),
            process_cpu_history: Arc::new(RwLock::new(HashMap::new())),
            sort_by: Arc::new(RwLock::new(ProcessSortBy::Cpu)),
            filter: Arc::new(RwLock::new(ProcessFilter::default())),
        }
    }

    pub fn set_sort_by(&self, sort_by: ProcessSortBy) {
        *self.sort_by.write() = sort_by;
    }

    pub fn set_filter(&self, filter: ProcessFilter) {
        *self.filter.write() = filter;
    }

    fn collect_process_metrics(&self) -> Result<Vec<ProcessMetrics>> {
        let mut system = self.system.write();
        system.refresh_processes_specifics(ProcessRefreshKind::everything());
        
        let mut metrics = Vec::new();
        let filter = self.filter.read().clone();
        let total_memory = system.total_memory() * 1024; // Convert to bytes
        
        for (pid, process) in system.processes() {
            let pid_u32 = pid.as_u32();
            let name = process.name().to_string();
            
            // Apply name filter
            if let Some(pattern) = &filter.name_pattern {
                if !name.to_lowercase().contains(&pattern.to_lowercase()) {
                    continue;
                }
            }
            
            let cpu_usage = process.cpu_usage();
            let memory_bytes = process.memory() * 1024; // Convert KB to bytes
            
            // Apply CPU and memory filters
            if cpu_usage < filter.min_cpu_percent || memory_bytes < filter.min_memory_bytes {
                continue;
            }
            
            // Skip system processes if configured
            if !filter.include_system && self.is_system_process(&name, pid_u32) {
                continue;
            }
            
            let memory_percent = if total_memory > 0 {
                (memory_bytes as f32 / total_memory as f32) * 100.0
            } else {
                0.0
            };
            
            let disk_usage = process.disk_usage();
            let status = process.status().to_string();
            
            // Get process start time
            let start_time = SystemTime::UNIX_EPOCH + Duration::from_secs(process.start_time());
            
            // Get thread count
            #[cfg(target_os = "linux")]
            let threads = self.get_linux_thread_count(pid_u32).unwrap_or(1);
            #[cfg(not(target_os = "linux"))]
            let threads = 1; // Default fallback
            
            metrics.push(ProcessMetrics {
                pid: pid_u32,
                name,
                cpu_usage_percent: cpu_usage,
                memory_bytes,
                memory_percent,
                disk_read_bytes: disk_usage.read_bytes,
                disk_write_bytes: disk_usage.written_bytes,
                status,
                threads,
                start_time,
            });
        }
        
        // Sort processes based on selected criteria
        self.sort_processes(&mut metrics);
        
        // Limit to top N processes if configured
        let max_processes = self.config.read().max_processes.unwrap_or(100);
        metrics.truncate(max_processes);
        
        Ok(metrics)
    }

    fn is_system_process(&self, name: &str, pid: u32) -> bool {
        // Common system process patterns
        let system_patterns = [
            "kernel", "systemd", "init", "kworker", "ksoftirqd",
            "migration", "rcu_", "watchdog", "kthread", "kdevtmpfs",
            "netns", "kauditd", "khungtaskd", "oom_reaper", "writeback",
            "kcompactd", "ksmd", "khugepaged", "crypto", "kintegrityd",
            "kblockd", "devfreq", "watchdogd", "systemd-", "dbus",
        ];
        
        // PID 0 and 1 are always system processes
        if pid <= 1 {
            return true;
        }
        
        let name_lower = name.to_lowercase();
        system_patterns.iter().any(|pattern| name_lower.contains(pattern))
    }

    fn sort_processes(&self, processes: &mut Vec<ProcessMetrics>) {
        let sort_by = *self.sort_by.read();
        
        match sort_by {
            ProcessSortBy::Cpu => {
                processes.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());
            }
            ProcessSortBy::Memory => {
                processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
            }
            ProcessSortBy::Name => {
                processes.sort_by(|a, b| a.name.cmp(&b.name));
            }
            ProcessSortBy::Pid => {
                processes.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn get_linux_thread_count(&self, pid: u32) -> Option<u32> {
        use std::fs;
        
        let task_path = format!("/proc/{}/task", pid);
        if let Ok(entries) = fs::read_dir(&task_path) {
            let count = entries.filter_map(|e| e.ok()).count() as u32;
            Some(count)
        } else {
            None
        }
    }

    fn update_history(&self, metrics: Vec<ProcessMetrics>) {
        let mut history = self.metrics_history.write();
        let config = self.config.read();
        
        // Update CPU history for better accuracy
        let mut cpu_history = self.process_cpu_history.write();
        for process in &metrics {
            cpu_history.insert(process.pid, process.cpu_usage_percent);
        }
        
        // Clean up old CPU history entries
        cpu_history.retain(|pid, _| metrics.iter().any(|p| p.pid == *pid));
        
        history.push_back(metrics);
        
        // Remove old metrics based on retention policy
        let max_entries = (config.retain_history_seconds * 1000 / config.interval_ms) as usize;
        while history.len() > max_entries {
            history.pop_front();
        }
    }
}

#[async_trait]
impl Monitor for ProcessMonitor {
    fn name(&self) -> &str {
        "Process Monitor"
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

        let process_metrics = self.collect_process_metrics()?;
        self.update_history(process_metrics.clone());
        *self.last_update.write() = SystemTime::now();

        let mut metrics = Vec::new();
        
        // Add aggregated metrics
        let total_processes = process_metrics.len();
        let total_cpu_usage: f32 = process_metrics.iter().map(|p| p.cpu_usage_percent).sum();
        let total_memory_usage: u64 = process_metrics.iter().map(|p| p.memory_bytes).sum();
        
        metrics.push(Metric::new(
            MetricType::ProcessCount,
            MetricValue::Integer(total_processes as i64),
            "count",
        ).with_tag("type", "monitored"));
        
        metrics.push(Metric::new(
            MetricType::ProcessCpuTotal,
            MetricValue::Float(total_cpu_usage as f64),
            "%",
        ));
        
        metrics.push(Metric::new(
            MetricType::ProcessMemoryTotal,
            MetricValue::Unsigned(total_memory_usage),
            "bytes",
        ));
        
        // Add individual process metrics for top processes
        let top_count = self.config.read().top_processes_count.unwrap_or(10);
        for (idx, process) in process_metrics.iter().take(top_count).enumerate() {
            let rank = (idx + 1).to_string();
            
            metrics.push(Metric::new(
                MetricType::ProcessCpu,
                MetricValue::Float(process.cpu_usage_percent as f64),
                "%",
            ).with_tag("pid", process.pid.to_string())
             .with_tag("name", &process.name)
             .with_tag("rank", &rank));
            
            metrics.push(Metric::new(
                MetricType::ProcessMemory,
                MetricValue::Unsigned(process.memory_bytes),
                "bytes",
            ).with_tag("pid", process.pid.to_string())
             .with_tag("name", &process.name)
             .with_tag("rank", &rank));
            
            if process.disk_read_bytes > 0 || process.disk_write_bytes > 0 {
                metrics.push(Metric::new(
                    MetricType::ProcessDiskIo,
                    MetricValue::Unsigned(process.disk_read_bytes),
                    "bytes",
                ).with_tag("pid", process.pid.to_string())
                 .with_tag("name", &process.name)
                 .with_tag("operation", "read"));
                
                metrics.push(Metric::new(
                    MetricType::ProcessDiskIo,
                    MetricValue::Unsigned(process.disk_write_bytes),
                    "bytes",
                ).with_tag("pid", process.pid.to_string())
                 .with_tag("name", &process.name)
                 .with_tag("operation", "write"));
            }
        }
        
        Ok(metrics)
    }

    async fn get_current_metrics(&self) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            let mut metrics = Vec::new();
            
            let total_cpu: f32 = latest.iter().map(|p| p.cpu_usage_percent).sum();
            metrics.push(Metric::new(
                MetricType::ProcessCpuTotal,
                MetricValue::Float(total_cpu as f64),
                "%",
            ));
            
            Ok(metrics)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_historical_metrics(&self, _duration_seconds: u64) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        let mut metrics = Vec::new();
        
        for process_list in history.iter() {
            let total_cpu: f32 = process_list.iter().map(|p| p.cpu_usage_percent).sum();
            metrics.push(Metric::new(
                MetricType::ProcessCpuTotal,
                MetricValue::Float(total_cpu as f64),
                "%",
            ));
        }
        
        Ok(metrics)
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, 
            "process_list" | "process_cpu" | "process_memory" | 
            "process_disk_io" | "process_filtering" | "process_sorting"
        )
    }
}

// Public API extensions for process monitoring
impl ProcessMonitor {
    pub async fn get_top_processes(&self, count: usize) -> Result<Vec<ProcessMetrics>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            Ok(latest.iter().take(count).cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    pub async fn find_process_by_name(&self, name: &str) -> Result<Vec<ProcessMetrics>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            Ok(latest.iter()
                .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
                .cloned()
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    pub async fn get_process_by_pid(&self, pid: u32) -> Result<Option<ProcessMetrics>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            Ok(latest.iter().find(|p| p.pid == pid).cloned())
        } else {
            Ok(None)
        }
    }
}