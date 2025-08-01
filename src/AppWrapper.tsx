import { useEffect, useState } from 'react';
import { Dashboard } from './components/Dashboard';
import { Header } from './components/Header';
import { ThemeProvider } from './contexts/ThemeContext';
import { SystemInfo, SystemMetrics } from './types';
import { mockTauri, mockListen } from './services/mockTauri';
import { detectTauriEnvironment, getTauriInvoke, getTauriListen } from './services/tauriDetector';
import './App.css';

declare global {
  interface Window {
    __TAURI__?: any;
  }
}

export default function AppWrapper() {
  console.log('AppWrapper component loaded');
  
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [isMockData, setIsMockData] = useState(true);

  console.log('AppWrapper state initialized');

  useEffect(() => {
    console.log('AppWrapper useEffect running');
    const init = async () => {
      try {
        // Wait a bit for Tauri to be available
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Use our comprehensive Tauri detector
        const isTauri = await detectTauriEnvironment();
        
        console.log('=== Tauri Detection Result ===');
        console.log('Is Tauri environment:', isTauri);
        
        if (isTauri) {
          console.log('Using real Tauri backend');
          console.log('TAURI DETECTED - NOT USING MOCK SERVICE');
          
          // Add a visual indicator
          document.title = 'System Monitor (Tauri)';
          setIsMockData(false);
          
          // Get Tauri invoke function
          const invoke = await getTauriInvoke();
          if (!invoke) {
            console.error('Failed to get Tauri invoke function');
            throw new Error('Tauri invoke not available');
          }
          
          // Get listen function
          const listen = await getTauriListen();
          if (!listen) {
            console.error('Failed to get Tauri listen function');
            throw new Error('Tauri listen not available');
          }
          
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
          // Running in browser - use mock Tauri service
          console.log('Tauri not available, using mock service');
          console.log('USING MOCK SERVICE - NOT REAL SYSTEM DATA');
          
          // Add a visual indicator
          document.title = 'System Monitor (Mock Data)';
          setIsMockData(true);
          
          console.log('Window object:', window);
          console.log('Tauri object details:', {
            hasTauri: !!window.__TAURI__,
            hasTauriTauri: !!(window.__TAURI__ && window.__TAURI__.tauri),
            hasTauriEvent: !!(window.__TAURI__ && window.__TAURI__.event),
            invokeType: window.__TAURI__?.tauri?.invoke ? typeof window.__TAURI__.tauri.invoke : 'undefined'
          });
          
          // Load system info
          try {
            const info = await mockTauri.invoke('get_system_info');
            setSystemInfo(info);
          } catch (err) {
            console.error('Failed to load mock system info:', err);
            setError(`Failed to load system info: ${err}`);
          }
          
          // Start monitoring
          try {
            await mockTauri.invoke('start_monitoring');
            setIsMonitoring(true);
          } catch (err) {
            console.error('Failed to start mock monitoring:', err);
            setError(`Failed to start monitoring: ${err}`);
          }
          
          // Listen for metrics updates
          const unlisten = await mockListen('system-metrics', (event: any) => {
            setMetrics(event.payload);
          });
          
          // Cleanup
          return () => {
            unlisten();
            mockTauri.invoke('stop_monitoring').catch(console.error);
          };
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
    try {
      const isTauri = await detectTauriEnvironment();
        
      if (isTauri) {
        const invoke = await getTauriInvoke();
        if (!invoke) {
          throw new Error('Tauri invoke not available');
        }
        if (isMonitoring) {
          await invoke('stop_monitoring');
          setIsMonitoring(false);
        } else {
          await invoke('start_monitoring');
          setIsMonitoring(true);
        }
      } else {
        // Use mock service
        if (isMonitoring) {
          await mockTauri.invoke('stop_monitoring');
          setIsMonitoring(false);
        } else {
          await mockTauri.invoke('start_monitoring');
          setIsMonitoring(true);
        }
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
        
        {isMockData && (
          <div className="mx-4 mt-4 p-4 bg-yellow-100 dark:bg-yellow-900/50 border border-yellow-300 dark:border-yellow-700 rounded-lg">
            <p className="text-yellow-700 dark:text-yellow-200 font-bold">
              ⚠️ MOCK DATA MODE - This is not real system data! 
              To see real data, run the application through Tauri, not in the browser.
            </p>
          </div>
        )}
        
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