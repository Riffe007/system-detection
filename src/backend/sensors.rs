use std::collections::HashMap;
use std::path::Path;
use crate::core::Result;

#[derive(Debug, Clone)]
pub struct SensorReading {
    pub name: String,
    pub value: f32,
    pub unit: String,
    pub sensor_type: SensorType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    Temperature,
    Fan,
    Voltage,
    Power,
    Current,
}

pub struct SensorsManager {
    #[cfg(target_os = "linux")]
    hwmon_sensors: HashMap<String, String>,
}

impl SensorsManager {
    pub fn new() -> Self {
        let mut manager = Self {
            #[cfg(target_os = "linux")]
            hwmon_sensors: HashMap::new(),
        };
        
        #[cfg(target_os = "linux")]
        manager.discover_hwmon_sensors();
        
        manager
    }
    
    #[cfg(target_os = "linux")]
    fn discover_hwmon_sensors(&mut self) {
        use std::fs;
        use std::path::Path;
        
        let hwmon_path = Path::new("/sys/class/hwmon");
        if let Ok(entries) = fs::read_dir(hwmon_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(name) = fs::read_to_string(path.join("name")) {
                    let name = name.trim().to_string();
                    self.hwmon_sensors.insert(name.clone(), path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    pub fn read_cpu_temperature(&self) -> Result<Option<f32>> {
        #[cfg(target_os = "linux")]
        {
            // Try different temperature sources
            let temp_sources = [
                "/sys/class/thermal/thermal_zone0/temp",
                "/sys/class/hwmon/hwmon0/temp1_input",
                "/sys/class/hwmon/hwmon1/temp1_input",
                "/sys/class/hwmon/hwmon2/temp1_input",
            ];
            
            for source in &temp_sources {
                if let Ok(temp_str) = std::fs::read_to_string(source) {
                    if let Ok(temp_millidegree) = temp_str.trim().parse::<f32>() {
                        return Ok(Some(temp_millidegree / 1000.0));
                    }
                }
            }
            
            // Try coretemp sensor
            for (name, path) in &self.hwmon_sensors {
                if name.contains("coretemp") || name.contains("k10temp") || name.contains("zenpower") {
                    let temp_path = format!("{}/temp1_input", path);
                    if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                        if let Ok(temp_millidegree) = temp_str.trim().parse::<f32>() {
                            return Ok(Some(temp_millidegree / 1000.0));
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows implementation would use WMI or OpenHardwareMonitor
            // This is a placeholder
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS implementation would use IOKit
            if let Ok(output) = std::process::Command::new("sysctl")
                .arg("-n")
                .arg("machdep.xcpm.cpu_thermal_level")
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(thermal_level) = output_str.trim().parse::<i32>() {
                    // Convert thermal level to approximate temperature
                    // This is a rough approximation
                    return Ok(Some(50.0 + (thermal_level as f32 * 5.0)));
                }
            }
        }
        
        Ok(None)
    }
    
    pub fn read_all_temperatures(&self) -> Vec<SensorReading> {
        let mut readings = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            // Read all temperature sensors
            for (name, path) in &self.hwmon_sensors {
                let base_path = Path::new(path);
                
                // Look for all temp*_input files
                for i in 1..10 {
                    let temp_input = format!("temp{}_input", i);
                    let temp_label = format!("temp{}_label", i);
                    
                    let input_path = base_path.join(&temp_input);
                    if input_path.exists() {
                        if let Ok(temp_str) = std::fs::read_to_string(&input_path) {
                            if let Ok(temp_millidegree) = temp_str.trim().parse::<f32>() {
                                let temp_celsius = temp_millidegree / 1000.0;
                                
                                // Try to get the label
                                let label = if let Ok(label_str) = std::fs::read_to_string(base_path.join(&temp_label)) {
                                    format!("{} - {}", name, label_str.trim())
                                } else {
                                    format!("{} - temp{}", name, i)
                                };
                                
                                readings.push(SensorReading {
                                    name: label,
                                    value: temp_celsius,
                                    unit: "°C".to_string(),
                                    sensor_type: SensorType::Temperature,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        readings
    }
    
    pub fn read_fan_speeds(&self) -> Vec<SensorReading> {
        let mut readings = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            for (name, path) in &self.hwmon_sensors {
                let base_path = Path::new(path);
                
                // Look for all fan*_input files
                for i in 1..10 {
                    let fan_input = format!("fan{}_input", i);
                    let fan_label = format!("fan{}_label", i);
                    
                    let input_path = base_path.join(&fan_input);
                    if input_path.exists() {
                        if let Ok(rpm_str) = std::fs::read_to_string(&input_path) {
                            if let Ok(rpm) = rpm_str.trim().parse::<f32>() {
                                // Skip if fan is not spinning
                                if rpm > 0.0 {
                                    let label = if let Ok(label_str) = std::fs::read_to_string(base_path.join(&fan_label)) {
                                        format!("{} - {}", name, label_str.trim())
                                    } else {
                                        format!("{} - fan{}", name, i)
                                    };
                                    
                                    readings.push(SensorReading {
                                        name: label,
                                        value: rpm,
                                        unit: "RPM".to_string(),
                                        sensor_type: SensorType::Fan,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        readings
    }
    
    pub fn read_voltages(&self) -> Vec<SensorReading> {
        let mut readings = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            for (name, path) in &self.hwmon_sensors {
                let base_path = Path::new(path);
                
                // Look for all in*_input files (voltages)
                for i in 0..20 {
                    let voltage_input = format!("in{}_input", i);
                    let voltage_label = format!("in{}_label", i);
                    
                    let input_path = base_path.join(&voltage_input);
                    if input_path.exists() {
                        if let Ok(mv_str) = std::fs::read_to_string(&input_path) {
                            if let Ok(millivolts) = mv_str.trim().parse::<f32>() {
                                let volts = millivolts / 1000.0;
                                
                                // Skip obviously invalid readings
                                if volts > 0.0 && volts < 20.0 {
                                    let label = if let Ok(label_str) = std::fs::read_to_string(base_path.join(&voltage_label)) {
                                        format!("{} - {}", name, label_str.trim())
                                    } else {
                                        format!("{} - in{}", name, i)
                                    };
                                    
                                    readings.push(SensorReading {
                                        name: label,
                                        value: volts,
                                        unit: "V".to_string(),
                                        sensor_type: SensorType::Voltage,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        readings
    }
    
    pub fn read_power_sensors(&self) -> Vec<SensorReading> {
        let mut readings = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            // Read Intel RAPL (Running Average Power Limit) sensors
            let rapl_path = Path::new("/sys/class/powercap/intel-rapl");
            if rapl_path.exists() {
                if let Ok(entries) = std::fs::read_dir(rapl_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.file_name().unwrap().to_str().unwrap().starts_with("intel-rapl:") {
                            if let Ok(name) = std::fs::read_to_string(path.join("name")) {
                                if let Ok(energy_str) = std::fs::read_to_string(path.join("energy_uj")) {
                                    if let Ok(energy_uj) = energy_str.trim().parse::<u64>() {
                                        // Convert microjoules to watts (approximate based on sampling rate)
                                        // This is a simplified calculation
                                        let watts = (energy_uj as f32) / 1_000_000.0;
                                        
                                        readings.push(SensorReading {
                                            name: format!("RAPL - {}", name.trim()),
                                            value: watts,
                                            unit: "W".to_string(),
                                            sensor_type: SensorType::Power,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Check for power*_input in hwmon
            for (name, path) in &self.hwmon_sensors {
                let base_path = Path::new(path);
                
                for i in 1..10 {
                    let power_input = format!("power{}_input", i);
                    let power_label = format!("power{}_label", i);
                    
                    let input_path = base_path.join(&power_input);
                    if input_path.exists() {
                        if let Ok(uw_str) = std::fs::read_to_string(&input_path) {
                            if let Ok(microwatts) = uw_str.trim().parse::<f32>() {
                                let watts = microwatts / 1_000_000.0;
                                
                                let label = if let Ok(label_str) = std::fs::read_to_string(base_path.join(&power_label)) {
                                    format!("{} - {}", name, label_str.trim())
                                } else {
                                    format!("{} - power{}", name, i)
                                };
                                
                                readings.push(SensorReading {
                                    name: label,
                                    value: watts,
                                    unit: "W".to_string(),
                                    sensor_type: SensorType::Power,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        readings
    }
}

// Integration with CPU monitor
impl super::cpu_monitor::CpuMonitor {
    pub fn update_temperature(&self) -> Option<f32> {
        let sensors = SensorsManager::new();
        sensors.read_cpu_temperature().ok().flatten()
    }
}