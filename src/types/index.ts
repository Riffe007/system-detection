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
  boot_time: number;
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
  // Specialized hardware accelerators (only populated if detected)
  dpus: DpuMetrics[];
  npus: NpuMetrics[];
  external_ddr: ExternalDdrMetrics[];
  fpgas: FpgaMetrics[];
  asics: AsicMetrics[];
  quantum_processors: QuantumProcessorMetrics[];
}

// High-Performance Metrics Types (for ultra-low-latency monitoring)
export interface HighPerfCpuMetrics {
  global_usage: number;
  per_core_usage: number[];
  frequency_mhz: number[];
  temperature?: number;
  load_average: [number, number, number];
  context_switches: number;
  interrupts: number;
  cache_misses: number;
  cache_hits: number;
}

export interface HighPerfMemoryMetrics {
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  cached_bytes: number;
  buffer_bytes: number;
  swap_total_bytes: number;
  swap_used_bytes: number;
  page_faults: number;
  page_ins: number;
  page_outs: number;
}

export interface HighPerfGpuMetrics {
  name: string;
  usage_percent: number;
  memory_used_bytes: number;
  memory_total_bytes: number;
  temperature_celsius: number;
  power_watts: number;
  fan_speed_percent?: number;
  clock_mhz: number;
  memory_clock_mhz: number;
}

export interface HighPerfDiskMetrics {
  device_name: string;
  mount_point: string;
  total_bytes: number;
  used_bytes: number;
  read_bytes_per_sec: number;
  write_bytes_per_sec: number;
  io_operations_per_sec: number;
  read_latency_ms: number;
  write_latency_ms: number;
}

export interface HighPerfNetworkMetrics {
  interface_name: string;
  bytes_sent_per_sec: number;
  bytes_received_per_sec: number;
  packets_sent_per_sec: number;
  packets_received_per_sec: number;
  errors_per_sec: number;
  latency_ms: number;
}

export interface HighPerfProcessMetrics {
  pid: number;
  name: string;
  cpu_usage_percent: number;
  memory_bytes: number;
  memory_percent: number;
  disk_read_bytes_per_sec: number;
  disk_write_bytes_per_sec: number;
  network_bytes_per_sec: number;
  threads: number;
  priority: number;
}

// Specialized hardware accelerator types
export interface DpuMetrics {
  name: string;
  vendor: string;
  model: string;
  usage_percent: number;
  memory_used_bytes: number;
  memory_total_bytes: number;
  temperature_celsius: number;
  power_watts: number;
  clock_mhz: number;
  throughput_gbps: number;
  packet_processing_rate: number;
  active_flows: number;
  driver_version: string;
}

export interface NpuMetrics {
  name: string;
  vendor: string;
  model: string;
  usage_percent: number;
  memory_used_bytes: number;
  memory_total_bytes: number;
  temperature_celsius: number;
  power_watts: number;
  clock_mhz: number;
  inference_rate: number;
  model_accuracy: number;
  active_models: number;
  driver_version: string;
}

export interface ExternalDdrMetrics {
  name: string;
  vendor: string;
  capacity_bytes: number;
  used_bytes: number;
  bandwidth_gbps: number;
  latency_ns: number;
  temperature_celsius: number;
  power_watts: number;
  error_rate: number;
  refresh_rate_hz: number;
}

export interface FpgaMetrics {
  name: string;
  vendor: string;
  model: string;
  usage_percent: number;
  temperature_celsius: number;
  power_watts: number;
  clock_mhz: number;
  logic_utilization: number;
  memory_utilization: number;
  dsp_utilization: number;
  bitstream_version: string;
}

export interface AsicMetrics {
  name: string;
  vendor: string;
  model: string;
  usage_percent: number;
  temperature_celsius: number;
  power_watts: number;
  clock_mhz: number;
  throughput_gbps: number;
  packet_processing_rate: number;
  active_channels: number;
}

export interface QuantumProcessorMetrics {
  name: string;
  vendor: string;
  qubits: number;
  coherence_time_ms: number;
  gate_fidelity: number;
  temperature_mk: number; // millikelvin
  power_watts: number;
  active_qubits: number;
  error_rate: number;
}

export interface HighPerfMetrics {
  timestamp_nanos: number;
  cpu: HighPerfCpuMetrics;
  memory: HighPerfMemoryMetrics;
  gpus: HighPerfGpuMetrics[];
  disks: HighPerfDiskMetrics[];
  networks: HighPerfNetworkMetrics[];
  processes: HighPerfProcessMetrics[];
  // Specialized hardware accelerators (only populated if detected)
  dpus: DpuMetrics[];
  npus: NpuMetrics[];
  external_ddr: ExternalDdrMetrics[];
  fpgas: FpgaMetrics[];
  asics: AsicMetrics[];
  quantum_processors: QuantumProcessorMetrics[];
}