use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use crossbeam::channel::{bounded, Sender, Receiver};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::{info, warn, error};

#[derive(Error, Debug)]
pub enum KernelMonitorError {
    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("System call failed: {0}")]
    SystemCallFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelMetrics {
    pub timestamp: u64,  // Nanosecond precision
    pub cpu: KernelCpuMetrics,
    pub memory: KernelMemoryMetrics,
    pub disk: KernelDiskMetrics,
    pub network: KernelNetworkMetrics,
    pub latency: KernelLatencyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelCpuMetrics {
    pub cycles: u64,
    pub instructions: u64,
    pub cache_misses: u64,
    pub branch_misses: u64,
    pub cpu_usage_percent: f64,
    pub frequency_mhz: u64,
    pub temperature_celsius: Option<f64>,
    pub power_watts: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelMemoryMetrics {
    pub page_faults: u64,
    pub page_ins: u64,
    pub page_outs: u64,
    pub swap_ins: u64,
    pub swap_outs: u64,
    pub memory_pressure: f64,
    pub numa_hits: u64,
    pub numa_misses: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelDiskMetrics {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_ops: u64,
    pub write_ops: u64,
    pub io_wait_time: u64,
    pub queue_depth: u32,
    pub latency_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelNetworkMetrics {
    pub packets_in: u64,
    pub packets_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub errors_in: u64,
    pub errors_out: u64,
    pub drops_in: u64,
    pub drops_out: u64,
    pub latency_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelLatencyMetrics {
    pub collection_latency_ns: u64,
    pub processing_latency_ns: u64,
    pub total_latency_ns: u64,
}

pub struct KernelMonitor {
    sender: Sender<KernelMetrics>,
    receiver: Receiver<KernelMetrics>,
    running: Arc<Mutex<bool>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl KernelMonitor {
    pub fn new() -> Result<Self, KernelMonitorError> {
        let (sender, receiver) = bounded(1000); // High-capacity channel
        
        Ok(Self {
            sender,
            receiver,
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
        })
    }

    pub fn start(&mut self) -> Result<(), KernelMonitorError> {
        if *self.running.lock() {
            return Ok(());
        }

        *self.running.lock() = true;
        let sender = self.sender.clone();
        let running = self.running.clone();

        let handle = std::thread::spawn(move || {
            Self::monitoring_loop(sender, running);
        });

        self.thread_handle = Some(handle);
        info!("Kernel-level monitoring started");
        Ok(())
    }

    pub fn stop(&mut self) {
        *self.running.lock() = false;
        
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Failed to join kernel monitoring thread: {:?}", e);
            }
        }
        
        info!("Kernel-level monitoring stopped");
    }

    pub fn get_latest_metrics(&self) -> Option<KernelMetrics> {
        self.receiver.try_recv().ok()
    }

    fn monitoring_loop(sender: Sender<KernelMetrics>, running: Arc<Mutex<bool>>) {
        while *running.lock() {
            let start = Instant::now();
            
            // Collect kernel-level metrics with nanosecond precision
            match Self::collect_kernel_metrics() {
                Ok(mut metrics) => {
                    let collection_latency = start.elapsed().as_nanos() as u64;
                    let processing_start = Instant::now();
                    
                    // Calculate latencies
                    metrics.latency.collection_latency_ns = collection_latency;
                    metrics.latency.processing_latency_ns = processing_start.elapsed().as_nanos() as u64;
                    metrics.latency.total_latency_ns = start.elapsed().as_nanos() as u64;
                    
                    // Send metrics (non-blocking)
                    if let Err(e) = sender.try_send(metrics) {
                        warn!("Failed to send kernel metrics: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to collect kernel metrics: {}", e);
                }
            }
            
                         // Sleep for 3 seconds between collections
            let elapsed = start.elapsed();
                         if elapsed < Duration::from_secs(3) {
                 std::thread::sleep(Duration::from_secs(3) - elapsed);
            }
        }
    }

    fn collect_kernel_metrics() -> Result<KernelMetrics, KernelMonitorError> {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                Self::collect_linux_kernel_metrics()
            } else if #[cfg(target_os = "windows")] {
                Self::collect_windows_kernel_metrics()
            } else {
                Err(KernelMonitorError::UnsupportedPlatform(
                    std::env::consts::OS.to_string()
                ))
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn collect_linux_kernel_metrics() -> Result<KernelMetrics, KernelMonitorError> {
        // TODO: Implement Linux eBPF-based collection
        // This will use perf_event_open for hardware counters
        // and eBPF programs for kernel-level data
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Ok(KernelMetrics {
            timestamp,
            cpu: KernelCpuMetrics {
                cycles: 0,
                instructions: 0,
                cache_misses: 0,
                branch_misses: 0,
                cpu_usage_percent: 0.0,
                frequency_mhz: 0,
                temperature_celsius: None,
                power_watts: None,
            },
            memory: KernelMemoryMetrics {
                page_faults: 0,
                page_ins: 0,
                page_outs: 0,
                swap_ins: 0,
                swap_outs: 0,
                memory_pressure: 0.0,
                numa_hits: 0,
                numa_misses: 0,
            },
            disk: KernelDiskMetrics {
                read_bytes: 0,
                write_bytes: 0,
                read_ops: 0,
                write_ops: 0,
                io_wait_time: 0,
                queue_depth: 0,
                latency_ns: 0,
            },
            network: KernelNetworkMetrics {
                packets_in: 0,
                packets_out: 0,
                bytes_in: 0,
                bytes_out: 0,
                errors_in: 0,
                errors_out: 0,
                drops_in: 0,
                drops_out: 0,
                latency_ns: 0,
            },
            latency: KernelLatencyMetrics {
                collection_latency_ns: 0,
                processing_latency_ns: 0,
                total_latency_ns: 0,
            },
        })
    }

    #[cfg(target_os = "windows")]
    fn collect_windows_kernel_metrics() -> Result<KernelMetrics, KernelMonitorError> {
        // TODO: Implement Windows ETW-based collection
        // This will use Event Tracing for Windows for kernel-level data
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Ok(KernelMetrics {
            timestamp,
            cpu: KernelCpuMetrics {
                cycles: 0,
                instructions: 0,
                cache_misses: 0,
                branch_misses: 0,
                cpu_usage_percent: 0.0,
                frequency_mhz: 0,
                temperature_celsius: None,
                power_watts: None,
            },
            memory: KernelMemoryMetrics {
                page_faults: 0,
                page_ins: 0,
                page_outs: 0,
                swap_ins: 0,
                swap_outs: 0,
                memory_pressure: 0.0,
                numa_hits: 0,
                numa_misses: 0,
            },
            disk: KernelDiskMetrics {
                read_bytes: 0,
                write_bytes: 0,
                read_ops: 0,
                write_ops: 0,
                io_wait_time: 0,
                queue_depth: 0,
                latency_ns: 0,
            },
            network: KernelNetworkMetrics {
                packets_in: 0,
                packets_out: 0,
                bytes_in: 0,
                bytes_out: 0,
                errors_in: 0,
                errors_out: 0,
                drops_in: 0,
                drops_out: 0,
                latency_ns: 0,
            },
            latency: KernelLatencyMetrics {
                collection_latency_ns: 0,
                processing_latency_ns: 0,
                total_latency_ns: 0,
            },
        })
    }
}

impl Drop for KernelMonitor {
    fn drop(&mut self) {
        self.stop();
    }
} 