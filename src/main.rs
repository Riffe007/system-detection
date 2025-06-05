#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod backend;
mod core;
mod services;
mod commands;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

use crate::services::MonitoringService;
use crate::core::SystemMetrics;

pub struct AppState {
    monitoring_service: Arc<Mutex<MonitoringService>>,
}

#[tauri::command]
async fn get_system_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let service = state.monitoring_service.lock().await;
    
    match service.get_system_info().await {
        Some(info) => Ok(serde_json::to_value(info).map_err(|e| e.to_string())?),
        None => Err("System info not available".to_string()),
    }
}

#[tauri::command]
async fn start_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    let service = state.monitoring_service.lock().await;
    service.start().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    let service = state.monitoring_service.lock().await;
    service.stop().await.map_err(|e| e.to_string())
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("system_monitor=debug,tauri=info")
        .init();

    let monitoring_service = Arc::new(Mutex::new(MonitoringService::new()));
    let service_clone = monitoring_service.clone();

    tauri::Builder::default()
        .manage(AppState {
            monitoring_service,
        })
        .setup(move |app| {
            let service = service_clone.clone();
            let app_handle = app.handle();
            
            // Initialize monitoring service
            tauri::async_runtime::spawn(async move {
                let service = service.lock().await;
                if let Err(e) = service.initialize().await {
                    tracing::error!("Failed to initialize monitoring service: {}", e);
                    return;
                }

                // Subscribe to metrics and forward to frontend
                let mut receiver = service.subscribe();
                
                loop {
                    match receiver.recv().await {
                        Ok(metrics) => {
                            if let Err(e) = app_handle.emit_all("system-metrics", &metrics) {
                                tracing::error!("Failed to emit metrics: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to receive metrics: {}", e);
                            break;
                        }
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            start_monitoring,
            stop_monitoring,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
