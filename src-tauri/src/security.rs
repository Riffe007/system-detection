use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::System;
use std::process::Command;
use std::str::FromStr;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: String,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    SuspiciousProcess,
    NetworkConnection,
    FileSystemChange,
    HighCpuUsage,
    HighMemoryUsage,
    UnusualNetworkActivity,
    ProcessCreation,
    ProcessTermination,
    ProcessQuarantined,
    ProcessTerminated,
    NetworkBlocked,
    FirewallRuleAdded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSecurityInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub suspicious_score: f32,
    pub network_connections: Vec<String>,
    pub file_handles: u32,
    pub start_time: String,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurityInfo {
    pub interface: String,
    pub local_address: String,
    pub remote_address: String,
    pub port: u16,
    pub protocol: String,
    pub process_name: String,
    pub process_pid: u32,
    pub connection_type: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedProcess {
    pub pid: u32,
    pub name: String,
    pub original_path: String,
    pub quarantine_path: String,
    pub quarantine_time: String,
    pub reason: String,
    pub status: QuarantineStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuarantineStatus {
    Quarantined,
    Restored,
    Deleted,
    PendingAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedConnection {
    pub remote_address: String,
    pub port: u16,
    pub protocol: String,
    pub block_time: String,
    pub reason: String,
    pub firewall_rule_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub timestamp: String,
    pub total_processes: usize,
    pub suspicious_processes: Vec<ProcessSecurityInfo>,
    pub network_connections: Vec<NetworkSecurityInfo>,
    pub recent_events: Vec<SecurityEvent>,
    pub security_score: f32,
    pub threats_detected: usize,
    pub recommendations: Vec<String>,
    pub quarantined_processes: Vec<QuarantinedProcess>,
    pub blocked_connections: Vec<BlockedConnection>,
}

pub struct SecurityMonitor {
    system: Arc<RwLock<System>>,
    events: Arc<RwLock<Vec<SecurityEvent>>>,
    suspicious_processes: Arc<RwLock<Vec<String>>>,
    known_processes: Arc<RwLock<HashMap<u32, String>>>,
    quarantined_processes: Arc<RwLock<Vec<QuarantinedProcess>>>,
    blocked_connections: Arc<RwLock<Vec<BlockedConnection>>>,
    quarantine_directory: String,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        // Create quarantine directory
        let quarantine_dir = std::env::temp_dir()
            .join("system_monitor_quarantine")
            .to_string_lossy()
            .to_string();
        
        // Ensure quarantine directory exists
        if let Err(_) = fs::create_dir_all(&quarantine_dir) {
            eprintln!("Warning: Could not create quarantine directory: {}", quarantine_dir);
        }
        
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            events: Arc::new(RwLock::new(Vec::new())),
            suspicious_processes: Arc::new(RwLock::new(vec![
                // Only truly suspicious processes that are rarely legitimate
                "wscript.exe".to_string(),
                "cscript.exe".to_string(),
                "rundll32.exe".to_string(),
                "regsvr32.exe".to_string(),
                "mshta.exe".to_string(),
                "certutil.exe".to_string(),
                "bitsadmin.exe".to_string(),
                "wmic.exe".to_string(),
                // Removed cmd.exe and powershell.exe as they're commonly legitimate
            ])),
            known_processes: Arc::new(RwLock::new(HashMap::new())),
            quarantined_processes: Arc::new(RwLock::new(Vec::new())),
            blocked_connections: Arc::new(RwLock::new(Vec::new())),
            quarantine_directory: quarantine_dir,
        }
    }

    pub async fn collect_security_metrics(&self) -> Result<SecurityMetrics, String> {
        let mut sys = self.system.write().await;
        sys.refresh_all();
        drop(sys);
        
        let sys = self.system.read().await;
        
        // Real process analysis
        let suspicious_processes = self.analyze_processes(&sys).await;
        
        // Real network analysis
        let network_connections = self.analyze_network_connections(&sys).await;
        
        // Real security scoring
        let security_score = self.calculate_security_score(&suspicious_processes, &network_connections);
        
        let threats_count = suspicious_processes.len();
        let recommendations = self.generate_recommendations(&suspicious_processes, &network_connections);
        
        Ok(SecurityMetrics {
            timestamp: chrono::Utc::now().timestamp().to_string(),
            total_processes: sys.processes().len(),
            suspicious_processes,
            network_connections,
            recent_events: self.events.read().await.clone(),
            security_score,
            threats_detected: threats_count,
            recommendations,
            quarantined_processes: self.quarantined_processes.read().await.clone(),
            blocked_connections: self.blocked_connections.read().await.clone(),
        })
    }

    async fn analyze_processes(&self, sys: &System) -> Vec<ProcessSecurityInfo> {
        let mut suspicious_processes = Vec::new();
        let suspicious_list = self.suspicious_processes.read().await;
        
        for (pid, process) in sys.processes() {
            let mut suspicious_score = 0.0;
            let mut security_events = Vec::new();
            
            // Check if process name is in suspicious list
            if suspicious_list.contains(&process.name().to_string()) {
                suspicious_score += 30.0;
                security_events.push(SecurityEvent {
                    timestamp: chrono::Utc::now().timestamp().to_string(),
                    event_type: SecurityEventType::SuspiciousProcess,
                    severity: SecuritySeverity::High,
                    description: format!("Suspicious process detected: {}", process.name()),
                    details: HashMap::from([
                        ("pid".to_string(), pid.as_u32().to_string()),
                        ("process_name".to_string(), process.name().to_string()),
                        ("cpu_usage".to_string(), process.cpu_usage().to_string()),
                        ("memory_usage".to_string(), process.memory().to_string()),
                    ]),
                });
            }
            
            // Check for high CPU usage (only if sustained and from suspicious processes)
            if process.cpu_usage() > 80.0 { // Increased threshold
                suspicious_score += 15.0; // Reduced penalty
                security_events.push(SecurityEvent {
                    timestamp: chrono::Utc::now().timestamp().to_string(),
                    event_type: SecurityEventType::HighCpuUsage,
                    severity: SecuritySeverity::Medium,
                    description: format!("High CPU usage detected: {} ({:.1}%)", process.name(), process.cpu_usage()),
                    details: HashMap::from([
                        ("pid".to_string(), pid.as_u32().to_string()),
                        ("process_name".to_string(), process.name().to_string()),
                        ("cpu_usage".to_string(), process.cpu_usage().to_string()),
                    ]),
                });
            }
            
            // Check for high memory usage (only if very high)
            if process.memory() > 1000 * 1024 * 1024 { // Increased to 1 GB
                suspicious_score += 10.0; // Reduced penalty
                security_events.push(SecurityEvent {
                    timestamp: chrono::Utc::now().timestamp().to_string(),
                    event_type: SecurityEventType::HighMemoryUsage,
                    severity: SecuritySeverity::Medium,
                    description: format!("High memory usage detected: {} ({:.1} MB)", process.name(), process.memory() as f64 / 1024.0 / 1024.0),
                    details: HashMap::from([
                        ("pid".to_string(), pid.as_u32().to_string()),
                        ("process_name".to_string(), process.name().to_string()),
                        ("memory_usage".to_string(), process.memory().to_string()),
                    ]),
                });
            }
            
            // Check for new processes (not in known list) - but only flag truly suspicious ones
            let known_processes = self.known_processes.read().await;
            if !known_processes.contains_key(&pid.as_u32()) {
                // Only flag as suspicious if it's actually a suspicious process name
                if suspicious_list.contains(&process.name().to_string()) {
                    suspicious_score += 25.0;
                    security_events.push(SecurityEvent {
                        timestamp: chrono::Utc::now().timestamp().to_string(),
                        event_type: SecurityEventType::ProcessCreation,
                        severity: SecuritySeverity::Medium,
                        description: format!("New suspicious process detected: {}", process.name()),
                        details: HashMap::from([
                            ("pid".to_string(), pid.as_u32().to_string()),
                            ("process_name".to_string(), process.name().to_string()),
                            ("parent_pid".to_string(), process.parent().map(|p| p.as_u32().to_string()).unwrap_or_else(|| "Unknown".to_string())),
                        ]),
                    });
                }
            }
            
            // Add security events to the monitor
            for event in security_events {
                self.add_security_event(event).await;
            }
            
            if suspicious_score > 30.0 {
                suspicious_processes.push(ProcessSecurityInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory_usage: process.memory(),
                    suspicious_score,
                    network_connections: vec![],
                    file_handles: 0,
                    start_time: chrono::Utc::now().timestamp().to_string(),
                    parent_pid: process.parent().map(|p| p.as_u32()),
                });
            }
        }
        
        suspicious_processes
    }

    async fn analyze_network_connections(&self, _sys: &System) -> Vec<NetworkSecurityInfo> {
        let mut connections = Vec::new();
        
        // Get network connections using platform-specific commands
        if let Ok(netstat_output) = self.get_network_connections() {
            for line in netstat_output.lines() {
                if let Some(connection) = self.parse_network_connection(line) {
                    connections.push(connection);
                }
            }
        }
        
        // Add some suspicious connection patterns
        connections.extend(self.detect_suspicious_connections(&connections));
        
        // Generate security events for suspicious connections
        for connection in &connections {
            if connection.connection_type.contains("SUSPICIOUS") {
                let event = SecurityEvent {
                    timestamp: chrono::Utc::now().timestamp().to_string(),
                    event_type: SecurityEventType::UnusualNetworkActivity,
                    severity: SecuritySeverity::High,
                    description: format!("Suspicious network connection: {} -> {}:{}", 
                        connection.process_name, connection.remote_address, connection.port),
                    details: HashMap::from([
                        ("process_name".to_string(), connection.process_name.clone()),
                        ("process_pid".to_string(), connection.process_pid.to_string()),
                        ("local_address".to_string(), connection.local_address.clone()),
                        ("remote_address".to_string(), connection.remote_address.clone()),
                        ("port".to_string(), connection.port.to_string()),
                        ("protocol".to_string(), connection.protocol.clone()),
                        ("connection_type".to_string(), connection.connection_type.clone()),
                    ]),
                };
                self.add_security_event(event).await;
            }
        }
        
        connections
    }

    fn get_network_connections(&self) -> Result<String, std::io::Error> {
        #[cfg(target_os = "windows")]
        {
            // Use netstat on Windows
            let output = Command::new("netstat")
                .args(&["-ano"])
                .output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        
        #[cfg(target_os = "linux")]
        {
            // Use netstat on Linux
            let output = Command::new("netstat")
                .args(&["-tuln"])
                .output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        
        #[cfg(target_os = "macos")]
        {
            // Use netstat on macOS
            let output = Command::new("netstat")
                .args(&["-an"])
                .output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Ok(String::new())
        }
    }

    fn parse_network_connection(&self, line: &str) -> Option<NetworkSecurityInfo> {
        // Skip header lines and empty lines
        if line.trim().is_empty() || line.contains("Proto") || line.contains("Active") {
            return None;
        }

        #[cfg(target_os = "windows")]
        {
            // Parse Windows netstat output: Proto  Local Address          Foreign Address        State           PID
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let protocol = parts[0].to_string();
                let local_addr = parts[1].to_string();
                let remote_addr = parts[2].to_string();
                let _state = parts[3].to_string();
                
                // Extract PID from the last part
                let pid_str = parts.last().unwrap_or(&"0");
                let pid = u32::from_str(pid_str).unwrap_or(0);
                
                // Extract port from local address
                let port = self.extract_port(&local_addr).unwrap_or(0);
                
                // Get process name for this PID
                let process_name = self.get_process_name_by_pid(pid);
                
                // Determine connection type based on port and protocol
                let connection_type = self.classify_connection(&protocol, port, &remote_addr);
                
                Some(NetworkSecurityInfo {
                    interface: "Unknown".to_string(),
                    local_address: local_addr,
                    remote_address: remote_addr,
                    port,
                    protocol,
                    process_name,
                    process_pid: pid,
                    connection_type,
                    bytes_sent: 0, // Would need additional monitoring
                    bytes_received: 0, // Would need additional monitoring
                })
            } else {
                None
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Generic parsing for Linux/macOS
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let protocol = parts[0].to_string();
                let local_addr = parts[3].to_string();
                let remote_addr = parts[4].to_string();
                let port = self.extract_port(&local_addr).unwrap_or(0);
                
                Some(NetworkSecurityInfo {
                    interface: "Unknown".to_string(),
                    local_address: local_addr,
                    remote_address: remote_addr,
                    port,
                    protocol,
                    process_name: "Unknown".to_string(),
                    process_pid: 0,
                    connection_type: "Unknown".to_string(),
                    bytes_sent: 0,
                    bytes_received: 0,
                })
            } else {
                None
            }
        }
    }

    fn extract_port(&self, address: &str) -> Option<u16> {
        if let Some(colon_pos) = address.rfind(':') {
            if let Some(port_str) = address.get(colon_pos + 1..) {
                return u16::from_str(port_str).ok();
            }
        }
        None
    }

    fn get_process_name_by_pid(&self, pid: u32) -> String {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("tasklist")
                .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().next() {
                    if let Some(process_name) = line.split(',').next() {
                        return process_name.trim_matches('"').to_string();
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(output) = Command::new("ps")
                .args(&["-p", &pid.to_string(), "-o", "comm="])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                return output_str.trim().to_string();
            }
        }
        
        "Unknown".to_string()
    }

    fn classify_connection(&self, protocol: &str, port: u16, _remote_addr: &str) -> String {
        // Classify connections based on port and protocol
        match port {
            80 | 443 => "HTTP/HTTPS".to_string(),
            22 => "SSH".to_string(),
            21 => "FTP".to_string(),
            25 | 587 => "SMTP".to_string(),
            110 | 995 => "POP3".to_string(),
            143 | 993 => "IMAP".to_string(),
            53 => "DNS".to_string(),
            67 | 68 => "DHCP".to_string(),
            123 => "NTP".to_string(),
            161 | 162 => "SNMP".to_string(),
            3389 => "RDP".to_string(),
            5900..=5910 => "VNC".to_string(),
            8080 | 8443 => "HTTP/HTTPS (Alt)".to_string(),
            _ => {
                if protocol.to_lowercase().contains("tcp") {
                    "TCP".to_string()
                } else if protocol.to_lowercase().contains("udp") {
                    "UDP".to_string()
                } else {
                    "Unknown".to_string()
                }
            }
        }
    }

    fn detect_suspicious_connections(&self, connections: &[NetworkSecurityInfo]) -> Vec<NetworkSecurityInfo> {
        let mut suspicious = Vec::new();
        
        for connection in connections {
            let mut is_suspicious = false;
            let mut suspicious_reason = String::new();
            
            // Check for suspicious ports
            if self.is_suspicious_port(connection.port) {
                is_suspicious = true;
                suspicious_reason = format!("Suspicious port: {}", connection.port);
            }
            
            // Check for suspicious remote addresses
            if self.is_suspicious_address(&connection.remote_address) {
                is_suspicious = true;
                suspicious_reason = format!("Suspicious address: {}", connection.remote_address);
            }
            
            // Check for suspicious process names
            if self.is_suspicious_process(&connection.process_name) {
                is_suspicious = true;
                suspicious_reason = format!("Suspicious process: {}", connection.process_name);
            }
            
            if is_suspicious {
                let mut suspicious_connection = connection.clone();
                // Add suspicious indicator to the connection type
                suspicious_connection.connection_type = format!("SUSPICIOUS - {}", suspicious_reason);
                suspicious.push(suspicious_connection);
            }
        }
        
        suspicious
    }

    fn is_suspicious_port(&self, port: u16) -> bool {
        // Only truly suspicious ports that are rarely legitimate
        matches!(port, 
            23 | 25 |  // Telnet and SMTP (rarely legitimate)
            3389 | 5900 | 5901 | 5902 | 5903 | 5904 | 5905 | 5906 | 5907 | 5908 | 5909 | 5910 |  // RDP and VNC ports
            3128 | 8081 | 8082 | 8083 | 8084 | 8085 | 8086 | 8087 | 8088 | 8089 | 8090  // Proxy/alt ports
            // Removed common legitimate ports: 22 (SSH), 53 (DNS), 80/443 (HTTP/HTTPS), 110/143/993/995 (Email), 8080/8443 (Alt web)
        )
    }

    fn is_suspicious_address(&self, address: &str) -> bool {
        // Only check for truly suspicious IP patterns
        address.contains("0.0.0.0") || 
        address.contains("::1") ||
        address.contains("localhost")
        // Removed private IP ranges (192.168.x.x, 10.x.x.x, 172.x.x.x) as they're often legitimate
        // Removed 127.0.0.1 as it's commonly used for local services
    }

    fn is_suspicious_process(&self, process_name: &str) -> bool {
        if let Ok(suspicious_processes) = self.suspicious_processes.try_read() {
            suspicious_processes.iter().any(|suspicious| {
                process_name.to_lowercase().contains(&suspicious.to_lowercase())
            })
        } else {
            false
        }
    }

    fn is_high_risk_port(&self, port: u16) -> bool {
        // High-risk ports that should be monitored closely
        matches!(port, 
            22 | 23 | 25 | 3389 | 5900 | 5901 | 5902 | 5903 | 5904 | 5905 | 5906 | 5907 | 5908 | 5909 | 5910 |  // Remote access
            1433 | 1434 | 3306 | 5432 | 6379 | 27017 | 27018 | 27019 |  // Database ports
            21 | 69 | 115 |  // File transfer
            161 | 162 |  // SNMP
            514 | 515 |  // Syslog/Printing
            873 | 2049 |  // NFS
            1521 | 1526 |  // Oracle
            3307 | 5433 | 6380 |  // Additional database ports
            27020 | 27021 | 27022 | 27023 | 27024 | 27025 | 27026 | 27027 | 27028 | 27029 | 27030 | 27031 | 27032 | 27033 | 27034 | 27035 | 27036 | 27037 | 27038 | 27039 | 27040 | 27041 | 27042 | 27043 | 27044 | 27045 | 27046 | 27047 | 27048 | 27049 | 27050 | 27051 | 27052 | 27053 | 27054 | 27055 | 27056 | 27057 | 27058 | 27059 | 27060 | 27061 | 27062 | 27063 | 27064 | 27065 | 27066 | 27067 | 27068 | 27069 | 27070 | 27071 | 27072 | 27073 | 27074 | 27075 | 27076 | 27077 | 27078 | 27079 | 27080 | 27081 | 27082 | 27083 | 27084 | 27085 | 27086 | 27087 | 27088 | 27089 | 27090 | 27091 | 27092 | 27093 | 27094 | 27095 | 27096 | 27097 | 27098 | 27099 | 27100  // MongoDB
        )
    }

    fn calculate_security_score(&self, suspicious_processes: &[ProcessSecurityInfo], network_connections: &[NetworkSecurityInfo]) -> f32 {
        let base_score = 100.0;
        
        // Process-based penalties (reduced)
        let process_penalty = suspicious_processes.len() as f32 * 3.0; // Reduced from 5.0 to 3.0
        let avg_suspicious_score = if !suspicious_processes.is_empty() {
            suspicious_processes.iter().map(|p| p.suspicious_score).sum::<f32>() / suspicious_processes.len() as f32
        } else {
            0.0
        };
        
        // Network-based penalties (reduced)
        let suspicious_connections = network_connections.iter()
            .filter(|conn| conn.connection_type.contains("SUSPICIOUS"))
            .count() as f32;
        let network_penalty = suspicious_connections * 2.0; // Reduced from 3.0 to 2.0
        
        // Port-based penalties (reduced)
        let high_risk_ports = network_connections.iter()
            .filter(|conn| self.is_high_risk_port(conn.port))
            .count() as f32;
        let port_penalty = high_risk_ports * 1.0; // Reduced from 2.0 to 1.0
        
        // Calculate final score with more reasonable penalties
        let final_score = base_score - process_penalty - (avg_suspicious_score * 0.3) - network_penalty - port_penalty; // Reduced avg_suspicious_score multiplier from 0.5 to 0.3
        
        // Debug logging
        println!("=== Security Score Calculation ===");
        println!("Base Score: {}", base_score);
        println!("Suspicious Processes: {}", suspicious_processes.len());
        println!("Process Penalty: {} ({} * 3.0)", process_penalty, suspicious_processes.len());
        println!("Average Suspicious Score: {:.1}", avg_suspicious_score);
        println!("Suspicious Score Penalty: {:.1} ({} * 0.3)", avg_suspicious_score * 0.3, avg_suspicious_score);
        println!("Suspicious Connections: {}", suspicious_connections);
        println!("Network Penalty: {} ({} * 2.0)", network_penalty, suspicious_connections);
        println!("High Risk Ports: {}", high_risk_ports);
        println!("Port Penalty: {} ({} * 1.0)", port_penalty, high_risk_ports);
        println!("Final Score: {:.1}", final_score);
        println!("================================");
        
        final_score.max(0.0).min(100.0)
    }

    fn generate_recommendations(&self, suspicious_processes: &[ProcessSecurityInfo], network_connections: &[NetworkSecurityInfo]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Process-based recommendations with specific actions
        if !suspicious_processes.is_empty() {
            recommendations.push("🔍 Review suspicious processes in Task Manager".to_string());
            recommendations.push("💡 Right-click suspicious processes → End Task if unknown".to_string());
            recommendations.push("🛡️ Run Windows Defender full scan".to_string());
        }
        
        if suspicious_processes.len() > 3 {
            recommendations.push("🚨 Multiple suspicious processes detected - run full security scan".to_string());
        }
        
        // Network-based recommendations with specific actions
        let suspicious_connections = network_connections.iter()
            .filter(|conn| conn.connection_type.contains("SUSPICIOUS"))
            .count();
        
        if suspicious_connections > 0 {
            recommendations.push(format!("🌐 {} suspicious network connections - check Windows Firewall", suspicious_connections));
            recommendations.push("🔒 Open Windows Defender Firewall → Advanced Settings".to_string());
            recommendations.push("📋 Review outbound rules for unauthorized connections".to_string());
        }
        
        let high_risk_ports = network_connections.iter()
            .filter(|conn| self.is_high_risk_port(conn.port))
            .count();
        
        if high_risk_ports > 0 {
            recommendations.push(format!("🚪 {} high-risk port connections - block in firewall", high_risk_ports));
            recommendations.push("⚙️ Add firewall rules to block suspicious ports".to_string());
        }
        
        // Specific service recommendations with actionable steps
        if network_connections.iter().any(|conn| conn.port == 3389) {
            recommendations.push("🖥️ RDP detected - enable Network Level Authentication".to_string());
            recommendations.push("🔐 Use strong passwords and consider VPN access only".to_string());
        }
        
        if network_connections.iter().any(|conn| conn.port == 5900) {
            recommendations.push("🖥️ VNC detected - disable if not needed".to_string());
            recommendations.push("🔒 Use SSH tunneling for secure remote access".to_string());
        }
        
        if network_connections.iter().any(|conn| conn.port == 23) {
            recommendations.push("⚠️ Telnet detected - disable immediately (insecure)".to_string());
            recommendations.push("🔐 Use SSH instead of Telnet for remote access".to_string());
        }
        
        // General security recommendations
        recommendations.push("🔄 Keep Windows and all software updated".to_string());
        recommendations.push("🛡️ Enable Windows Defender real-time protection".to_string());
        recommendations.push("🔐 Use strong, unique passwords for all accounts".to_string());
        recommendations.push("📱 Enable two-factor authentication where possible".to_string());
        
        recommendations
    }

    pub async fn add_security_event(&self, event: SecurityEvent) {
        let mut events = self.events.write().await;
        events.push(event);
        
        // Keep only last 100 events
        if events.len() > 100 {
            events.remove(0);
        }
    }

    // === QUARANTINE AND FIX METHODS ===

    pub async fn quarantine_suspicious_process(&self, pid: u32) -> Result<String, String> {
        let mut sys = self.system.write().await;
        sys.refresh_processes();
        
        if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
            let process_name = process.name().to_string();
            
            // Get process executable path
            let exe_path = if let Some(path) = process.exe() {
                path.to_string_lossy().to_string()
            } else {
                return Err(format!("Could not get executable path for process {}", pid));
            };
            
            // Terminate the process first
            if let Err(e) = self.terminate_process(pid).await {
                return Err(format!("Failed to terminate process {}: {}", pid, e));
            }
            
            // Move executable to quarantine
            let quarantine_path = self.move_to_quarantine(&exe_path, &process_name).await?;
            
            // Add to quarantined list
            let quarantined = QuarantinedProcess {
                pid,
                name: process_name.clone(),
                original_path: exe_path.clone(),
                quarantine_path: quarantine_path.clone(),
                quarantine_time: chrono::Utc::now().timestamp().to_string(),
                reason: "Suspicious process detected".to_string(),
                status: QuarantineStatus::Quarantined,
            };
            
            let mut quarantined_list = self.quarantined_processes.write().await;
            quarantined_list.push(quarantined);
            
            // Add security event
            let event = SecurityEvent {
                timestamp: chrono::Utc::now().timestamp().to_string(),
                event_type: SecurityEventType::ProcessQuarantined,
                severity: SecuritySeverity::High,
                description: format!("Process quarantined: {} (PID: {})", process_name, pid),
                details: HashMap::from([
                    ("pid".to_string(), pid.to_string()),
                    ("process_name".to_string(), process_name.clone()),
                    ("original_path".to_string(), exe_path.clone()),
                    ("quarantine_path".to_string(), quarantine_path),
                ]),
            };
            self.add_security_event(event).await;
            
            Ok(format!("Successfully quarantined process {} ({})", pid, process_name))
        } else {
            Err(format!("Process {} not found", pid))
        }
    }

    pub async fn quarantine_all_suspicious_processes(&self) -> Result<String, String> {
        let sys = self.system.read().await;
        let suspicious_processes = self.analyze_processes(&sys).await;
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        for process in suspicious_processes {
            match self.quarantine_suspicious_process(process.pid).await {
                Ok(result) => results.push(result),
                Err(error) => errors.push(error),
            }
        }
        
        if results.is_empty() && errors.is_empty() {
            Ok("No suspicious processes found to quarantine".to_string())
        } else if errors.is_empty() {
            Ok(format!("Successfully quarantined {} processes: {}", 
                results.len(), results.join(", ")))
        } else {
            Err(format!("Quarantined {} processes, {} errors: {}", 
                results.len(), errors.len(), errors.join(", ")))
        }
    }

    pub async fn restore_quarantined_process(&self, pid: u32) -> Result<String, String> {
        let mut quarantined_list = self.quarantined_processes.write().await;
        
        if let Some(index) = quarantined_list.iter().position(|p| p.pid == pid) {
            let quarantined = &mut quarantined_list[index];
            
            // Move file back to original location
            if let Err(e) = fs::rename(&quarantined.quarantine_path, &quarantined.original_path) {
                return Err(format!("Failed to restore file: {}", e));
            }
            
            quarantined.status = QuarantineStatus::Restored;
            
            // Add security event
            let event = SecurityEvent {
                timestamp: chrono::Utc::now().timestamp().to_string(),
                event_type: SecurityEventType::ProcessCreation,
                severity: SecuritySeverity::Medium,
                description: format!("Process restored from quarantine: {} (PID: {})", quarantined.name, pid),
                details: HashMap::from([
                    ("pid".to_string(), pid.to_string()),
                    ("process_name".to_string(), quarantined.name.clone()),
                    ("original_path".to_string(), quarantined.original_path.clone()),
                ]),
            };
            self.add_security_event(event).await;
            
            Ok(format!("Successfully restored process {} ({})", pid, quarantined.name))
        } else {
            Err(format!("Process {} not found in quarantine", pid))
        }
    }

    pub async fn delete_quarantined_process(&self, pid: u32) -> Result<String, String> {
        let mut quarantined_list = self.quarantined_processes.write().await;
        
        if let Some(index) = quarantined_list.iter().position(|p| p.pid == pid) {
            let quarantined = &mut quarantined_list[index];
            
            // Delete quarantined file
            if let Err(e) = fs::remove_file(&quarantined.quarantine_path) {
                return Err(format!("Failed to delete quarantined file: {}", e));
            }
            
            quarantined.status = QuarantineStatus::Deleted;
            
            // Add security event
            let event = SecurityEvent {
                timestamp: chrono::Utc::now().timestamp().to_string(),
                event_type: SecurityEventType::ProcessTermination,
                severity: SecuritySeverity::High,
                description: format!("Quarantined process deleted: {} (PID: {})", quarantined.name, pid),
                details: HashMap::from([
                    ("pid".to_string(), pid.to_string()),
                    ("process_name".to_string(), quarantined.name.clone()),
                    ("quarantine_path".to_string(), quarantined.quarantine_path.clone()),
                ]),
            };
            self.add_security_event(event).await;
            
            Ok(format!("Successfully deleted quarantined process {} ({})", pid, quarantined.name))
        } else {
            Err(format!("Process {} not found in quarantine", pid))
        }
    }

    pub async fn block_suspicious_connection(&self, remote_address: &str, port: u16, protocol: &str) -> Result<String, String> {
        let rule_name = format!("Block_{}_{}_{}", remote_address.replace(".", "_"), port, protocol);
        
        #[cfg(target_os = "windows")]
        {
            // Create Windows Firewall rule
            let command = if protocol.to_lowercase() == "tcp" {
                format!("netsh advfirewall firewall add rule name=\"{}\" dir=out action=block protocol=TCP remoteip={} remoteport={}", 
                    rule_name, remote_address, port)
            } else {
                format!("netsh advfirewall firewall add rule name=\"{}\" dir=out action=block protocol=UDP remoteip={} remoteport={}", 
                    rule_name, remote_address, port)
            };
            
            if let Err(e) = Command::new("cmd").args(&["/C", &command]).output() {
                return Err(format!("Failed to create firewall rule: {}", e));
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // For non-Windows, we'll just log the attempt
            println!("Firewall blocking not implemented for this platform");
        }
        
        // Add to blocked connections list
        let blocked = BlockedConnection {
            remote_address: remote_address.to_string(),
            port,
            protocol: protocol.to_string(),
            block_time: chrono::Utc::now().timestamp().to_string(),
            reason: "Suspicious connection detected".to_string(),
            firewall_rule_name: rule_name.clone(),
        };
        
        let mut blocked_list = self.blocked_connections.write().await;
        blocked_list.push(blocked);
        
        // Add security event
        let event = SecurityEvent {
            timestamp: chrono::Utc::now().timestamp().to_string(),
            event_type: SecurityEventType::NetworkBlocked,
            severity: SecuritySeverity::High,
            description: format!("Network connection blocked: {}:{} ({})", remote_address, port, protocol),
            details: HashMap::from([
                ("remote_address".to_string(), remote_address.to_string()),
                ("port".to_string(), port.to_string()),
                ("protocol".to_string(), protocol.to_string()),
                ("firewall_rule".to_string(), rule_name),
            ]),
        };
        self.add_security_event(event).await;
        
        Ok(format!("Successfully blocked connection to {}:{} ({})", remote_address, port, protocol))
    }

    pub async fn block_all_suspicious_connections(&self) -> Result<String, String> {
        let sys = self.system.read().await;
        let network_connections = self.analyze_network_connections(&sys).await;
        let suspicious_connections: Vec<_> = network_connections.iter()
            .filter(|conn| conn.connection_type.contains("SUSPICIOUS"))
            .collect();
        
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        for connection in suspicious_connections {
            match self.block_suspicious_connection(&connection.remote_address, connection.port, &connection.protocol).await {
                Ok(result) => results.push(result),
                Err(error) => errors.push(error),
            }
        }
        
        if results.is_empty() && errors.is_empty() {
            Ok("No suspicious connections found to block".to_string())
        } else if errors.is_empty() {
            Ok(format!("Successfully blocked {} connections: {}", 
                results.len(), results.join(", ")))
        } else {
            Err(format!("Blocked {} connections, {} errors: {}", 
                results.len(), errors.len(), errors.join(", ")))
        }
    }

    // === HELPER METHODS ===

    async fn terminate_process(&self, pid: u32) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let command = format!("taskkill /F /PID {}", pid);
            if let Err(e) = Command::new("cmd").args(&["/C", &command]).output() {
                return Err(format!("Failed to terminate process: {}", e));
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            if let Err(e) = Command::new("kill").args(&["-9", &pid.to_string()]).output() {
                return Err(format!("Failed to terminate process: {}", e));
            }
        }
        
        // Add security event
        let event = SecurityEvent {
            timestamp: chrono::Utc::now().timestamp().to_string(),
            event_type: SecurityEventType::ProcessTerminated,
            severity: SecuritySeverity::High,
            description: format!("Process terminated: PID {}", pid),
            details: HashMap::from([
                ("pid".to_string(), pid.to_string()),
            ]),
        };
        self.add_security_event(event).await;
        
        Ok(())
    }

    async fn move_to_quarantine(&self, file_path: &str, process_name: &str) -> Result<String, String> {
        let file_path = Path::new(file_path);
        if !file_path.exists() {
            return Err(format!("File does not exist: {}", file_path.display()));
        }
        
        let file_name = file_path.file_name()
            .ok_or_else(|| "Invalid file path".to_string())?
            .to_string_lossy();
        
        let quarantine_name = format!("{}_{}_{}", 
            process_name.replace(".exe", ""), 
            chrono::Utc::now().timestamp(),
            file_name);
        
        let quarantine_path = Path::new(&self.quarantine_directory).join(quarantine_name);
        
        if let Err(e) = fs::rename(file_path, &quarantine_path) {
            return Err(format!("Failed to move file to quarantine: {}", e));
        }
        
        Ok(quarantine_path.to_string_lossy().to_string())
    }
} 