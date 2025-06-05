use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use sysinfo::{System, SystemExt, NetworkExt, NetworksExt};

use crate::core::{
    NetworkMetrics, Metric, MetricType, MetricValue, Monitor, MonitorConfig, MonitorError,
    MonitorState, Result,
};

pub struct NetworkMonitor {
    state: Arc<RwLock<MonitorState>>,
    config: Arc<RwLock<MonitorConfig>>,
    system: Arc<RwLock<System>>,
    metrics_history: Arc<RwLock<VecDeque<Vec<NetworkMetrics>>>>,
    last_update: Arc<RwLock<SystemTime>>,
    previous_stats: Arc<RwLock<HashMap<String, NetworkStats>>>,
}

#[derive(Clone, Debug)]
struct NetworkStats {
    bytes_sent: u64,
    bytes_received: u64,
    packets_sent: u64,
    packets_received: u64,
    errors_sent: u64,
    errors_received: u64,
    timestamp: SystemTime,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MonitorState::Uninitialized)),
            config: Arc::new(RwLock::new(MonitorConfig::default())),
            system: Arc::new(RwLock::new(System::new_all())),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            last_update: Arc::new(RwLock::new(SystemTime::now())),
            previous_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn collect_network_metrics(&self) -> Result<Vec<NetworkMetrics>> {
        let mut system = self.system.write();
        system.refresh_networks();
        system.refresh_networks_list();

        let mut metrics = Vec::new();
        let mut current_stats = HashMap::new();
        let now = SystemTime::now();
        let previous_stats = self.previous_stats.read();

        for (interface_name, network) in system.networks() {
            let bytes_sent = network.total_transmitted();
            let bytes_received = network.total_received();
            let packets_sent = network.total_packets_transmitted();
            let packets_received = network.total_packets_received();
            let errors_sent = network.total_errors_on_transmitted();
            let errors_received = network.total_errors_on_received();

            // Store current stats for rate calculation
            let stats = NetworkStats {
                bytes_sent,
                bytes_received,
                packets_sent,
                packets_received,
                errors_sent,
                errors_received,
                timestamp: now,
            };

            // Calculate rates if we have previous stats
            let (bytes_sent_rate, bytes_received_rate) = if let Some(prev_stats) = previous_stats.get(interface_name) {
                if let Ok(duration) = now.duration_since(prev_stats.timestamp) {
                    let secs = duration.as_secs_f64();
                    if secs > 0.0 {
                        let sent_rate = ((bytes_sent.saturating_sub(prev_stats.bytes_sent)) as f64 / secs) as u64;
                        let recv_rate = ((bytes_received.saturating_sub(prev_stats.bytes_received)) as f64 / secs) as u64;
                        (sent_rate, recv_rate)
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            current_stats.insert(interface_name.clone(), stats);

            // Get additional interface information
            let (is_up, mac_address, ip_addresses, speed_mbps) = self.get_interface_details(interface_name);

            metrics.push(NetworkMetrics {
                interface_name: interface_name.clone(),
                is_up,
                mac_address,
                ip_addresses,
                bytes_sent,
                bytes_received,
                packets_sent,
                packets_received,
                errors_sent,
                errors_received,
                speed_mbps,
                bytes_sent_rate,
                bytes_received_rate,
            });
        }

        // Update previous stats for next calculation
        *self.previous_stats.write() = current_stats;

        Ok(metrics)
    }

    fn get_interface_details(&self, interface_name: &str) -> (bool, String, Vec<String>, Option<u64>) {
        let mut is_up = true;
        let mut mac_address = String::from("00:00:00:00:00:00");
        let mut ip_addresses = Vec::new();
        let mut speed_mbps = None;

        #[cfg(target_os = "linux")]
        {
            use std::fs;
            use std::path::Path;

            // Check if interface is up
            let state_path = format!("/sys/class/net/{}/operstate", interface_name);
            if let Ok(state) = fs::read_to_string(&state_path) {
                is_up = state.trim() == "up";
            }

            // Get MAC address
            let mac_path = format!("/sys/class/net/{}/address", interface_name);
            if let Ok(mac) = fs::read_to_string(&mac_path) {
                mac_address = mac.trim().to_string();
            }

            // Get speed
            let speed_path = format!("/sys/class/net/{}/speed", interface_name);
            if let Ok(speed_str) = fs::read_to_string(&speed_path) {
                if let Ok(speed) = speed_str.trim().parse::<u64>() {
                    if speed > 0 && speed < 100000 { // Sanity check
                        speed_mbps = Some(speed);
                    }
                }
            }

            // Get IP addresses using ip command
            if let Ok(output) = std::process::Command::new("ip")
                .args(&["addr", "show", interface_name])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("inet ") {
                        if let Some(ip_part) = line.split_whitespace().nth(1) {
                            if let Some(ip) = ip_part.split('/').next() {
                                ip_addresses.push(ip.to_string());
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows implementation would use WMI or iphlpapi
            // This is a placeholder
            use std::process::Command;
            if let Ok(output) = Command::new("wmic")
                .args(&["nic", "where", &format!("NetConnectionID='{}'", interface_name), "get", "MACAddress,Speed,NetConnectionStatus"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Parse WMI output
                // This is simplified
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS implementation would use ifconfig or system configuration framework
            use std::process::Command;
            if let Ok(output) = Command::new("ifconfig")
                .arg(interface_name)
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Parse ifconfig output
                for line in output_str.lines() {
                    if line.contains("ether") {
                        if let Some(mac) = line.split_whitespace().nth(1) {
                            mac_address = mac.to_string();
                        }
                    } else if line.contains("inet ") {
                        if let Some(ip) = line.split_whitespace().nth(1) {
                            ip_addresses.push(ip.to_string());
                        }
                    }
                }
            }
        }

        (is_up, mac_address, ip_addresses, speed_mbps)
    }

    fn update_history(&self, metrics: Vec<NetworkMetrics>) {
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
impl Monitor for NetworkMonitor {
    fn name(&self) -> &str {
        "Network Monitor"
    }

    fn state(&self) -> MonitorState {
        *self.state.read()
    }

    async fn initialize(&mut self, config: MonitorConfig) -> Result<()> {
        *self.state.write() = MonitorState::Initializing;
        *self.config.write() = config;
        
        // Initialize system info
        let mut system = self.system.write();
        system.refresh_networks();
        system.refresh_networks_list();
        
        // Collect initial stats
        let _ = self.collect_network_metrics()?;
        
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

        let network_metrics = self.collect_network_metrics()?;
        self.update_history(network_metrics.clone());
        *self.last_update.write() = SystemTime::now();

        let mut metrics = Vec::new();
        
        for network in network_metrics.iter() {
            // Skip loopback and inactive interfaces unless configured otherwise
            if network.interface_name.contains("lo") && !self.config.read().include_loopback {
                continue;
            }

            // Network throughput rates
            metrics.push(Metric::new(
                MetricType::NetworkThroughput,
                MetricValue::Unsigned(network.bytes_sent_rate),
                "bytes/s",
            ).with_tag("interface", &network.interface_name)
             .with_tag("direction", "sent"));
            
            metrics.push(Metric::new(
                MetricType::NetworkThroughput,
                MetricValue::Unsigned(network.bytes_received_rate),
                "bytes/s",
            ).with_tag("interface", &network.interface_name)
             .with_tag("direction", "received"));
            
            // Total bytes transferred
            metrics.push(Metric::new(
                MetricType::NetworkBytes,
                MetricValue::Unsigned(network.bytes_sent),
                "bytes",
            ).with_tag("interface", &network.interface_name)
             .with_tag("direction", "sent"));
            
            metrics.push(Metric::new(
                MetricType::NetworkBytes,
                MetricValue::Unsigned(network.bytes_received),
                "bytes",
            ).with_tag("interface", &network.interface_name)
             .with_tag("direction", "received"));
            
            // Packet counts
            metrics.push(Metric::new(
                MetricType::NetworkPackets,
                MetricValue::Unsigned(network.packets_sent),
                "packets",
            ).with_tag("interface", &network.interface_name)
             .with_tag("direction", "sent"));
            
            metrics.push(Metric::new(
                MetricType::NetworkPackets,
                MetricValue::Unsigned(network.packets_received),
                "packets",
            ).with_tag("interface", &network.interface_name)
             .with_tag("direction", "received"));
            
            // Error counts
            if network.errors_sent > 0 || network.errors_received > 0 {
                metrics.push(Metric::new(
                    MetricType::NetworkErrors,
                    MetricValue::Unsigned(network.errors_sent),
                    "errors",
                ).with_tag("interface", &network.interface_name)
                 .with_tag("direction", "sent"));
                
                metrics.push(Metric::new(
                    MetricType::NetworkErrors,
                    MetricValue::Unsigned(network.errors_received),
                    "errors",
                ).with_tag("interface", &network.interface_name)
                 .with_tag("direction", "received"));
            }
            
            // Interface status
            metrics.push(Metric::new(
                MetricType::NetworkStatus,
                MetricValue::Boolean(network.is_up),
                "status",
            ).with_tag("interface", &network.interface_name));
            
            // Link speed if available
            if let Some(speed) = network.speed_mbps {
                metrics.push(Metric::new(
                    MetricType::NetworkSpeed,
                    MetricValue::Unsigned(speed),
                    "Mbps",
                ).with_tag("interface", &network.interface_name));
            }
        }
        
        Ok(metrics)
    }

    async fn get_current_metrics(&self) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        
        if let Some(latest) = history.back() {
            let mut metrics = Vec::new();
            
            for network in latest.iter() {
                if network.interface_name.contains("lo") && !self.config.read().include_loopback {
                    continue;
                }
                
                metrics.push(Metric::new(
                    MetricType::NetworkThroughput,
                    MetricValue::Unsigned(network.bytes_sent_rate + network.bytes_received_rate),
                    "bytes/s",
                ).with_tag("interface", &network.interface_name));
            }
            
            Ok(metrics)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_historical_metrics(&self, _duration_seconds: u64) -> Result<Vec<Metric>> {
        let history = self.metrics_history.read();
        let mut metrics = Vec::new();
        
        for network_list in history.iter() {
            for network in network_list.iter() {
                if network.interface_name.contains("lo") && !self.config.read().include_loopback {
                    continue;
                }
                
                metrics.push(Metric::new(
                    MetricType::NetworkThroughput,
                    MetricValue::Unsigned(network.bytes_sent_rate + network.bytes_received_rate),
                    "bytes/s",
                ).with_tag("interface", &network.interface_name));
            }
        }
        
        Ok(metrics)
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, 
            "network_throughput" | "network_bytes" | "network_packets" | 
            "network_errors" | "network_status" | "network_speed"
        )
    }
}

// Extension for NetworkMetrics to include rate calculations
impl NetworkMetrics {
    pub fn new(interface_name: String) -> Self {
        Self {
            interface_name,
            is_up: false,
            mac_address: String::from("00:00:00:00:00:00"),
            ip_addresses: Vec::new(),
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            errors_sent: 0,
            errors_received: 0,
            speed_mbps: None,
            bytes_sent_rate: 0,
            bytes_received_rate: 0,
        }
    }
}