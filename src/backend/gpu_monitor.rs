use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::core::{
    GpuMetrics, Metric, MetricType, MetricValue, Monitor, MonitorConfig, MonitorError,
    MonitorState, Result,
};

#[cfg(feature = "nvidia")]
use nvml_wrapper::{Nvml, Device as NvmlDevice};

pub struct GpuMonitor {
    state: Arc<RwLock<MonitorState>>,
    config: Arc<RwLock<MonitorConfig>>,
    metrics_history: Arc<RwLock<VecDeque<Vec<GpuMetrics>>>>,
    last_update: Arc<RwLock<SystemTime>>,
    #[cfg(feature = "nvidia")]
    nvml: Arc<RwLock<Option<Nvml>>>,
    gpu_type: Arc<RwLock<GpuType>>,
}

#[derive(Debug, Clone)]
enum GpuType {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

impl GpuMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MonitorState::Uninitialized)),
            config: Arc::new(RwLock::new(MonitorConfig::default())),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            last_update: Arc::new(RwLock::new(SystemTime::now())),
            #[cfg(feature = "nvidia")]
            nvml: Arc::new(RwLock::new(None)),
            gpu_type: Arc::new(RwLock::new(GpuType::Unknown)),
        }
    }

    fn detect_gpu_type(&self) -> GpuType {
        // First, try NVIDIA
        #[cfg(feature = "nvidia")]
        {
            if let Ok(nvml) = Nvml::init() {
                if nvml.device_count().unwrap_or(0) > 0 {
                    *self.nvml.write() = Some(nvml);
                    return GpuType::Nvidia;
                }
            }
        }

        // Try to detect via PCI devices
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/bus/pci/devices") {
                if contents.contains("10de") {
                    return GpuType::Nvidia;
                } else if contents.contains("1002") {
                    return GpuType::Amd;
                } else if contents.contains("8086") && contents.contains("class=0300") {
                    return GpuType::Intel;
                }
            }
        }

        // Try Windows WMI
        #[cfg(target_os = "windows")]
        {
            // Would use WMI or DirectX to detect GPU
            // This is a simplified version
            use std::process::Command;
            if let Ok(output) = Command::new("wmic")
                .args(&["path", "win32_VideoController", "get", "name"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("NVIDIA") {
                    return GpuType::Nvidia;
                } else if output_str.contains("AMD") || output_str.contains("Radeon") {
                    return GpuType::Amd;
                } else if output_str.contains("Intel") {
                    return GpuType::Intel;
                }
            }
        }

        GpuType::Unknown
    }

    fn collect_gpu_metrics(&self) -> Result<Vec<GpuMetrics>> {
        let gpu_type = self.gpu_type.read().clone();
        
        match gpu_type {
            GpuType::Nvidia => self.collect_nvidia_metrics(),
            GpuType::Amd => self.collect_amd_metrics(),
            GpuType::Intel => self.collect_intel_metrics(),
            GpuType::Unknown => Ok(Vec::new()),
        }
    }

    #[cfg(feature = "nvidia")]
    fn collect_nvidia_metrics(&self) -> Result<Vec<GpuMetrics>> {
        let nvml_guard = self.nvml.read();
        let nvml = nvml_guard.as_ref()
            .ok_or(MonitorError::NotInitialized)?;

        let device_count = nvml.device_count()
            .map_err(|e| MonitorError::CollectionError(e.to_string()))?;

        let mut metrics = Vec::new();

        for i in 0..device_count {
            let device = nvml.device_by_index(i)
                .map_err(|e| MonitorError::CollectionError(e.to_string()))?;

            let name = device.name()
                .unwrap_or_else(|_| format!("GPU {}", i));
            
            let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .unwrap_or(0) as f32;
            
            let utilization = device.utilization_rates()
                .map(|u| u.gpu)
                .unwrap_or(0) as f32;
            
            let memory_info = device.memory_info()
                .map_err(|e| MonitorError::CollectionError(e.to_string()))?;
            
            let power = device.power_usage()
                .unwrap_or(0) as f32 / 1000.0; // Convert mW to W
            
            let fan_speed = device.fan_speed(0)
                .ok()
                .map(|s| s as f32);
            
            let clocks = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
                .unwrap_or(0);
            
            let memory_clock = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
                .unwrap_or(0);
            
            let driver_version = nvml.sys_driver_version()
                .unwrap_or_else(|_| "Unknown".to_string());

            metrics.push(GpuMetrics {
                name,
                driver_version,
                temperature_celsius: temperature,
                usage_percent: utilization,
                memory_total_bytes: memory_info.total,
                memory_used_bytes: memory_info.used,
                memory_usage_percent: (memory_info.used as f32 / memory_info.total as f32) * 100.0,
                power_watts: power,
                fan_speed_percent: fan_speed,
                clock_mhz: clocks,
                memory_clock_mhz: memory_clock,
            });
        }

        Ok(metrics)
    }

    #[cfg(not(feature = "nvidia"))]
    fn collect_nvidia_metrics(&self) -> Result<Vec<GpuMetrics>> {
        // Fallback for when NVIDIA feature is not enabled
        self.collect_generic_metrics("NVIDIA")
    }

    fn collect_amd_metrics(&self) -> Result<Vec<GpuMetrics>> {
        // AMD GPU monitoring implementation
        // This would use AMD's ROCm SMI or ADL SDK
        #[cfg(target_os = "linux")]
        {
            // Try to use rocm-smi
            use std::process::Command;
            let output = Command::new("rocm-smi")
                .arg("--showtemp")
                .arg("--showuse")
                .arg("--showmeminfo")
                .output();
            
            if let Ok(output) = output {
                // Parse rocm-smi output
                return self.parse_rocm_smi_output(&String::from_utf8_lossy(&output.stdout));
            }
        }
        
        self.collect_generic_metrics("AMD")
    }

    fn collect_intel_metrics(&self) -> Result<Vec<GpuMetrics>> {
        // Intel GPU monitoring
        #[cfg(target_os = "linux")]
        {
            // Try to read from sysfs
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.to_string_lossy().contains("card") {
                            // Read Intel GPU metrics from sysfs
                            return self.read_intel_sysfs_metrics(&path);
                        }
                    }
                }
            }
        }
        
        self.collect_generic_metrics("Intel")
    }

    fn collect_generic_metrics(&self, vendor: &str) -> Result<Vec<GpuMetrics>> {
        // Generic fallback that provides basic information
        Ok(vec![GpuMetrics {
            name: format!("{} Graphics", vendor),
            driver_version: "Unknown".to_string(),
            temperature_celsius: 0.0,
            usage_percent: 0.0,
            memory_total_bytes: 0,
            memory_used_bytes: 0,
            memory_usage_percent: 0.0,
            power_watts: 0.0,
            fan_speed_percent: None,
            clock_mhz: 0,
            memory_clock_mhz: 0,
        }])
    }

    #[cfg(target_os = "linux")]
    fn parse_rocm_smi_output(&self, output: &str) -> Result<Vec<GpuMetrics>> {
        // Parse rocm-smi output format
        let mut metrics = Vec::new();
        // Implementation would parse the actual rocm-smi output
        // This is a placeholder
        Ok(metrics)
    }

    #[cfg(target_os = "linux")]
    fn read_intel_sysfs_metrics(&self, card_path: &std::path::Path) -> Result<Vec<GpuMetrics>> {
        // Read Intel GPU metrics from sysfs
        let mut metrics = GpuMetrics {
            name: "Intel Graphics".to_string(),
            driver_version: "i915".to_string(),
            temperature_celsius: 0.0,
            usage_percent: 0.0,
            memory_total_bytes: 0,
            memory_used_bytes: 0,
            memory_usage_percent: 0.0,
            power_watts: 0.0,
            fan_speed_percent: None,
            clock_mhz: 0,
            memory_clock_mhz: 0,
        };

        // Try to read frequency
        if let Ok(freq) = std::fs::read_to_string(card_path.join("gt_cur_freq_mhz")) {
            if let Ok(freq_val) = freq.trim().parse::<u32>() {
                metrics.clock_mhz = freq_val;
            }
        }

        Ok(vec![metrics])
    }

    fn update_history(&self, metrics: Vec<GpuMetrics>) {
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
impl Monitor for GpuMonitor {
    fn name(&self) -> &str {
        "GPU Monitor"
    }

    fn state(&self) -> MonitorState {
        *self.state.read()
    }

    async fn initialize(&mut self, config: MonitorConfig) -> Result<()> {
        *self.state.write() = MonitorState::Initializing;
        *self.config.write() = config;
        
        // Detect GPU type
        let gpu_type = self.detect_gpu_type();
        *self.gpu_type.write() = gpu_type;
        
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

        let gpu_metrics = self.collect_gpu_metrics()?;
        self.update_history(gpu_metrics.clone());
        *self.last_update.write() = SystemTime::now();

        let mut metrics = Vec::new();
        
        for (idx, gpu) in gpu_metrics.iter().enumerate() {
            let gpu_id = idx.to_string();
            
            metrics.push(Metric::new(
                MetricType::GpuUsage,
                MetricValue::Float(gpu.usage_percent as f64),
                "%",
            ).with_tag("gpu", &gpu_id).with_tag("name", &gpu.name));
            
            metrics.push(Metric::new(
                MetricType::GpuTemperature,
                MetricValue::Float(gpu.temperature_celsius as f64),
                "°C",
            ).with_tag("gpu", &gpu_id));
            
            metrics.push(Metric::new(
                MetricType::GpuMemoryUsage,
                MetricValue::Float(gpu.memory_usage_percent as f64),
                "%",
            ).with_tag("gpu", &gpu_id));
            
            metrics.push(Metric::new(
                MetricType::GpuPower,
                MetricValue::Float(gpu.power_watts as f64),
                "W",
            ).with_tag("gpu", &gpu_id));
            
            if let Some(fan_speed) = gpu.fan_speed_percent {
                metrics.push(Metric::new(
                    MetricType::GpuFanSpeed,
                    MetricValue::Float(fan_speed as f64),
                    "%",
                ).with_tag("gpu", &gpu_id));
            }
        }
        
        Ok(metrics)
    }

    async fn get_current_metrics(&self) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            let mut metrics = Vec::new();
            
            for (idx, gpu) in latest.iter().enumerate() {
                metrics.push(Metric::new(
                    MetricType::GpuUsage,
                    MetricValue::Float(gpu.usage_percent as f64),
                    "%",
                ).with_tag("gpu", idx.to_string()));
            }
            
            Ok(metrics)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_historical_metrics(&self, _duration_seconds: u64) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        let mut metrics = Vec::new();
        
        for gpu_list in history.iter() {
            for (idx, gpu) in gpu_list.iter().enumerate() {
                metrics.push(Metric::new(
                    MetricType::GpuUsage,
                    MetricValue::Float(gpu.usage_percent as f64),
                    "%",
                ).with_tag("gpu", idx.to_string()));
            }
        }
        
        Ok(metrics)
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, 
            "gpu_usage" | "gpu_temperature" | "gpu_memory" | 
            "gpu_power" | "gpu_clock" | "gpu_fan_speed"
        )
    }
}