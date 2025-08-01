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
}

// Security Types
export interface SecurityEvent {
  timestamp: string;
  event_type: SecurityEventType;
  severity: SecuritySeverity;
  description: string;
  details: Record<string, string>;
}

export type SecurityEventType = 
  | 'SuspiciousProcess'
  | 'NetworkConnection'
  | 'FileSystemChange'
  | 'HighCpuUsage'
  | 'HighMemoryUsage'
  | 'UnusualNetworkActivity'
  | 'ProcessCreation'
  | 'ProcessTermination'
  | 'ProcessQuarantined'
  | 'ProcessTerminated'
  | 'NetworkBlocked'
  | 'FirewallRuleAdded';

export type SecuritySeverity = 'Low' | 'Medium' | 'High' | 'Critical';

export interface ProcessSecurityInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_usage: number;
  suspicious_score: number;
  network_connections: string[];
  file_handles: number;
  start_time: string;
  parent_pid?: number;
}

export interface NetworkSecurityInfo {
  interface: string;
  local_address: string;
  remote_address: string;
  port: number;
  protocol: string;
  process_name: string;
  process_pid: number;
  connection_type: string;
  bytes_sent: number;
  bytes_received: number;
}

export interface QuarantinedProcess {
  pid: number;
  name: string;
  original_path: string;
  quarantine_path: string;
  quarantine_time: string;
  reason: string;
  status: QuarantineStatus;
}

export type QuarantineStatus = 'Quarantined' | 'Restored' | 'Deleted' | 'PendingAnalysis';

export interface BlockedConnection {
  remote_address: string;
  port: number;
  protocol: string;
  block_time: string;
  reason: string;
  firewall_rule_name: string;
}

export interface SecurityMetrics {
  timestamp: string;
  total_processes: number;
  suspicious_processes: ProcessSecurityInfo[];
  network_connections: NetworkSecurityInfo[];
  recent_events: SecurityEvent[];
  security_score: number;
  threats_detected: number;
  recommendations: string[];
  quarantined_processes: QuarantinedProcess[];
  blocked_connections: BlockedConnection[];
}

// Optimization Types
export interface PerformanceBottleneck {
  category: BottleneckCategory;
  severity: BottleneckSeverity;
  description: string;
  current_value: number;
  threshold: number;
  recommendation: string;
  impact_score: number;
}

export type BottleneckCategory = 'CPU' | 'Memory' | 'Disk' | 'Network' | 'Process' | 'System';
export type BottleneckSeverity = 'Low' | 'Medium' | 'High' | 'Critical';

export interface OptimizationRecommendation {
  title: string;
  description: string;
  category: string;
  priority: number;
  estimated_impact: string;
  implementation_difficulty: string;
  actions: string[];
}

export interface ResourceUsage {
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_usage: number;
  io_wait: number;
  load_average: [number, number, number];
}

export interface ProcessPerformance {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_usage: number;
  memory_percent: number;
  priority: string;
  threads: number;
  io_read_bytes: number;
  io_write_bytes: number;
  performance_score: number;
}

export interface SystemHealth {
  cpu_health: number;
  memory_health: number;
  disk_health: number;
  network_health: number;
  overall_health: number;
}

export interface OptimizationMetrics {
  timestamp: string;
  overall_performance_score: number;
  resource_usage: ResourceUsage;
  bottlenecks: PerformanceBottleneck[];
  top_performance_processes: ProcessPerformance[];
  recommendations: OptimizationRecommendation[];
  system_health: SystemHealth;
}