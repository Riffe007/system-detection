import { useEffect, useState } from 'react';
import { Dashboard } from './components/Dashboard';
import { Header } from './components/Header';
import { ThemeProvider } from './contexts/ThemeContext';
import { SystemInfo, SystemMetrics } from './types';
import './App.css';

declare global {
  interface Window {
    __TAURI__?: any;
  }
}

export default function AppWrapper() {
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const init = async () => {
      try {
        // Check if we're in Tauri environment
        if (window.__TAURI__) {
          const { invoke } = window.__TAURI__.tauri;
          const { listen } = window.__TAURI__.event;
          
          // Load system info
          try {
            const info = await invoke('get_system_info');
            setSystemInfo(info);
          } catch (err) {
            console.error('Failed to load system info:', err);
            setError(`Failed to load system info: ${err}`);
          }
          
          // Start monitoring
          try {
            await invoke('start_monitoring');
            setIsMonitoring(true);
          } catch (err) {
            console.error('Failed to start monitoring:', err);
            setError(`Failed to start monitoring: ${err}`);
          }
          
          // Listen for metrics updates
          const unlisten = await listen('system-metrics', (event: any) => {
            setMetrics(event.payload);
          });
          
          // Cleanup
          return () => {
            unlisten();
            invoke('stop_monitoring').catch(console.error);
          };
        } else {
          // Running in browser - use mock data
          setSystemInfo({
            hostname: 'localhost',
            os_name: 'Linux',
            os_version: '6.11.0',
            kernel_version: '6.11.0-26-generic',
            architecture: 'x86_64',
            cpu_brand: 'Intel Core i7',
            cpu_cores: 8,
            cpu_threads: 16,
            total_memory: 16777216000,
            boot_time: new Date(Date.now() - 86400000).toISOString()
          });
          
          // Mock metrics
          setInterval(() => {
            setMetrics({
              timestamp: new Date().toISOString(),
              system_info: systemInfo!,
              cpu: {
                usage_percent: Math.random() * 100,
                frequency_mhz: 2400,
                per_core_usage: Array(8).fill(0).map(() => Math.random() * 100),
                temperature_celsius: 45 + Math.random() * 20,
                load_average: [1.2, 1.5, 1.8],
                processes_total: 250,
                processes_running: 5,
                context_switches: 10000,
                interrupts: 50000
              },
              memory: {
                total_bytes: 16777216000,
                used_bytes: 8388608000 + Math.random() * 2147483648,
                available_bytes: 8388608000,
                cached_bytes: 2147483648,
                swap_total_bytes: 4294967296,
                swap_used_bytes: 1073741824,
                usage_percent: 50 + Math.random() * 20,
                swap_usage_percent: 25
              },
              gpus: [],
              disks: [],
              networks: [],
              top_processes: []
            });
          }, 1000);
        }
      } catch (err) {
        console.error('Initialization error:', err);
        setError(`Initialization error: ${err}`);
      } finally {
        setLoading(false);
      }
    };
    
    init();
  }, []);

  const toggleMonitoring = async () => {
    if (!window.__TAURI__) return;
    
    try {
      const { invoke } = window.__TAURI__.tauri;
      if (isMonitoring) {
        await invoke('stop_monitoring');
        setIsMonitoring(false);
      } else {
        await invoke('start_monitoring');
        setIsMonitoring(true);
      }
    } catch (err) {
      console.error('Failed to toggle monitoring:', err);
      setError(`Failed to toggle monitoring: ${err}`);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-400">Loading System Monitor...</p>
        </div>
      </div>
    );
  }

  return (
    <ThemeProvider>
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors duration-200">
        <Header 
          systemInfo={systemInfo}
          isMonitoring={isMonitoring}
          onToggleMonitoring={toggleMonitoring}
        />
        
        {error && (
          <div className="mx-4 mt-4 p-4 bg-red-100 dark:bg-red-900/50 border border-red-300 dark:border-red-700 rounded-lg">
            <p className="text-red-700 dark:text-red-200">{error}</p>
          </div>
        )}
        
        <main className="container mx-auto px-4 py-8">
          <Dashboard systemInfo={systemInfo} metrics={metrics} />
        </main>
      </div>
    </ThemeProvider>
  );
}