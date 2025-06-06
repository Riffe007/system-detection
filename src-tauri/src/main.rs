#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod monitoring;

use std::sync::Arc;
use tauri::{Manager, State, Emitter};
use tokio::sync::RwLock;
use monitoring::{MonitoringService, SystemInfo, SystemMetrics};

type ServiceState = Arc<RwLock<MonitoringService>>;

#[tauri::command]
async fn get_system_info(state: State<'_, ServiceState>) -> Result<SystemInfo, String> {
    let service = state.read().await;
    service.get_system_info()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_monitoring(state: State<'_, ServiceState>, app: tauri::AppHandle) -> Result<(), String> {
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
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            start_monitoring,
            stop_monitoring,
            get_current_metrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}