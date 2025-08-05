#[cfg(target_os = "linux")]
use {
    std::fs::File,
    std::os::unix::io::{AsRawFd, RawFd},
    std::ptr,
    nix::libc::{self, c_void, size_t, ssize_t},
    nix::sys::mman::{mmap, munmap, MapFlags, ProtFlags},
    nix::unistd::close,
    std::sync::atomic::{AtomicU64, Ordering},
    std::sync::Arc,
    parking_lot::Mutex,
    tracing::{info, warn, error},
};

#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct PerfEvent {
    fd: RawFd,
    mmap_addr: *mut c_void,
    mmap_size: size_t,
}

#[cfg(target_os = "linux")]
impl PerfEvent {
    pub fn new(
        event_type: u32,
        event_config: u64,
        pid: i32,
        cpu: i32,
        group_fd: i32,
        flags: u64,
    ) -> Result<Self, nix::Error> {
        let fd = unsafe {
            libc::syscall(
                libc::SYS_perf_event_open,
                &libc::perf_event_attr {
                    type_: event_type,
                    size: std::mem::size_of::<libc::perf_event_attr>() as u32,
                    config: event_config,
                    sample_period: 0,
                    sample_freq: 0,
                    sample_type: 0,
                    read_format: 0,
                    flags: 0,
                    wakeup_events: 0,
                    bp_type: 0,
                    config1: 0,
                    config2: 0,
                    branch_sample_type: 0,
                    sample_regs_user: 0,
                    sample_stack_user: 0,
                    clockid: 0,
                    sample_regs_intr: 0,
                    aux_watermark: 0,
                    sample_max_stack: 0,
                    __reserved_2: 0,
                },
                pid,
                cpu,
                group_fd,
                flags,
            ) as RawFd
        };

        if fd == -1 {
            return Err(nix::Error::last());
        }

        // Get the size of the mmap area
        let mmap_size = unsafe {
            let mut size = 0u64;
            if libc::ioctl(fd, libc::PERF_EVENT_IOC_ID, &mut size) == -1 {
                close(fd)?;
                return Err(nix::Error::last());
            }
            size as size_t
        };

        // Map the perf event buffer
        let mmap_addr = unsafe {
            mmap(
                ptr::null_mut(),
                mmap_size,
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                fd,
                0,
            )?
        };

        Ok(Self {
            fd,
            mmap_addr,
            mmap_size,
        })
    }

    pub fn enable(&self) -> Result<(), nix::Error> {
        if unsafe { libc::ioctl(self.fd, libc::PERF_EVENT_IOC_ENABLE, 0) } == -1 {
            return Err(nix::Error::last());
        }
        Ok(())
    }

    pub fn disable(&self) -> Result<(), nix::Error> {
        if unsafe { libc::ioctl(self.fd, libc::PERF_EVENT_IOC_DISABLE, 0) } == -1 {
            return Err(nix::Error::last());
        }
        Ok(())
    }

    pub fn read_value(&self) -> Result<u64, nix::Error> {
        let mut value: u64 = 0;
        let size = std::mem::size_of::<u64>();
        
        let result = unsafe {
            libc::read(
                self.fd,
                &mut value as *mut u64 as *mut c_void,
                size as size_t,
            )
        };

        if result == -1 {
            return Err(nix::Error::last());
        }

        Ok(value)
    }
}

#[cfg(target_os = "linux")]
impl Drop for PerfEvent {
    fn drop(&mut self) {
        unsafe {
            munmap(self.mmap_addr, self.mmap_size).ok();
            close(self.fd).ok();
        }
    }
}

#[cfg(target_os = "linux")]
pub struct LinuxHardwareCounters {
    cpu_cycles: Option<PerfEvent>,
    instructions: Option<PerfEvent>,
    cache_misses: Option<PerfEvent>,
    branch_misses: Option<PerfEvent>,
    page_faults: Option<PerfEvent>,
    context_switches: Option<PerfEvent>,
}

#[cfg(target_os = "linux")]
impl LinuxHardwareCounters {
    pub fn new() -> Result<Self, nix::Error> {
        let cpu_cycles = PerfEvent::new(
            libc::PERF_TYPE_HARDWARE,
            libc::PERF_COUNT_HW_CPU_CYCLES as u64,
            -1, // All processes
            0,  // CPU 0
            -1, // No group
            0,  // No flags
        ).ok();

        let instructions = PerfEvent::new(
            libc::PERF_TYPE_HARDWARE,
            libc::PERF_COUNT_HW_INSTRUCTIONS as u64,
            -1,
            0,
            -1,
            0,
        ).ok();

        let cache_misses = PerfEvent::new(
            libc::PERF_TYPE_HARDWARE,
            libc::PERF_COUNT_HW_CACHE_MISSES as u64,
            -1,
            0,
            -1,
            0,
        ).ok();

        let branch_misses = PerfEvent::new(
            libc::PERF_TYPE_HARDWARE,
            libc::PERF_COUNT_HW_BRANCH_MISSES as u64,
            -1,
            0,
            -1,
            0,
        ).ok();

        let page_faults = PerfEvent::new(
            libc::PERF_TYPE_SOFTWARE,
            libc::PERF_COUNT_SW_PAGE_FAULTS as u64,
            -1,
            0,
            -1,
            0,
        ).ok();

        let context_switches = PerfEvent::new(
            libc::PERF_TYPE_SOFTWARE,
            libc::PERF_COUNT_SW_CONTEXT_SWITCHES as u64,
            -1,
            0,
            -1,
            0,
        ).ok();

        Ok(Self {
            cpu_cycles,
            instructions,
            cache_misses,
            branch_misses,
            page_faults,
            context_switches,
        })
    }

    pub fn start(&mut self) -> Result<(), nix::Error> {
        if let Some(ref event) = self.cpu_cycles {
            event.enable()?;
        }
        if let Some(ref event) = self.instructions {
            event.enable()?;
        }
        if let Some(ref event) = self.cache_misses {
            event.enable()?;
        }
        if let Some(ref event) = self.branch_misses {
            event.enable()?;
        }
        if let Some(ref event) = self.page_faults {
            event.enable()?;
        }
        if let Some(ref event) = self.context_switches {
            event.enable()?;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), nix::Error> {
        if let Some(ref event) = self.cpu_cycles {
            event.disable()?;
        }
        if let Some(ref event) = self.instructions {
            event.disable()?;
        }
        if let Some(ref event) = self.cache_misses {
            event.disable()?;
        }
        if let Some(ref event) = self.branch_misses {
            event.disable()?;
        }
        if let Some(ref event) = self.page_faults {
            event.disable()?;
        }
        if let Some(ref event) = self.context_switches {
            event.disable()?;
        }
        Ok(())
    }

    pub fn read_values(&self) -> Result<HardwareCounterValues, nix::Error> {
        Ok(HardwareCounterValues {
            cpu_cycles: self.cpu_cycles.as_ref().and_then(|e| e.read_value().ok()).unwrap_or(0),
            instructions: self.instructions.as_ref().and_then(|e| e.read_value().ok()).unwrap_or(0),
            cache_misses: self.cache_misses.as_ref().and_then(|e| e.read_value().ok()).unwrap_or(0),
            branch_misses: self.branch_misses.as_ref().and_then(|e| e.read_value().ok()).unwrap_or(0),
            page_faults: self.page_faults.as_ref().and_then(|e| e.read_value().ok()).unwrap_or(0),
            context_switches: self.context_switches.as_ref().and_then(|e| e.read_value().ok()).unwrap_or(0),
        })
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct HardwareCounterValues {
    pub cpu_cycles: u64,
    pub instructions: u64,
    pub cache_misses: u64,
    pub branch_misses: u64,
    pub page_faults: u64,
    pub context_switches: u64,
}

#[cfg(not(target_os = "linux"))]
pub struct LinuxHardwareCounters;

#[cfg(not(target_os = "linux"))]
impl LinuxHardwareCounters {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Err("Linux hardware counters not available on this platform".into())
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn read_values(&self) -> Result<HardwareCounterValues, Box<dyn std::error::Error>> {
        Ok(HardwareCounterValues {
            cpu_cycles: 0,
            instructions: 0,
            cache_misses: 0,
            branch_misses: 0,
            page_faults: 0,
            context_switches: 0,
        })
    }
}

#[cfg(not(target_os = "linux"))]
#[derive(Debug, Clone)]
pub struct HardwareCounterValues {
    pub cpu_cycles: u64,
    pub instructions: u64,
    pub cache_misses: u64,
    pub branch_misses: u64,
    pub page_faults: u64,
    pub context_switches: u64,
} 