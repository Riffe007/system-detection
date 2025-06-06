use system_monitor::backend::{CpuMonitor, MemoryMonitor};
use system_monitor::core::{Monitor, MonitorConfig};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Quick System Monitor Test");
    println!("=========================\n");
    
    // Test CPU Monitor with timeout
    println!("Testing CPU Monitor...");
    let mut cpu_monitor = CpuMonitor::new();
    cpu_monitor.initialize(MonitorConfig::default()).await?;
    
    match timeout(Duration::from_secs(5), cpu_monitor.collect()).await {
        Ok(Ok(metrics)) => println!("✓ CPU: {} metrics collected", metrics.len()),
        Ok(Err(e)) => println!("✗ CPU: Error - {}", e),
        Err(_) => println!("✗ CPU: Timeout"),
    }
    
    // Test Memory Monitor with timeout
    println!("\nTesting Memory Monitor...");
    let mut memory_monitor = MemoryMonitor::new();
    memory_monitor.initialize(MonitorConfig::default()).await?;
    
    match timeout(Duration::from_secs(5), memory_monitor.collect()).await {
        Ok(Ok(metrics)) => println!("✓ Memory: {} metrics collected", metrics.len()),
        Ok(Err(e)) => println!("✗ Memory: Error - {}", e),
        Err(_) => println!("✗ Memory: Timeout"),
    }
    
    println!("\nTest completed!");
    
    Ok(())
}