use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::{System, Disks, Networks};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub category: BottleneckCategory,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub current_value: f32,
    pub threshold: f32,
    pub recommendation: String,
    pub impact_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckCategory {
    CPU,
    Memory,
    Disk,
    Network,
    Process,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub title: String,
    pub description: String,
    pub priority: String,
    pub estimated_impact: f32,
    pub difficulty: String,
    pub action_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_usage: f32,
    pub io_wait: f32,
    pub load_average: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPerformance {
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_percent: f32,
    pub priority: String,
    pub threads: u32,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub performance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_score: f32,
    pub cpu_health: f32,
    pub memory_health: f32,
    pub disk_health: f32,
    pub network_health: f32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    pub timestamp: String,
    pub resource_usage: ResourceUsage,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub top_processes: Vec<ProcessPerformance>,
    pub system_health: SystemHealth,
    pub overall_score: f32,
    pub recommendations: Vec<OptimizationRecommendation>,
}

pub struct OptimizationMonitor {
    system: Arc<RwLock<System>>,
    performance_thresholds: Arc<RwLock<HashMap<String, f32>>>,
}

impl OptimizationMonitor {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("cpu_critical".to_string(), 90.0);
        thresholds.insert("cpu_high".to_string(), 70.0);
        thresholds.insert("memory_critical".to_string(), 85.0);
        thresholds.insert("memory_high".to_string(), 70.0);
        thresholds.insert("disk_critical".to_string(), 90.0);
        thresholds.insert("disk_high".to_string(), 80.0);
        
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            performance_thresholds: Arc::new(RwLock::new(thresholds)),
        }
    }

    pub async fn collect_optimization_metrics(&self) -> Result<OptimizationMetrics, String> {
        let mut sys = self.system.write().await;
        sys.refresh_all();
        drop(sys);
        
        let sys = self.system.read().await;
        
        // Real resource analysis
        let resource_usage = self.collect_resource_usage(&sys).await;
        
        // Real bottleneck detection
        let bottlenecks = self.identify_bottlenecks(&resource_usage).await;
        
        // Real process performance analysis
        let top_processes = self.analyze_process_performance(&sys).await;
        
        // Calculate system health
        let system_health = self.calculate_system_health(&resource_usage, &bottlenecks);
        
        // Calculate overall performance score
        let overall_score = self.calculate_performance_score(&resource_usage, &bottlenecks);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&bottlenecks, &resource_usage).await;
        
        Ok(OptimizationMetrics {
            timestamp: chrono::Utc::now().timestamp().to_string(),
            resource_usage,
            bottlenecks,
            top_processes,
            system_health,
            overall_score,
            recommendations,
        })
    }

    async fn collect_resource_usage(&self, sys: &System) -> ResourceUsage {
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_usage = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;
        
        // Real disk usage calculation
        let mut total_disk_space = 0u64;
        let mut used_disk_space = 0u64;
        
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            total_disk_space += disk.total_space();
            used_disk_space += disk.total_space() - disk.available_space();
        }
        
        let disk_usage = if total_disk_space > 0 {
            (used_disk_space as f32 / total_disk_space as f32) * 100.0
        } else {
            0.0
        };
        
        // Real network usage calculation
        let mut total_network_usage = 0.0;
        let networks = Networks::new_with_refreshed_list();
        
        for (_, network) in &networks {
            let received = network.received() as f32;
            let transmitted = network.transmitted() as f32;
            total_network_usage += received + transmitted;
        }
        
        // Convert to percentage (normalize based on typical network speeds)
        let network_usage = if total_network_usage > 0.0 {
            let normalized = total_network_usage / (1024.0 * 1024.0 * 100.0); // Normalize to MB/s
            if normalized > 100.0 { 100.0 } else { normalized }
        } else {
            0.0
        };
        
        ResourceUsage {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_usage,
            io_wait: 0.0, // Would need additional monitoring
            load_average: [0.0, 0.0, 0.0], // Would need additional monitoring
        }
    }

    async fn identify_bottlenecks(&self, resource_usage: &ResourceUsage) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();
        let thresholds = self.performance_thresholds.read().await;
        
        // CPU bottlenecks
        if resource_usage.cpu_usage > *thresholds.get("cpu_critical").unwrap_or(&90.0) {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::CPU,
                severity: BottleneckSeverity::Critical,
                description: format!("Critical CPU usage detected: {:.1}%", resource_usage.cpu_usage),
                current_value: resource_usage.cpu_usage,
                threshold: *thresholds.get("cpu_critical").unwrap_or(&90.0),
                recommendation: "Close unnecessary applications and check for runaway processes. Consider restarting the system if the issue persists.".to_string(),
                impact_score: 0.9,
            });
        } else if resource_usage.cpu_usage > *thresholds.get("cpu_high").unwrap_or(&70.0) {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::CPU,
                severity: BottleneckSeverity::High,
                description: format!("High CPU usage detected: {:.1}%", resource_usage.cpu_usage),
                current_value: resource_usage.cpu_usage,
                threshold: *thresholds.get("cpu_high").unwrap_or(&70.0),
                recommendation: "Monitor CPU-intensive processes and consider optimization. Close unused browser tabs and background applications.".to_string(),
                impact_score: 0.7,
            });
        } else if resource_usage.cpu_usage > 50.0 {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::CPU,
                severity: BottleneckSeverity::Medium,
                description: format!("Moderate CPU usage detected: {:.1}%", resource_usage.cpu_usage),
                current_value: resource_usage.cpu_usage,
                threshold: 50.0,
                recommendation: "CPU usage is moderate. Monitor for any unusual spikes or patterns.".to_string(),
                impact_score: 0.4,
            });
        }
        
        // Memory bottlenecks
        if resource_usage.memory_usage > *thresholds.get("memory_critical").unwrap_or(&85.0) {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::Memory,
                severity: BottleneckSeverity::Critical,
                description: format!("Critical memory usage detected: {:.1}%", resource_usage.memory_usage),
                current_value: resource_usage.memory_usage,
                threshold: *thresholds.get("memory_critical").unwrap_or(&85.0),
                recommendation: "Close applications immediately and restart if necessary. Check for memory leaks in running applications.".to_string(),
                impact_score: 0.9,
            });
        } else if resource_usage.memory_usage > *thresholds.get("memory_high").unwrap_or(&70.0) {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::Memory,
                severity: BottleneckSeverity::High,
                description: format!("High memory usage detected: {:.1}%", resource_usage.memory_usage),
                current_value: resource_usage.memory_usage,
                threshold: *thresholds.get("memory_high").unwrap_or(&70.0),
                recommendation: "Check for memory leaks and close unused applications. Consider restarting memory-intensive applications.".to_string(),
                impact_score: 0.7,
            });
        } else if resource_usage.memory_usage > 60.0 {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::Memory,
                severity: BottleneckSeverity::Medium,
                description: format!("Moderate memory usage detected: {:.1}%", resource_usage.memory_usage),
                current_value: resource_usage.memory_usage,
                threshold: 60.0,
                recommendation: "Memory usage is moderate. Monitor for any unusual patterns.".to_string(),
                impact_score: 0.4,
            });
        }
        
        // Disk bottlenecks
        if resource_usage.disk_usage > *thresholds.get("disk_critical").unwrap_or(&90.0) {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::Disk,
                severity: BottleneckSeverity::Critical,
                description: format!("Critical disk usage detected: {:.1}%", resource_usage.disk_usage),
                current_value: resource_usage.disk_usage,
                threshold: *thresholds.get("disk_critical").unwrap_or(&90.0),
                recommendation: "Free up disk space immediately. Delete temporary files, uninstall unused applications, and move large files to external storage.".to_string(),
                impact_score: 0.8,
            });
        } else if resource_usage.disk_usage > *thresholds.get("disk_high").unwrap_or(&80.0) {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::Disk,
                severity: BottleneckSeverity::High,
                description: format!("High disk usage detected: {:.1}%", resource_usage.disk_usage),
                current_value: resource_usage.disk_usage,
                threshold: *thresholds.get("disk_high").unwrap_or(&80.0),
                recommendation: "Free up disk space. Delete temporary files and uninstall unused applications.".to_string(),
                impact_score: 0.6,
            });
        } else if resource_usage.disk_usage > 70.0 {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::Disk,
                severity: BottleneckSeverity::Medium,
                description: format!("Moderate disk usage detected: {:.1}%", resource_usage.disk_usage),
                current_value: resource_usage.disk_usage,
                threshold: 70.0,
                recommendation: "Consider freeing up some disk space for better performance.".to_string(),
                impact_score: 0.3,
            });
        }
        
        // Network bottlenecks (if network usage is high)
        if resource_usage.network_usage > 80.0 {
            bottlenecks.push(PerformanceBottleneck {
                category: BottleneckCategory::Network,
                severity: BottleneckSeverity::High,
                description: format!("High network usage detected: {:.1}%", resource_usage.network_usage),
                current_value: resource_usage.network_usage,
                threshold: 80.0,
                recommendation: "Check for background downloads or uploads. Monitor network-intensive applications.".to_string(),
                impact_score: 0.5,
            });
        }
        
        bottlenecks
    }

    async fn analyze_process_performance(&self, sys: &System) -> Vec<ProcessPerformance> {
        let mut processes: Vec<ProcessPerformance> = sys.processes()
            .iter()
            .map(|(_, process)| {
                let memory_percent = (process.memory() as f32 / sys.total_memory() as f32) * 100.0;
                let performance_score = 100.0 - (process.cpu_usage() + memory_percent * 0.5);
                
                // Get real thread count - simplified for now
                let thread_count = 1; // sysinfo doesn't provide easy thread count access
                
                // Get real IO stats if available
                let io_stats = process.disk_usage();
                let io_read = io_stats.read_bytes;
                let io_write = io_stats.written_bytes;
                
                ProcessPerformance {
                    name: process.name().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory_usage: process.memory(),
                    memory_percent,
                    priority: "Normal".to_string(), // Would need additional monitoring
                    threads: thread_count,
                    io_read_bytes: io_read,
                    io_write_bytes: io_write,
                    performance_score: performance_score.max(0.0),
                }
            })
            .collect();
        
        // Sort by performance score (worst first) and take top 10
        processes.sort_by(|a, b| a.performance_score.partial_cmp(&b.performance_score).unwrap());
        processes.truncate(10);
        processes
    }

    fn calculate_system_health(&self, resource_usage: &ResourceUsage, _bottlenecks: &[PerformanceBottleneck]) -> SystemHealth {
        let cpu_health = 100.0 - resource_usage.cpu_usage;
        let memory_health = 100.0 - resource_usage.memory_usage;
        let disk_health = 100.0 - resource_usage.disk_usage;
        let network_health = 100.0 - resource_usage.network_usage;
        
        let overall_score = (cpu_health + memory_health + disk_health + network_health) / 4.0;
        
        let status = if overall_score > 80.0 {
            "Excellent".to_string()
        } else if overall_score > 60.0 {
            "Good".to_string()
        } else if overall_score > 40.0 {
            "Fair".to_string()
        } else {
            "Poor".to_string()
        };
        
        SystemHealth {
            overall_score,
            cpu_health,
            memory_health,
            disk_health,
            network_health,
            status,
        }
    }

    fn calculate_performance_score(&self, resource_usage: &ResourceUsage, bottlenecks: &[PerformanceBottleneck]) -> f32 {
        let base_score = 100.0;
        let resource_penalty = (resource_usage.cpu_usage + resource_usage.memory_usage + resource_usage.disk_usage) / 3.0;
        let bottleneck_penalty = bottlenecks.len() as f32 * 5.0;
        
        let final_score = base_score - resource_penalty - bottleneck_penalty;
        final_score.max(0.0).min(100.0)
    }

    async fn generate_recommendations(&self, _bottlenecks: &[PerformanceBottleneck], resource_usage: &ResourceUsage) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        // CPU recommendations
        if resource_usage.cpu_usage > 70.0 {
            recommendations.push(OptimizationRecommendation {
                title: "Optimize CPU Usage".to_string(),
                description: "High CPU usage detected. Consider closing unnecessary applications.".to_string(),
                priority: "High".to_string(),
                estimated_impact: 0.8,
                difficulty: "Easy".to_string(),
                action_steps: vec![
                    "Close unused browser tabs".to_string(),
                    "End unnecessary background processes".to_string(),
                    "Check for malware or mining software".to_string(),
                ],
            });
        }
        
        // Memory recommendations
        if resource_usage.memory_usage > 70.0 {
            recommendations.push(OptimizationRecommendation {
                title: "Free Up Memory".to_string(),
                description: "High memory usage detected. Free up RAM for better performance.".to_string(),
                priority: "High".to_string(),
                estimated_impact: 0.7,
                difficulty: "Easy".to_string(),
                action_steps: vec![
                    "Close memory-intensive applications".to_string(),
                    "Restart applications that have been running for a long time".to_string(),
                    "Check for memory leaks".to_string(),
                ],
            });
        }
        
        // Disk recommendations
        if resource_usage.disk_usage > 80.0 {
            recommendations.push(OptimizationRecommendation {
                title: "Free Up Disk Space".to_string(),
                description: "Low disk space detected. Free up storage for better performance.".to_string(),
                priority: "Medium".to_string(),
                estimated_impact: 0.6,
                difficulty: "Medium".to_string(),
                action_steps: vec![
                    "Delete temporary files".to_string(),
                    "Uninstall unused applications".to_string(),
                    "Move large files to external storage".to_string(),
                ],
            });
        }
        
        // General optimization
        if recommendations.is_empty() {
            recommendations.push(OptimizationRecommendation {
                title: "System Optimization".to_string(),
                description: "Your system is performing well. Consider these maintenance tasks.".to_string(),
                priority: "Low".to_string(),
                estimated_impact: 0.3,
                difficulty: "Easy".to_string(),
                action_steps: vec![
                    "Update system drivers".to_string(),
                    "Run disk cleanup".to_string(),
                    "Defragment hard drive (if applicable)".to_string(),
                ],
            });
        }
        
        recommendations
    }
} 