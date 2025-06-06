use system_monitor::backend::{CpuMonitor, MemoryMonitor, GpuMonitor, StorageMonitor};
use system_monitor::core::{Monitor, MonitorConfig};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize monitoring
    println!("System Monitor Test Runner");
    println!("=========================\n");
    
    // Test CPU Monitor
    println!("Testing CPU Monitor...");
    let mut cpu_monitor = CpuMonitor::new();
    cpu_monitor.initialize(MonitorConfig::default()).await?;
    let cpu_metrics = cpu_monitor.collect().await?;
    println!("CPU Metrics collected: {} metrics", cpu_metrics.len());
    
    // Test Memory Monitor
    println!("\nTesting Memory Monitor...");
    let mut memory_monitor = MemoryMonitor::new();
    memory_monitor.initialize(MonitorConfig::default()).await?;
    let memory_metrics = memory_monitor.collect().await?;
    println!("Memory Metrics collected: {} metrics", memory_metrics.len());
    
    // Test GPU Monitor
    println!("\nTesting GPU Monitor...");
    let mut gpu_monitor = GpuMonitor::new();
    gpu_monitor.initialize(MonitorConfig::default()).await?;
    let gpu_metrics = gpu_monitor.collect().await?;
    println!("GPU Metrics collected: {} metrics", gpu_metrics.len());
    
    // Test Storage Monitor
    println!("\nTesting Storage Monitor...");
    let mut storage_monitor = StorageMonitor::new();
    storage_monitor.initialize(MonitorConfig::default()).await?;
    let storage_metrics = storage_monitor.collect().await?;
    println!("Storage Metrics collected: {} metrics", storage_metrics.len());
    
    println!("\nAll monitors tested successfully!");
    
    Ok(())
}