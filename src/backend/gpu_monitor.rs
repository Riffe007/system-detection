use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;

use crate::core::{
    GpuMetrics, Metric, MetricType, MetricValue, Monitor, MonitorConfig, MonitorError,
    MonitorState, Result,
};

#[cfg(feature = "nvidia")]
use nvml_wrapper::Nvml;

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
        #[cfg(target_os = "linux")]
        {
            // First try ROCm SMI for newer AMD GPUs
            use std::process::Command;
            let output = Command::new("rocm-smi")
                .arg("--json")
                .output();
            
            if let Ok(output) = output {
                if output.status.success() {
                    return self.parse_rocm_smi_json(&String::from_utf8_lossy(&output.stdout));
                }
            }
            
            // Fallback to reading from sysfs for AMDGPU driver
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                let mut metrics = Vec::new();
                
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    
                    // Look for AMD GPU cards
                    if name.starts_with("card") && !name.contains("card0-") {
                        if let Ok(device_path) = std::fs::read_link(path.join("device/driver")) {
                            if device_path.to_string_lossy().contains("amdgpu") {
                                if let Ok(gpu_metrics) = self.read_amd_sysfs_metrics(&path) {
                                    metrics.push(gpu_metrics);
                                }
                            }
                        }
                    }
                }
                
                if !metrics.is_empty() {
                    return Ok(metrics);
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Try AMD ADL (AMD Display Library)
            if let Ok(metrics) = self.collect_amd_adl_metrics() {
                return Ok(metrics);
            }
        }
        
        self.collect_generic_metrics("AMD")
    }

    fn collect_intel_metrics(&self) -> Result<Vec<GpuMetrics>> {
        // Intel GPU monitoring
        #[cfg(target_os = "linux")]
        {
            // Try to read from sysfs for Intel i915 driver
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                let mut metrics = Vec::new();
                
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    
                    // Look for Intel GPU cards (renderD devices are also Intel GPUs)
                    if name.starts_with("card") && !name.contains("card0-") {
                        if let Ok(device_path) = std::fs::read_link(path.join("device/driver")) {
                            if device_path.to_string_lossy().contains("i915") {
                                if let Ok(gpu_metrics) = self.read_intel_sysfs_metrics(&path) {
                                    metrics.extend(gpu_metrics);
                                }
                            }
                        }
                    }
                }
                
                if !metrics.is_empty() {
                    return Ok(metrics);
                }
            }
            
            // Try intel_gpu_top tool as fallback
            if let Ok(output) = std::process::Command::new("intel_gpu_top")
                .arg("-J")
                .arg("-o")
                .arg("-")
                .output()
            {
                if output.status.success() {
                    return self.parse_intel_gpu_top(&String::from_utf8_lossy(&output.stdout));
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Try Windows WMI for Intel graphics
            if let Ok(metrics) = self.collect_intel_wmi_metrics() {
                return Ok(metrics);
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
    fn parse_rocm_smi_json(&self, output: &str) -> Result<Vec<GpuMetrics>> {
        use serde_json::Value;
        
        let json: Value = serde_json::from_str(output)
            .map_err(|e| MonitorError::CollectionError(format!("Failed to parse ROCm JSON: {}", e)))?;
        
        let mut metrics = Vec::new();
        
        if let Some(devices) = json.as_object() {
            for (device_id, device_data) in devices {
                if let Some(data) = device_data.as_object() {
                    let name = data.get("Card series")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&format!("AMD GPU {}", device_id))
                        .to_string();
                    
                    let temperature = data.get("Temperature (Sensor edge) (C)")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;
                    
                    let usage = data.get("GPU use (%)")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;
                    
                    let memory_used = data.get("GPU memory use")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.split('/').next())
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(0) * 1024 * 1024; // Convert MB to bytes
                    
                    let memory_total = data.get("GPU memory use")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.split('/').nth(1))
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(0) * 1024 * 1024; // Convert MB to bytes
                    
                    let power = data.get("Average Graphics Package Power (W)")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;
                    
                    let clock_mhz = data.get("SCLK clock speed:")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.trim_end_matches("Mhz").parse::<u32>().ok())
                        .unwrap_or(0);
                    
                    let memory_clock_mhz = data.get("MCLK clock speed:")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.trim_end_matches("Mhz").parse::<u32>().ok())
                        .unwrap_or(0);
                    
                    let fan_speed = data.get("Fan speed (%)")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    
                    metrics.push(GpuMetrics {
                        name,
                        driver_version: "amdgpu".to_string(),
                        temperature_celsius: temperature,
                        usage_percent: usage,
                        memory_total_bytes: memory_total,
                        memory_used_bytes: memory_used,
                        memory_usage_percent: if memory_total > 0 {
                            (memory_used as f32 / memory_total as f32) * 100.0
                        } else { 0.0 },
                        power_watts: power,
                        fan_speed_percent: fan_speed,
                        clock_mhz,
                        memory_clock_mhz,
                    });
                }
            }
        }
        
        Ok(metrics)
    }
    
    #[cfg(target_os = "linux")]
    fn read_amd_sysfs_metrics(&self, card_path: &std::path::Path) -> Result<GpuMetrics> {
        let device_path = card_path.join("device");
        
        // Read GPU name
        let name = std::fs::read_to_string(device_path.join("product_name"))
            .or_else(|_| std::fs::read_to_string(device_path.join("name")))
            .unwrap_or_else(|_| "AMD GPU".to_string())
            .trim()
            .to_string();
        
        // Read temperature from hwmon
        let mut temperature = 0.0f32;
        if let Ok(hwmon_entries) = std::fs::read_dir(device_path.join("hwmon")) {
            for entry in hwmon_entries.flatten() {
                let temp_path = entry.path().join("temp1_input");
                if let Ok(temp_str) = std::fs::read_to_string(temp_path) {
                    if let Ok(temp_millidegree) = temp_str.trim().parse::<f32>() {
                        temperature = temp_millidegree / 1000.0;
                        break;
                    }
                }
            }
        }
        
        // Read GPU usage
        let usage = std::fs::read_to_string(device_path.join("gpu_busy_percent"))
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.0);
        
        // Read memory info
        let memory_total = std::fs::read_to_string(device_path.join("mem_info_vram_total"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        
        let memory_used = std::fs::read_to_string(device_path.join("mem_info_vram_used"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        
        // Read power
        let power = std::fs::read_to_string(device_path.join("hwmon/hwmon0/power1_average"))
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .map(|p| p / 1_000_000.0) // Convert microwatts to watts
            .unwrap_or(0.0);
        
        // Read clocks
        let clock_mhz = self.read_amd_clock(&device_path, "pp_dpm_sclk")
            .unwrap_or(0);
        
        let memory_clock_mhz = self.read_amd_clock(&device_path, "pp_dpm_mclk")
            .unwrap_or(0);
        
        // Read fan speed
        let fan_speed = std::fs::read_to_string(device_path.join("hwmon/hwmon0/pwm1"))
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .map(|pwm| (pwm / 255.0) * 100.0); // Convert PWM to percentage
        
        Ok(GpuMetrics {
            name,
            driver_version: "amdgpu".to_string(),
            temperature_celsius: temperature,
            usage_percent: usage,
            memory_total_bytes: memory_total,
            memory_used_bytes: memory_used,
            memory_usage_percent: if memory_total > 0 {
                (memory_used as f32 / memory_total as f32) * 100.0
            } else { 0.0 },
            power_watts: power,
            fan_speed_percent: fan_speed,
            clock_mhz: clock_mhz,
            memory_clock_mhz: memory_clock_mhz,
        })
    }
    
    #[cfg(target_os = "linux")]
    fn read_amd_clock(&self, device_path: &std::path::Path, clock_file: &str) -> Option<u32> {
        std::fs::read_to_string(device_path.join(clock_file))
            .ok()
            .and_then(|content| {
                // Parse the active clock from the DPM states
                content.lines()
                    .find(|line| line.contains('*'))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|s| s.trim_end_matches("Mhz").parse::<u32>().ok())
                    })
            })
    }
    
    #[cfg(target_os = "windows")]
    fn collect_amd_adl_metrics(&self) -> Result<Vec<GpuMetrics>> {
        // This would use AMD's ADL SDK for Windows
        // For now, we'll use WMI as a fallback
        use std::process::Command;
        
        let output = Command::new("wmic")
            .args(&["path", "Win32_VideoController", "where", "Name like '%AMD%' or Name like '%Radeon%'", "get", "Name,DriverVersion,AdapterRAM", "/format:csv"])
            .output()
            .map_err(|e| MonitorError::CollectionError(format!("Failed to run WMI: {}", e)))?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut metrics = Vec::new();
        
        for line in output_str.lines().skip(2) { // Skip headers
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let name = parts[2].to_string();
                let driver_version = parts[1].to_string();
                let memory_total = parts[3].parse::<u64>().unwrap_or(0);
                
                metrics.push(GpuMetrics {
                    name,
                    driver_version,
                    temperature_celsius: 0.0,
                    usage_percent: 0.0,
                    memory_total_bytes: memory_total,
                    memory_used_bytes: 0,
                    memory_usage_percent: 0.0,
                    power_watts: 0.0,
                    fan_speed_percent: None,
                    clock_mhz: 0,
                    memory_clock_mhz: 0,
                });
            }
        }
        
        Ok(metrics)
    }

    #[cfg(target_os = "linux")]
    fn read_intel_sysfs_metrics(&self, card_path: &std::path::Path) -> Result<Vec<GpuMetrics>> {
        let device_path = card_path.join("device");
        
        // Read Intel GPU name
        let name = std::fs::read_to_string("/sys/devices/virtual/dmi/id/board_name")
            .map(|s| format!("Intel Graphics ({})", s.trim()))
            .unwrap_or_else(|_| "Intel Graphics".to_string());
        
        // Read current frequency
        let clock_mhz = std::fs::read_to_string(card_path.join("gt_cur_freq_mhz"))
            .or_else(|_| std::fs::read_to_string(device_path.join("gt_cur_freq_mhz")))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        
        // Read max frequency for reference
        let max_freq = std::fs::read_to_string(card_path.join("gt_max_freq_mhz"))
            .or_else(|_| std::fs::read_to_string(device_path.join("gt_max_freq_mhz")))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(clock_mhz);
        
        // Calculate usage based on frequency (approximation)
        let usage_percent = if max_freq > 0 {
            (clock_mhz as f32 / max_freq as f32) * 100.0
        } else {
            0.0
        };
        
        // Read power consumption
        let power_watts = self.read_intel_power(&device_path).unwrap_or(0.0);
        
        // Read temperature
        let temperature_celsius = self.read_intel_temperature(&device_path).unwrap_or(0.0);
        
        // Try to get memory info from debugfs (requires root)
        let (memory_total, memory_used) = self.read_intel_memory_info().unwrap_or((0, 0));
        
        Ok(vec![GpuMetrics {
            name,
            driver_version: "i915".to_string(),
            temperature_celsius,
            usage_percent,
            memory_total_bytes: memory_total,
            memory_used_bytes: memory_used,
            memory_usage_percent: if memory_total > 0 {
                (memory_used as f32 / memory_total as f32) * 100.0
            } else { 0.0 },
            power_watts,
            fan_speed_percent: None, // Intel integrated GPUs typically don't have fans
            clock_mhz,
            memory_clock_mhz: 0, // Not easily accessible for Intel GPUs
        }])
    }
    
    #[cfg(target_os = "linux")]
    fn read_intel_power(&self, device_path: &std::path::Path) -> Option<f32> {
        // Try multiple power reading locations
        let power_paths = vec![
            device_path.join("power/energy_uj"),
            device_path.join("power1_average"),
            device_path.join("hwmon/hwmon0/power1_average"),
        ];
        
        for path in power_paths {
            if let Ok(power_str) = std::fs::read_to_string(&path) {
                if let Ok(power_uj) = power_str.trim().parse::<f64>() {
                    // Convert microjoules to watts (need to track time delta for accurate calculation)
                    // For now, return a rough estimate
                    return Some((power_uj / 1_000_000.0) as f32);
                }
            }
        }
        
        None
    }
    
    #[cfg(target_os = "linux")]
    fn read_intel_temperature(&self, device_path: &std::path::Path) -> Option<f32> {
        // Try to read temperature from thermal zones
        if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let thermal_path = entry.path();
                if let Ok(thermal_type) = std::fs::read_to_string(thermal_path.join("type")) {
                    if thermal_type.trim().contains("gpu") || thermal_type.trim().contains("gfx") {
                        if let Ok(temp_str) = std::fs::read_to_string(thermal_path.join("temp")) {
                            if let Ok(temp_millidegree) = temp_str.trim().parse::<f32>() {
                                return Some(temp_millidegree / 1000.0);
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
    
    #[cfg(target_os = "linux")]
    fn read_intel_memory_info(&self) -> Option<(u64, u64)> {
        // Try to parse memory info from i915_gem_objects in debugfs
        if let Ok(gem_objects) = std::fs::read_to_string("/sys/kernel/debug/dri/0/i915_gem_objects") {
            let mut total_bytes = 0u64;
            let mut active_bytes = 0u64;
            
            for line in gem_objects.lines() {
                if line.contains("total") && line.contains("objects") {
                    // Parse lines like: "831 objects, 123456789 bytes"
                    if let Some(bytes_part) = line.split(',').nth(1) {
                        if let Some(bytes_str) = bytes_part.split_whitespace().next() {
                            total_bytes = bytes_str.parse().unwrap_or(0);
                        }
                    }
                } else if line.contains("active") {
                    if let Some(bytes_part) = line.split(',').nth(1) {
                        if let Some(bytes_str) = bytes_part.split_whitespace().next() {
                            active_bytes = bytes_str.parse().unwrap_or(0);
                        }
                    }
                }
            }
            
            if total_bytes > 0 {
                return Some((total_bytes, active_bytes));
            }
        }
        
        None
    }
    
    #[cfg(target_os = "linux")]
    fn parse_intel_gpu_top(&self, output: &str) -> Result<Vec<GpuMetrics>> {
        use serde_json::Value;
        
        // intel_gpu_top outputs JSON with engine utilization
        let json: Value = serde_json::from_str(output)
            .map_err(|e| MonitorError::CollectionError(format!("Failed to parse intel_gpu_top JSON: {}", e)))?;
        
        let mut usage_percent = 0.0f32;
        let mut render_usage = 0.0f32;
        
        if let Some(engines) = json["engines"].as_object() {
            // Calculate average usage across all engines
            let mut total_usage = 0.0;
            let mut engine_count = 0;
            
            for (engine_name, engine_data) in engines {
                if let Some(busy) = engine_data["busy"].as_f64() {
                    total_usage += busy;
                    engine_count += 1;
                    
                    if engine_name.contains("Render") || engine_name.contains("3D") {
                        render_usage = busy as f32;
                    }
                }
            }
            
            if engine_count > 0 {
                usage_percent = (total_usage / engine_count as f64) as f32;
            }
        }
        
        let frequency = json["frequency"]["actual"].as_u64().unwrap_or(0) as u32;
        
        Ok(vec![GpuMetrics {
            name: "Intel Graphics".to_string(),
            driver_version: "i915".to_string(),
            temperature_celsius: 0.0,
            usage_percent: usage_percent.max(render_usage), // Use the higher of average or render usage
            memory_total_bytes: 0,
            memory_used_bytes: 0,
            memory_usage_percent: 0.0,
            power_watts: json["power"]["value"].as_f64().unwrap_or(0.0) as f32,
            fan_speed_percent: None,
            clock_mhz: frequency,
            memory_clock_mhz: 0,
        }])
    }
    
    #[cfg(target_os = "windows")]
    fn collect_intel_wmi_metrics(&self) -> Result<Vec<GpuMetrics>> {
        use std::process::Command;
        
        let output = Command::new("wmic")
            .args(&["path", "Win32_VideoController", "where", "Name like '%Intel%'", "get", "Name,DriverVersion,AdapterRAM,CurrentRefreshRate", "/format:csv"])
            .output()
            .map_err(|e| MonitorError::CollectionError(format!("Failed to run WMI: {}", e)))?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut metrics = Vec::new();
        
        for line in output_str.lines().skip(2) { // Skip headers
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let name = parts[2].to_string();
                let driver_version = parts[3].to_string();
                let memory_total = parts[1].parse::<u64>().unwrap_or(0);
                
                metrics.push(GpuMetrics {
                    name,
                    driver_version,
                    temperature_celsius: 0.0,
                    usage_percent: 0.0,
                    memory_total_bytes: memory_total,
                    memory_used_bytes: 0,
                    memory_usage_percent: 0.0,
                    power_watts: 0.0,
                    fan_speed_percent: None,
                    clock_mhz: 0,
                    memory_clock_mhz: 0,
                });
            }
        }
        
        Ok(metrics)
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