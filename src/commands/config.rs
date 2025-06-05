use tauri::State;
use crate::core::{AppConfig, ConfigManager};
use crate::AppState;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config_manager = state.config_manager.lock().await;
    Ok(config_manager.config().clone())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut config_manager = state.config_manager.lock().await;
    
    // Validate the new configuration
    *config_manager.config_mut() = config;
    config_manager.validate().map_err(|e| e.to_string())?;
    
    // Save to disk
    config_manager.save().map_err(|e| e.to_string())?;
    
    // Apply the new configuration to the monitoring service
    let monitoring_service = state.monitoring_service.lock().await;
    monitoring_service.apply_config(config_manager.config()).await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_config_value(
    state: State<'_, AppState>,
    path: String,
) -> Result<Value, String> {
    let config_manager = state.config_manager.lock().await;
    let config = config_manager.config();
    
    // Convert config to JSON for easy path navigation
    let config_json = serde_json::to_value(config)
        .map_err(|e| e.to_string())?;
    
    // Navigate the path (e.g., "monitoring.cpu.interval_ms")
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = &config_json;
    
    for part in parts {
        match current.get(part) {
            Some(value) => current = value,
            None => return Err(format!("Config path '{}' not found", path)),
        }
    }
    
    Ok(current.clone())
}

#[tauri::command]
pub async fn set_config_value(
    state: State<'_, AppState>,
    path: String,
    value: Value,
) -> Result<(), String> {
    let mut config_manager = state.config_manager.lock().await;
    
    // Get current config as mutable JSON
    let mut config_json = serde_json::to_value(config_manager.config())
        .map_err(|e| e.to_string())?;
    
    // Navigate to the parent of the target path
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return Err("Invalid config path".to_string());
    }
    
    let mut current = &mut config_json;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Set the value at the final key
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), value);
            } else {
                return Err(format!("Parent of '{}' is not an object", path));
            }
        } else {
            // Navigate deeper
            match current.get_mut(part) {
                Some(next) => current = next,
                None => return Err(format!("Config path '{}' not found", path)),
            }
        }
    }
    
    // Convert back to AppConfig
    let new_config: AppConfig = serde_json::from_value(config_json)
        .map_err(|e| format!("Invalid configuration: {}", e))?;
    
    // Update and save
    *config_manager.config_mut() = new_config;
    config_manager.validate().map_err(|e| e.to_string())?;
    config_manager.save().map_err(|e| e.to_string())?;
    
    // Apply the new configuration
    let monitoring_service = state.monitoring_service.lock().await;
    monitoring_service.apply_config(config_manager.config()).await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn reset_config(state: State<'_, AppState>) -> Result<(), String> {
    let mut config_manager = state.config_manager.lock().await;
    
    // Reset to default configuration
    *config_manager.config_mut() = AppConfig::default();
    
    // Save to disk
    config_manager.save().map_err(|e| e.to_string())?;
    
    // Apply the default configuration
    let monitoring_service = state.monitoring_service.lock().await;
    monitoring_service.apply_config(config_manager.config()).await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn validate_config(config: AppConfig) -> Result<(), String> {
    // Create a temporary config manager for validation
    let temp_manager = ConfigManager {
        config_path: std::path::PathBuf::from("temp"),
        config,
    };
    
    temp_manager.validate().map_err(|e| e.to_string())
}