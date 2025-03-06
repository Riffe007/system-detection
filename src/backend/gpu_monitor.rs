use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::device::physical::PhysicalDevice;
use vulkano::VulkanLibrary;
use std::sync::Arc;
use log::info;
use std::collections::HashMap;

/// Struct for GPU Monitoring
pub struct GpuMonitor;

impl GpuMonitor {
    /// Detects GPUs and their specifications
    pub fn detect() -> Option<HashMap<String, String>> {
        // Initialize Vulkan library
        let library = VulkanLibrary::new().ok()?;
        
        // Create Vulkan instance
        let instance = Instance::new(library, InstanceCreateInfo::default()).ok()?;
        
        // Enumerate physical devices (GPUs)
        let devices: Vec<Arc<PhysicalDevice>> = instance.enumerate_physical_devices().ok()?.collect();

        // If no GPUs are found, return None
        if devices.is_empty() {
            return None;
        }

        let mut gpu_info = HashMap::new();
        for device in devices {
            let device_name = device.properties().device_name.clone();
            let memory = device.memory_properties().memory_heaps.iter()
                .map(|m| m.size) // Get size of each memory heap
                .sum::<u64>() / 1024 / 1024 / 1024; // Convert to GB
            
            let api_version = device.api_version();

            gpu_info.insert("Name".to_string(), device_name);
            gpu_info.insert("Memory".to_string(), format!("{} GB", memory));
            gpu_info.insert(
                "API Version".to_string(), 
                format!("{}.{}.{}", api_version.major, api_version.minor, api_version.patch) // ✅ FIXED
            );
        }

        info!("Detected GPU: {:?}", gpu_info);
        Some(gpu_info)
    }
}
