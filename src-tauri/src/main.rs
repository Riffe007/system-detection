#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod monitoring;

use std::sync::Arc;
use tauri::{Manager, State, Emitter};
use tokio::sync::RwLock;
use monitoring::{MonitoringService, SystemInfo, SystemMetrics};

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
            get_current_metrics
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