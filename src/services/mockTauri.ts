// Mock Tauri API for development without Tauri runtime
import { SystemInfo, SystemMetrics } from '../types';

const mockSystemInfo: SystemInfo = {
  hostname: 'dev-machine',
  os_name: 'Linux',
  os_version: '5.15.0',
  kernel_version: '5.15.0-generic',
  architecture: 'x86_64',
  cpu_brand: 'Intel Core i7-9700K',
  cpu_cores: 8,
  cpu_threads: 8,
  total_memory: 16 * 1024 * 1024 * 1024, // 16GB
  boot_time: Math.floor((Date.now() - 3600000) / 1000), // 1 hour ago in seconds
};

let metricsInterval: NodeJS.Timeout | null = null;
let metricsCallback: ((metrics: SystemMetrics) => void) | null = null;

const generateMockMetrics = (): SystemMetrics => {
  const baseUsage = 30;
  const variation = Math.sin(Date.now() / 5000) * 20;
  
  return {
    timestamp: new Date().toISOString(),
    system_info: mockSystemInfo,
    cpu: {
      usage_percent: Math.max(0, Math.min(100, baseUsage + variation + Math.random() * 10)),
      frequency_mhz: 3600,
      per_core_usage: Array(8).fill(0).map(() => 
        Math.max(0, Math.min(100, baseUsage + Math.random() * 30))
      ),
      temperature_celsius: 45 + Math.random() * 15,
      load_average: [1.2, 0.8, 0.9],
      context_switches: 100000 + Math.floor(Math.random() * 50000),
      interrupts: 50000 + Math.floor(Math.random() * 10000),
      processes_total: 250 + Math.floor(Math.random() * 50),
      processes_running: 2 + Math.floor(Math.random() * 5),
    },
    memory: {
      total_bytes: 16 * 1024 * 1024 * 1024,
      used_bytes: 8 * 1024 * 1024 * 1024 + Math.random() * 2 * 1024 * 1024 * 1024,
      available_bytes: 8 * 1024 * 1024 * 1024 - Math.random() * 2 * 1024 * 1024 * 1024,
      cached_bytes: 2 * 1024 * 1024 * 1024,
      swap_total_bytes: 8 * 1024 * 1024 * 1024,
      swap_used_bytes: Math.random() * 1024 * 1024 * 1024,
      usage_percent: 50 + variation / 2 + Math.random() * 10,
      swap_usage_percent: Math.random() * 20,
    },
    gpus: [],
    disks: [
      {
        mount_point: '/',
        device_name: '/dev/sda1',
        fs_type: 'ext4',
        total_bytes: 512 * 1024 * 1024 * 1024,
        used_bytes: 256 * 1024 * 1024 * 1024,
        available_bytes: 256 * 1024 * 1024 * 1024,
        usage_percent: 50,
        read_bytes_per_sec: Math.random() * 10 * 1024 * 1024,
        write_bytes_per_sec: Math.random() * 5 * 1024 * 1024,
        io_operations_per_sec: Math.floor(Math.random() * 1000),
      },
      {
        mount_point: '/home',
        device_name: '/dev/sda2',
        fs_type: 'ext4',
        total_bytes: 1024 * 1024 * 1024 * 1024,
        used_bytes: 600 * 1024 * 1024 * 1024,
        available_bytes: 424 * 1024 * 1024 * 1024,
        usage_percent: 58.6,
        read_bytes_per_sec: Math.random() * 5 * 1024 * 1024,
        write_bytes_per_sec: Math.random() * 3 * 1024 * 1024,
        io_operations_per_sec: Math.floor(Math.random() * 500),
      },
    ],
    networks: [
      {
        interface_name: 'eth0',
        mac_address: '00:11:22:33:44:55',
        bytes_sent: Math.floor(Math.random() * 1024 * 1024 * 1024),
        bytes_received: Math.floor(Math.random() * 2 * 1024 * 1024 * 1024),
        packets_sent: Math.floor(Math.random() * 1000000),
        packets_received: Math.floor(Math.random() * 2000000),
        bytes_sent_rate: Math.random() * 1024 * 1024,
        bytes_received_rate: Math.random() * 2 * 1024 * 1024,
        is_up: true,
        ip_addresses: ['192.168.1.100'],
        errors_sent: 0,
        errors_received: 0,
      },
    ],
    top_processes: [
      {
        pid: 1234,
        name: 'firefox',
        cpu_usage_percent: 15 + Math.random() * 10,
        memory_bytes: 1.5 * 1024 * 1024 * 1024,
        memory_percent: 9.5,
        disk_read_bytes: 0,
        disk_write_bytes: 0,
        status: 'Running',
        threads: 120,
        start_time: new Date(Date.now() - 7200000).toISOString(),
      },
      {
        pid: 5678,
        name: 'code',
        cpu_usage_percent: 10 + Math.random() * 5,
        memory_bytes: 800 * 1024 * 1024,
        memory_percent: 5.0,
        disk_read_bytes: 0,
        disk_write_bytes: 0,
        status: 'Running',
        threads: 45,
        start_time: new Date(Date.now() - 3600000).toISOString(),
      },
      {
        pid: 9012,
        name: 'chrome',
        cpu_usage_percent: 8 + Math.random() * 5,
        memory_bytes: 1.2 * 1024 * 1024 * 1024,
        memory_percent: 7.5,
        disk_read_bytes: 0,
        disk_write_bytes: 0,
        status: 'Running',
        threads: 89,
        start_time: new Date(Date.now() - 1800000).toISOString(),
      },
      {
        pid: 3456,
        name: 'node',
        cpu_usage_percent: 5 + Math.random() * 3,
        memory_bytes: 256 * 1024 * 1024,
        memory_percent: 1.6,
        disk_read_bytes: 0,
        disk_write_bytes: 0,
        status: 'Running',
        threads: 12,
        start_time: new Date(Date.now() - 600000).toISOString(),
      },
      {
        pid: 7890,
        name: 'systemd',
        cpu_usage_percent: 2 + Math.random() * 2,
        memory_bytes: 128 * 1024 * 1024,
        memory_percent: 0.8,
        disk_read_bytes: 0,
        disk_write_bytes: 0,
        status: 'Sleeping',
        threads: 1,
        start_time: new Date(Date.now() - 86400000).toISOString(),
      },
    ],
  };
};

export const mockTauri = {
  invoke: async (cmd: string, args?: any): Promise<any> => {
    console.log(`Mock Tauri invoke: ${cmd}`, args);
    
    switch (cmd) {
      case 'get_system_info':
        return mockSystemInfo;
      
      case 'start_monitoring':
        if (metricsInterval) {
          clearInterval(metricsInterval);
        }
        metricsInterval = setInterval(() => {
          if (metricsCallback) {
            metricsCallback(generateMockMetrics());
          }
        }, 1000);
        return;
      
      case 'stop_monitoring':
        if (metricsInterval) {
          clearInterval(metricsInterval);
          metricsInterval = null;
        }
        return;
      
      case 'get_current_metrics':
        return generateMockMetrics();
      
      default:
        console.warn(`Unknown Tauri command: ${cmd}`);
        return null;
    }
  },
};

export const mockListen = (event: string, callback: (event: { payload: any }) => void) => {
  console.log(`Mock Tauri listen: ${event}`);
  
  if (event === 'system-metrics') {
    metricsCallback = (metrics) => callback({ payload: metrics });
  }
  
  return Promise.resolve(() => {
    if (event === 'system-metrics') {
      metricsCallback = null;
    }
  });
};