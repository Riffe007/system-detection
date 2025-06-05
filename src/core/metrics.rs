use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    Float(f64),
    Integer(i64),
    Unsigned(u64),
    String(String),
    Boolean(bool),
    FloatArray(Vec<f64>),
    IntegerArray(Vec<i64>),
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricValue::Float(v) => write!(f, "{:.2}", v),
            MetricValue::Integer(v) => write!(f, "{}", v),
            MetricValue::Unsigned(v) => write!(f, "{}", v),
            MetricValue::String(v) => write!(f, "{}", v),
            MetricValue::Boolean(v) => write!(f, "{}", v),
            MetricValue::FloatArray(v) => write!(f, "{:?}", v),
            MetricValue::IntegerArray(v) => write!(f, "{:?}", v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    CpuUsage,
    CpuFrequency,
    CpuTemperature,
    MemoryUsage,
    MemoryAvailable,
    SwapUsage,
    GpuUsage,
    GpuMemoryUsage,
    GpuMemory,
    GpuTemperature,
    GpuPower,
    GpuFanSpeed,
    DiskUsage,
    DiskSpace,
    DiskIo,
    NetworkThroughput,
    NetworkBytes,
    NetworkPackets,
    NetworkErrors,
    NetworkStatus,
    NetworkSpeed,
    ProcessCount,
    ProcessCpu,
    ProcessCpuTotal,
    ProcessMemory,
    ProcessMemoryTotal,
    ProcessDiskIo,
    SystemUptime,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricType::CpuUsage => write!(f, "CPU Usage"),
            MetricType::CpuFrequency => write!(f, "CPU Frequency"),
            MetricType::CpuTemperature => write!(f, "CPU Temperature"),
            MetricType::MemoryUsage => write!(f, "Memory Usage"),
            MetricType::MemoryAvailable => write!(f, "Memory Available"),
            MetricType::SwapUsage => write!(f, "Swap Usage"),
            MetricType::GpuUsage => write!(f, "GPU Usage"),
            MetricType::GpuMemoryUsage => write!(f, "GPU Memory Usage"),
            MetricType::GpuMemory => write!(f, "GPU Memory"),
            MetricType::GpuTemperature => write!(f, "GPU Temperature"),
            MetricType::GpuPower => write!(f, "GPU Power"),
            MetricType::GpuFanSpeed => write!(f, "GPU Fan Speed"),
            MetricType::DiskUsage => write!(f, "Disk Usage"),
            MetricType::DiskSpace => write!(f, "Disk Space"),
            MetricType::DiskIo => write!(f, "Disk I/O"),
            MetricType::NetworkThroughput => write!(f, "Network Throughput"),
            MetricType::NetworkBytes => write!(f, "Network Bytes"),
            MetricType::NetworkPackets => write!(f, "Network Packets"),
            MetricType::NetworkErrors => write!(f, "Network Errors"),
            MetricType::NetworkStatus => write!(f, "Network Status"),
            MetricType::NetworkSpeed => write!(f, "Network Speed"),
            MetricType::ProcessCount => write!(f, "Process Count"),
            MetricType::ProcessCpu => write!(f, "Process CPU"),
            MetricType::ProcessCpuTotal => write!(f, "Total Process CPU"),
            MetricType::ProcessMemory => write!(f, "Process Memory"),
            MetricType::ProcessMemoryTotal => write!(f, "Total Process Memory"),
            MetricType::ProcessDiskIo => write!(f, "Process Disk I/O"),
            MetricType::SystemUptime => write!(f, "System Uptime"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub unit: String,
    pub timestamp: std::time::SystemTime,
    pub tags: std::collections::HashMap<String, String>,
}

impl Metric {
    pub fn new(metric_type: MetricType, value: MetricValue, unit: impl Into<String>) -> Self {
        Self {
            metric_type,
            value,
            unit: unit.into(),
            timestamp: std::time::SystemTime::now(),
            tags: std::collections::HashMap::new(),
        }
    }

    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}