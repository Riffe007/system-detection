import { useEffect, useState } from 'react';
import { Dashboard } from './components/Dashboard';
import { Header } from './components/Header';
import { ThemeProvider } from './contexts/ThemeContext';
import { SystemInfo, SystemMetrics } from './types';
import { mockTauri, mockListen } from './services/mockTauri';
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
        // Wait a bit for Tauri to be available
        await new Promise(resolve => setTimeout(resolve, 100));
        
        // Check if we're in Tauri environment
        console.log('Tauri window object:', window.__TAURI__);
        console.log('Window object keys:', Object.keys(window));
        console.log('All window properties:', Object.getOwnPropertyNames(window));
        
        // Try different Tauri detection methods
        const tauriObject = (window as any).__TAURI__;
        console.log('Raw Tauri object:', tauriObject);
        console.log('Tauri object keys:', tauriObject ? Object.keys(tauriObject) : 'undefined');
        
        // More robust Tauri detection
        const isTauriAvailable = window.__TAURI__ && 
          window.__TAURI__.tauri && 
          window.__TAURI__.event &&
          typeof window.__TAURI__.tauri.invoke === 'function';
        
        // Alternative detection: check if we're in a Tauri webview
        const isTauriWebview = navigator.userAgent.includes('Tauri') || 
          window.location.protocol === 'tauri:' ||
          window.location.href.includes('tauri');
        
        console.log('Is Tauri available:', isTauriAvailable);
        console.log('Is Tauri webview:', isTauriWebview);
        console.log('User agent:', navigator.userAgent);
        console.log('Location:', window.location.href);
        
        if (isTauriAvailable || isTauriWebview) {
          console.log('Using real Tauri backend');
          
          // Check if Tauri APIs are actually available
          if (!window.__TAURI__?.tauri?.invoke) {
            console.error('Tauri invoke not available, falling back to mock');
            throw new Error('Tauri APIs not available');
          }
          
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
          // Running in browser - use mock Tauri service
          console.log('Tauri not available, using mock service');
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
      const isTauriAvailable = window.__TAURI__ && 
        window.__TAURI__.tauri && 
        window.__TAURI__.event &&
        typeof window.__TAURI__.tauri.invoke === 'function';
      
      const isTauriWebview = navigator.userAgent.includes('Tauri') || 
        window.location.protocol === 'tauri:' ||
        window.location.href.includes('tauri');
        
      if (isTauriAvailable || isTauriWebview) {
        const { invoke } = window.__TAURI__.tauri;
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