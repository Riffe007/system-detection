#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod monitoring;

use std::sync::Arc;
use tauri::{Manager, State, Emitter};
use tokio::sync::RwLock;
use monitoring::{MonitoringService, SystemInfo, SystemMetrics};
use system_monitor::security::SecurityMetrics;
use system_monitor::optimization::OptimizationMetrics;

type ServiceState = Arc<RwLock<MonitoringService>>;

#[tauri::command]
async fn get_system_info(state: State<'_, ServiceState>) -> Result<SystemInfo, String> {
    println!("=== get_system_info called ===");
    let service = state.read().await;
    match service.get_system_info().await {
        Ok(info) => {
            println!("System info retrieved successfully:");
            println!("  Hostname: {}", info.hostname);
            println!("  OS: {} {}", info.os_name, info.os_version);
            println!("  CPU: {}", info.cpu_brand);
            println!("  Architecture: {}", info.architecture);
            println!("  CPU Cores: {}", info.cpu_cores);
            println!("  Total Memory: {} MB", info.total_memory / 1024 / 1024);
            Ok(info)
        }
        Err(e) => {
            println!("ERROR getting system info: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn start_monitoring(state: State<'_, ServiceState>, app: tauri::AppHandle) -> Result<(), String> {
    println!("=== start_monitoring called ===");
    let mut service = state.write().await;
    
    // Clone app handle for the callback
    let app_handle = app.clone();
    println!("App handle cloned for metrics callback");
    
    // Set up the metrics callback to emit events to the frontend
    service.set_metrics_callback(move |metrics| {
        println!("Emitting system-metrics event with {} processes", metrics.top_processes.len());
        let result = app_handle.emit("system-metrics", &metrics);
        if let Err(e) = result {
            println!("Error emitting system-metrics event: {}", e);
        }
    }).await;
    
    println!("Starting monitoring service...");
    service.start_monitoring().await;
    println!("Monitoring service started successfully");
    Ok(())
}

#[tauri::command]
async fn stop_monitoring(_state: State<'_, ServiceState>) -> Result<(), String> {
    // In this simple version, we don't stop the monitoring
    Ok(())
}

#[tauri::command]
async fn get_current_metrics(state: State<'_, ServiceState>) -> Result<SystemMetrics, String> {
    println!("=== get_current_metrics called ===");
    let service = state.read().await;
    match service.collect_metrics().await {
        Ok(metrics) => {
            println!("Current metrics collected successfully with {} processes", metrics.top_processes.len());
            println!("Returning metrics from get_current_metrics command");
            Ok(metrics)
        }
        Err(e) => {
            println!("ERROR collecting current metrics: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn get_security_metrics(state: State<'_, ServiceState>) -> Result<SecurityMetrics, String> {
    println!("=== get_security_metrics called ===");
    let service = state.read().await;
    match service.security_monitor.collect_security_metrics().await {
        Ok(metrics) => {
            println!("Security metrics collected successfully with {} suspicious processes", metrics.suspicious_processes.len());
            Ok(metrics)
        }
        Err(e) => {
            println!("ERROR collecting security metrics: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn get_optimization_metrics(state: State<'_, ServiceState>) -> Result<OptimizationMetrics, String> {
    println!("=== get_optimization_metrics called ===");
    let service = state.read().await;
    match service.optimization_monitor.collect_optimization_metrics().await {
        Ok(metrics) => {
            println!("Optimization metrics collected successfully");
            Ok(metrics)
        }
        Err(e) => {
            println!("ERROR collecting optimization metrics: {}", e);
            Err(e)
        }
    }
}

// === SECURITY QUARANTINE AND FIX COMMANDS ===

#[tauri::command]
async fn quarantine_suspicious_process(state: State<'_, ServiceState>, pid: u32) -> Result<String, String> {
    println!("=== quarantine_suspicious_process called for PID {} ===", pid);
    let service = state.read().await;
    match service.security_monitor.quarantine_suspicious_process(pid).await {
        Ok(result) => {
            println!("Successfully quarantined process {}: {}", pid, result);
            Ok(result)
        }
        Err(e) => {
            println!("ERROR quarantining process {}: {}", pid, e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn quarantine_all_suspicious_processes(state: State<'_, ServiceState>) -> Result<String, String> {
    println!("=== quarantine_all_suspicious_processes called ===");
    let service = state.read().await;
    match service.security_monitor.quarantine_all_suspicious_processes().await {
        Ok(result) => {
            println!("Successfully quarantined all suspicious processes: {}", result);
            Ok(result)
        }
        Err(e) => {
            println!("ERROR quarantining all suspicious processes: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn restore_quarantined_process(state: State<'_, ServiceState>, pid: u32) -> Result<String, String> {
    println!("=== restore_quarantined_process called for PID {} ===", pid);
    let service = state.read().await;
    match service.security_monitor.restore_quarantined_process(pid).await {
        Ok(result) => {
            println!("Successfully restored process {}: {}", pid, result);
            Ok(result)
        }
        Err(e) => {
            println!("ERROR restoring process {}: {}", pid, e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn delete_quarantined_process(state: State<'_, ServiceState>, pid: u32) -> Result<String, String> {
    println!("=== delete_quarantined_process called for PID {} ===", pid);
    let service = state.read().await;
    match service.security_monitor.delete_quarantined_process(pid).await {
        Ok(result) => {
            println!("Successfully deleted quarantined process {}: {}", pid, result);
            Ok(result)
        }
        Err(e) => {
            println!("ERROR deleting quarantined process {}: {}", pid, e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn block_suspicious_connection(state: State<'_, ServiceState>, remote_address: String, port: u16, protocol: String) -> Result<String, String> {
    println!("=== block_suspicious_connection called for {}:{} ({}) ===", remote_address, port, protocol);
    let service = state.read().await;
    match service.security_monitor.block_suspicious_connection(&remote_address, port, &protocol).await {
        Ok(result) => {
            println!("Successfully blocked connection: {}", result);
            Ok(result)
        }
        Err(e) => {
            println!("ERROR blocking connection: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn block_all_suspicious_connections(state: State<'_, ServiceState>) -> Result<String, String> {
    println!("=== block_all_suspicious_connections called ===");
    let service = state.read().await;
    match service.security_monitor.block_all_suspicious_connections().await {
        Ok(result) => {
            println!("Successfully blocked all suspicious connections: {}", result);
            Ok(result)
        }
        Err(e) => {
            println!("ERROR blocking all suspicious connections: {}", e);
            Err(e)
        }
    }
}



fn main() {
    println!("=== Starting System Monitor Tauri Application ===");
    
    // Initialize the monitoring service
    println!("Initializing monitoring service...");
    let service = Arc::new(RwLock::new(MonitoringService::new()));
    println!("Monitoring service initialized successfully");
    
    tauri::Builder::default()
        .manage(service)
        .setup(|app| {
            println!("=== Tauri App Setup ===");
            println!("App is initializing...");
            
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    println!("Opening devtools for main window");
                    window.open_devtools();
                    
                    // Log window properties
                    if let Ok(pos) = window.outer_position() {
                        println!("Window position: {:?}", pos);
                    }
                    if let Ok(size) = window.outer_size() {
                        println!("Window size: {:?}", size);
                    }
                } else {
                    println!("WARNING: Main window not found!");
                }
            }
            
            println!("Tauri setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            start_monitoring,
            stop_monitoring,
            get_current_metrics,
            get_security_metrics,
            get_optimization_metrics,
            quarantine_suspicious_process,
            quarantine_all_suspicious_processes,
            restore_quarantined_process,
            delete_quarantined_process,
            block_suspicious_connection,
            block_all_suspicious_connections
        ])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Focused(focused) => {
                    println!("Window {} focused: {}", window.label(), focused);
                }
                tauri::WindowEvent::Resized(size) => {
                    println!("Window {} resized to: {:?}", window.label(), size);
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}