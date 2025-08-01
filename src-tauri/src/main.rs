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
    println!("start_monitoring called");
    let mut service = state.write().await;
    
    // Clone app handle for the callback
    let app_handle = app.clone();
    
    // Set up the metrics callback to emit events to the frontend
    service.set_metrics_callback(move |metrics| {
        let _ = app_handle.emit("system-metrics", &metrics);
    }).await;
    
    service.start_monitoring().await;
    Ok(())
}

#[tauri::command]
async fn stop_monitoring(_state: State<'_, ServiceState>) -> Result<(), String> {
    // In this simple version, we don't stop the monitoring
    Ok(())
}

#[tauri::command]
async fn get_current_metrics(state: State<'_, ServiceState>) -> Result<SystemMetrics, String> {
    let service = state.read().await;
    service.collect_metrics()
        .await
}

fn main() {
    // Initialize the monitoring service
    let service = Arc::new(RwLock::new(MonitoringService::new()));
    
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