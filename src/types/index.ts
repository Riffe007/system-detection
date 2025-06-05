export interface SystemInfo {
  hostname: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  architecture: string;
  cpu_brand: string;
  cpu_cores: number;
  cpu_threads: number;
  total_memory: number;
  boot_time: string;
}

export interface CpuMetrics {
  usage_percent: number;
  frequency_mhz: number;
  temperature_celsius?: number;
  load_average: [number, number, number];
  per_core_usage: number[];
  processes_running: number;
  processes_total: number;
  context_switches: number;
  interrupts: number;
}

export interface MemoryMetrics {
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  cached_bytes: number;
  swap_total_bytes: number;
  swap_used_bytes: number;
  usage_percent: number;
  swap_usage_percent: number;
}

export interface GpuMetrics {
  name: string;
  driver_version: string;
  temperature_celsius: number;
  usage_percent: number;
  memory_total_bytes: number;
  memory_used_bytes: number;
  memory_usage_percent: number;
  power_watts: number;
  fan_speed_percent?: number;
  clock_mhz: number;
  memory_clock_mhz: number;
}

export interface DiskMetrics {
  mount_point: string;
  device_name: string;
  fs_type: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  usage_percent: number;
  read_bytes_per_sec: number;
  write_bytes_per_sec: number;
  io_operations_per_sec: number;
}

export interface NetworkMetrics {
  interface_name: string;
  is_up: boolean;
  mac_address: string;
  ip_addresses: string[];
  bytes_sent: number;
  bytes_received: number;
  packets_sent: number;
  packets_received: number;
  errors_sent: number;
  errors_received: number;
  speed_mbps?: number;
  bytes_sent_rate: number;
  bytes_received_rate: number;
}

export interface ProcessMetrics {
  pid: number;
  name: string;
  cpu_usage_percent: number;
  memory_bytes: number;
  memory_percent: number;
  disk_read_bytes: number;
  disk_write_bytes: number;
  status: string;
  threads: number;
  start_time: string;
}

export interface SystemMetrics {
  timestamp: string;
  system_info: SystemInfo;
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  gpus: GpuMetrics[];
  disks: DiskMetrics[];
  networks: NetworkMetrics[];
  top_processes: ProcessMetrics[];
}