import { useEffect, useState } from 'react';
import { Dashboard } from './components/Dashboard';
import { Header } from './components/Header';
import { ThemeProvider } from './contexts/ThemeContext';
import { SystemInfo, SystemMetrics } from './types';
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

  console.log('AppWrapper state initialized');

  useEffect(() => {
    console.log('AppWrapper useEffect running');
    const init = async () => {
      try {
        console.log('Starting initialization...');
        
        // Set a timeout for the entire init process
        const initTimeout = setTimeout(() => {
          console.error('Initialization timeout - setting loading to false');
          setLoading(false);
          setError('Initialization timed out. Please refresh the page.');
        }, 15000); // 15 seconds timeout
        
        // Wait a bit for Tauri to be available
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Use our comprehensive Tauri detector
        const isTauri = await detectTauriEnvironment();
        
        console.log('=== Tauri Detection Result ===');
        console.log('Is Tauri environment:', isTauri);
        
        if (!isTauri) {
          console.error('Tauri environment not detected');
          setError('This application requires Tauri runtime. Please run the application through Tauri, not in a browser.');
          setLoading(false);
          return;
        }
        
        console.log('Using real Tauri backend');
        console.log('TAURI DETECTED - USING REAL SYSTEM DATA');
        
        // Add a visual indicator
        document.title = 'System Monitor';
        
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
        
        // Test basic connectivity by trying to get system info directly
        console.log('Testing Tauri connectivity...');
        
        // Load system info
        try {
          console.log('Calling get_system_info...');
          const info = await invoke('get_system_info');
          console.log('System info received:', info);
          setSystemInfo(info);
          console.log('System info state updated');
        } catch (err) {
          console.error('Failed to load system info:', err);
          setError(`Failed to load system info: ${err}`);
          return; // Don't continue if we can't get system info
        }
        
        // Start monitoring
        try {
          console.log('Starting monitoring...');
          await invoke('start_monitoring');
          setIsMonitoring(true);
          console.log('Monitoring started successfully');
          console.log('Monitoring state updated');
        } catch (err) {
          console.error('Failed to start monitoring:', err);
          setError(`Failed to start monitoring: ${err}`);
          return; // Don't continue if we can't start monitoring
        }
        
        // Listen for metrics updates
        try {
          console.log('Setting up metrics listener...');
          const unlisten = await listen('system-metrics', (event: any) => {
            console.log('Received metrics update via event:', event);
            console.log('Event payload:', event.payload);
            setMetrics(event.payload);
          });
          console.log('Metrics listener set up successfully');
          
          // Set up a fallback polling mechanism in case events don't work
          const pollInterval = setInterval(async () => {
            try {
              console.log('Polling for metrics...');
              const currentMetrics = await invoke('get_current_metrics');
              console.log('Polled metrics received:', currentMetrics);
              setMetrics(currentMetrics);
            } catch (err) {
              console.error('Polling fallback failed:', err);
              console.error('Error details:', err.message, err.stack);
            }
          }, 2000); // Poll every 2 seconds as fallback
          
          // Store cleanup function for later
          const cleanup = () => {
            console.log('Cleaning up metrics listener...');
            clearInterval(pollInterval);
            unlisten();
            invoke('stop_monitoring').catch(console.error);
          };
          
          // Clear the timeout since we completed successfully
          clearTimeout(initTimeout);
          setError(null); // Clear any previous errors
          console.log('Initialization completed successfully');
          
          // Return cleanup function
          return cleanup;
        } catch (err) {
          console.error('Failed to set up metrics listener:', err);
          console.log('Continuing without event listener, will use polling fallback');
          
          // Set up polling as primary method if event listener fails
          const pollInterval = setInterval(async () => {
            try {
              console.log('Polling for metrics (primary method)...');
              const currentMetrics = await invoke('get_current_metrics');
              console.log('Polled metrics received (primary):', currentMetrics);
              setMetrics(currentMetrics);
            } catch (err) {
              console.error('Polling failed (primary):', err);
              console.error('Error details:', err.message, err.stack);
            }
          }, 2000);
          
          // Store cleanup function for later
          const cleanup = () => {
            console.log('Cleaning up polling...');
            clearInterval(pollInterval);
            invoke('stop_monitoring').catch(console.error);
          };
          
          // Clear the timeout since we completed successfully
          clearTimeout(initTimeout);
          setError(null); // Clear any previous errors
          console.log('Initialization completed successfully');
          
          // Return cleanup function
          return cleanup;
        }
      } catch (err) {
        console.error('Initialization error:', err);
        setError(`Initialization error: ${err}`);
      } finally {
        console.log('Setting loading to false');
        setLoading(false);
        console.log('Loading state set to false');
      }
    };
    
    init();
  }, []);

  const toggleMonitoring = async () => {
    try {
      const isTauri = await detectTauriEnvironment();
        
      if (!isTauri) {
        throw new Error('Tauri environment not available');
      }
      
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
    } catch (err) {
      console.error('Failed to toggle monitoring:', err);
      setError(`Failed to toggle monitoring: ${err}`);
    }
  };

  console.log('AppWrapper render - loading:', loading, 'systemInfo:', !!systemInfo, 'error:', error);
  
  if (loading) {
    console.log('Rendering loading state');
    return (
      <div className="min-h-screen bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-400">Loading System Monitor...</p>
        </div>
      </div>
    );
  }

  console.log('Rendering main app');
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