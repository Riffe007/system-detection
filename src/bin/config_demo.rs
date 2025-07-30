use system_monitor::core::config::ConfigManager;
use system_monitor::services::MonitoringService;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("System Monitor Configuration Demo");
    println!("=================================\n");
    
    // Load or create configuration
    println!("Loading configuration...");
    let config_manager = ConfigManager::new()?;
    let config = config_manager.config();
    
    println!("✓ Configuration loaded from: {:?}", std::env::current_dir()?.join("config.toml"));
    println!("\nCurrent settings:");
    println!("  CPU monitoring interval: {}ms", config.monitoring.cpu.interval_ms);
    println!("  Memory monitoring interval: {}ms", config.monitoring.memory.interval_ms);
    println!("  Theme: {}", config.ui.theme);
    println!("  Graph history points: {}", config.ui.graph_history_points);
    
    // Initialize monitoring service
    println!("\nInitializing monitoring service...");
    let monitoring_service = MonitoringService::new();
    monitoring_service.initialize().await?;
    
    // Apply configuration
    println!("Applying configuration to monitoring service...");
    monitoring_service.apply_config(config).await?;
    
    // Start monitoring
    println!("Starting monitoring...");
    monitoring_service.start().await?;
    
    // Subscribe to metrics
    let mut receiver = monitoring_service.subscribe();
    
    println!("\nCollecting metrics for 5 seconds...\n");
    
    // Collect metrics for 5 seconds
    let start = std::time::Instant::now();
    let mut count = 0;
    
    while start.elapsed() < Duration::from_secs(5) {
        tokio::select! {
            Ok(metrics) = receiver.recv() => {
                count += 1;
                println!("Metrics #{}: CPU: {:.1}%, Memory: {:.1}%, Processes: {}", 
                    count,
                    metrics.cpu.usage_percent,
                    metrics.memory.usage_percent,
                    metrics.cpu.processes_total
                );
            }
            _ = sleep(Duration::from_secs(1)) => {
                // Timeout
            }
        }
    }
    
    // Stop monitoring
    println!("\nStopping monitoring...");
    monitoring_service.stop().await?;
    
    println!("✓ Demo completed successfully!");
    println!("\nConfiguration file saved at: {:?}", std::env::current_dir()?.join("config.toml"));
    
    Ok(())
}