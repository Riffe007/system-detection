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
  console.log('=== AppWrapper component loaded ===');
  console.log('Window location:', window.location.href);
  console.log('Window __TAURI__:', window.__TAURI__);
  console.log('Document title:', document.title);
  console.log('Document ready state:', document.readyState);
  
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
        }, 10000); // 10 seconds timeout
        
        // Simple Tauri detection - just check if window.__TAURI__ exists
        console.log('Checking for Tauri environment...');
        console.log('window.__TAURI__:', window.__TAURI__);
        
        if (!window.__TAURI__) {
          console.error('Tauri environment not detected');
          setError('This application requires Tauri runtime. Please run the application through Tauri, not in a browser.');
          setLoading(false);
          return;
        }
        
        console.log('✓ Tauri environment detected');
        
        console.log('Using real Tauri backend');
        console.log('TAURI DETECTED - USING REAL SYSTEM DATA');
        
        // Add a visual indicator
        document.title = 'System Monitor';
        
        // Get Tauri invoke function - use direct access
        const invoke = window.__TAURI__.core?.invoke || window.__TAURI__.tauri?.invoke;
        if (!invoke) {
          console.error('Failed to get Tauri invoke function');
          throw new Error('Tauri invoke not available');
        }
        console.log('✓ Tauri invoke function available');
        
        // Get listen function - use direct access
        const listen = window.__TAURI__.event?.listen || window.__TAURI__.tauri?.event?.listen;
        if (!listen) {
          console.error('Failed to get Tauri listen function');
          throw new Error('Tauri listen not available');
        }
        console.log('✓ Tauri listen function available');
        
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
        
                 // Get initial metrics immediately
         try {
           console.log('Getting initial metrics...');
           const initialMetrics = await invoke('get_current_metrics');
           console.log('Initial metrics received:', initialMetrics);
           if (initialMetrics) {
             setMetrics(initialMetrics);
             console.log('Initial metrics state updated');
           }
         } catch (err) {
           console.error('Failed to get initial metrics:', err);
         }
         
         // Set up polling as the primary method for getting metrics
         console.log('Setting up polling for metrics...');
         const pollInterval = setInterval(async () => {
           try {
             console.log('Polling for metrics...');
             const currentMetrics = await invoke('get_current_metrics');
             console.log('Polled metrics received:', currentMetrics);
             if (currentMetrics) {
               setMetrics(currentMetrics);
               console.log('Metrics state updated via polling');
             }
           } catch (err) {
             console.error('Polling failed:', err);
             if (err instanceof Error) {
               console.error('Error details:', err.message, err.stack);
             }
           }
         }, 3000); // Poll every 3 seconds
          
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
          
          // Set loading to false since we have system info and monitoring is started
          setLoading(false);
          console.log('Loading state set to false - initialization complete');
          
          // Return cleanup function
          return cleanup;
        
      } catch (err) {
        console.error('Initialization error:', err);
        setError(`Initialization error: ${err}`);
      }
    };
    
    init();
  }, []);

  const toggleMonitoring = async () => {
    try {
      if (!window.__TAURI__) {
        throw new Error('Tauri environment not available');
      }
      
      const invoke = window.__TAURI__.core?.invoke || window.__TAURI__.tauri?.invoke;
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

  console.log('AppWrapper render - loading:', loading, 'systemInfo:', !!systemInfo, 'metrics:', !!metrics, 'error:', error);
  
  if (loading) {
    console.log('Rendering loading state');
    return (
      <div className="min-h-screen bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-400">Loading System Monitor...</p>
          <p className="text-gray-500 text-sm mt-2">System Info: {systemInfo ? '✓' : '⏳'} | Metrics: {metrics ? '✓' : '⏳'}</p>
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