import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Dashboard } from './components/Dashboard';
import { Header } from './components/Header';
import { SystemInfo, SystemMetrics } from './types';
import './App.css';

function App() {
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Load system info on mount
    loadSystemInfo();
    
    // Start monitoring automatically
    startMonitoring();

    // Listen for metrics updates
    const unsubscribe = listen<SystemMetrics>('system-metrics', (event) => {
      setMetrics(event.payload);
    });

    return () => {
      unsubscribe.then(fn => fn());
      stopMonitoring();
    };
  }, []);

  const loadSystemInfo = async () => {
    try {
      const info = await invoke<SystemInfo>('get_system_info');
      setSystemInfo(info);
    } catch (err) {
      setError(`Failed to load system info: ${err}`);
    }
  };

  const startMonitoring = async () => {
    try {
      await invoke('start_monitoring');
      setIsMonitoring(true);
      setError(null);
    } catch (err) {
      setError(`Failed to start monitoring: ${err}`);
    }
  };

  const stopMonitoring = async () => {
    try {
      await invoke('stop_monitoring');
      setIsMonitoring(false);
    } catch (err) {
      console.error('Failed to stop monitoring:', err);
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100">
      <Header 
        systemInfo={systemInfo}
        isMonitoring={isMonitoring}
        onToggleMonitoring={isMonitoring ? stopMonitoring : startMonitoring}
      />
      
      {error && (
        <div className="mx-4 mt-4 p-4 bg-red-900/50 border border-red-700 rounded-lg">
          <p className="text-red-200">{error}</p>
        </div>
      )}
      
      <main className="container mx-auto px-4 py-8">
        <Dashboard systemInfo={systemInfo} metrics={metrics} />
      </main>
    </div>
  );
}

export default App;