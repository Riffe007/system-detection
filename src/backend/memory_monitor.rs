use sysinfo::{System, SystemExt};

/// Struct for Monitoring Memory Usage
pub struct MemoryMonitor;

impl MemoryMonitor {
    /// Returns total, available, and used memory in **GB**
    pub fn get_usage() -> (u64, u64, u64) {
        let mut sys = System::new_all();
        sys.refresh_memory(); // ✅ Refresh memory stats before fetching

        let total_memory_gb = sys.total_memory() / 1_073_741_824; // ✅ Convert Bytes → GB (Divide by 1_073_741_824)
        let available_memory_gb = sys.available_memory() / 1_073_741_824; // ✅ Convert Bytes → GB
        let used_memory_gb = total_memory_gb.saturating_sub(available_memory_gb); // ✅ Prevent negative values

        (total_memory_gb, available_memory_gb, used_memory_gb)
    }
}
