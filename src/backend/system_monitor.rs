use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use raw_cpuid::CpuId;
use serde_json::json;
use serde_json;
use std::fs;

pub struct SystemSpecs {
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub has_avx: bool,
    pub has_avx2: bool,
    pub has_avx512: bool,
    pub total_memory: u64, // in MB
    pub available_memory: u64,
    pub disk_space: u64, // in GB
    pub gpu_name: Option<String>,
    pub cuda_supported: bool,
}

impl SystemSpecs {
    pub fn detect() -> Self {
        let sys = System::new_all();
        let cpu = sys.cpus().get(0).unwrap();
        let cpuid = CpuId::new();

        let has_avx = cpuid.get_feature_info().map_or(false, |f| f.has_avx());
        let has_avx2 = cpuid.get_extended_feature_info().map_or(false, |f| f.has_avx2());
        let has_avx512 = cpuid.get_extended_feature_info().map_or(false, |f| f.has_avx512f());

        let total_memory = sys.total_memory() / 1024; // Convert to MB
        let available_memory = sys.available_memory() / 1024;

        let disk = sys.disks().get(0).unwrap();
        let disk_space = disk.total_space() / (1024 * 1024 * 1024); // Convert to GB

        let gpu_info = match nvml_wrapper::Nvml::init() {
            Ok(nvml) => {
                if let Ok(device) = nvml.device_by_index(0) {
                    Some(device.name().unwrap_or("Unknown GPU".to_string()))
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        let cuda_supported = gpu_info.is_some();

        Self {
            cpu_name: cpu.brand().to_string(),
            cpu_cores: sys.physical_core_count().unwrap_or(1),
            cpu_threads: sys.cpus().len(),
            has_avx,
            has_avx2,
            has_avx512,
            total_memory,
            available_memory,
            disk_space,
            gpu_name: gpu_info,
            cuda_supported,
        }
    }

    pub fn recommend_config(&self) -> serde_json::Value {
        let model_format = if self.has_avx512 {
            "AWQ (High Accuracy)"
        } else if self.has_avx2 {
            "GPTQ 8-bit (Balanced)"
        } else {
            "GGUF 4-bit (Efficiency)"
        };

        let vector_db = if self.total_memory < 8000 {
            "SQLite (Lightweight)"
        } else if self.total_memory < 16000 {
            "DuckDB (Mid-range)"
        } else {
            "FAISS (High Performance)"
        };

        json!({
            "recommended_model_format": model_format,
            "recommended_vector_database": vector_db,
            "recommended_local_cache": if self.available_memory > 8000 { "Advanced Prefetch Cache" } else { "Simple LRU Cache" },
            "cuda_supported": self.cuda_supported,
            "recommended_execution": if self.cuda_supported { "GPU Acceleration" } else { "Optimized CPU Mode" }
        })
    }

    pub fn save_config(&self) {
        let recommendations = self.recommend_config();
        fs::write("config.json", serde_json::to_string_pretty(&recommendations).unwrap())
            .expect("Failed to save configuration");
    }
}
