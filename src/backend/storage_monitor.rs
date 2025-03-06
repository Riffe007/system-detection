use sysinfo::{System, SystemExt, DiskExt};

/// Struct for Monitoring Storage Usage
pub struct StorageMonitor;

impl StorageMonitor {
    /// Returns total and available storage in GB
    pub fn get_usage() -> (u64, u64) {
        let sys = System::new_all();
        let total_storage = sys.disks().iter().map(|disk| disk.total_space()).sum::<u64>() / 1024 / 1024 / 1024;
        let available_storage = sys.disks().iter().map(|disk| disk.available_space()).sum::<u64>() / 1024 / 1024 / 1024;
        (total_storage, available_storage)
    }
}
