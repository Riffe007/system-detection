import { useState, useEffect, useRef } from 'react';
import { Monitor, Cpu, HardDrive, Clock, TrendingUp, AlertTriangle, Database } from 'lucide-react';
import { kernelMonitoringService, KernelMetrics } from '../../services/kernelService';

interface KernelMonitorProps {
  className?: string;
}

export default function KernelMonitor({ className }: KernelMonitorProps) {
  const [metrics, setMetrics] = useState<KernelMetrics | null>(null);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    startKernelMonitoring();
    return () => {
      stopKernelMonitoring();
    };
  }, []);

  useEffect(() => {
    if (metrics && canvasRef.current) {
      drawLatencyGraph();
    }
  }, [metrics]);

  const startKernelMonitoring = async () => {
    try {
      await kernelMonitoringService.startMonitoring((newMetrics) => {
        setMetrics(newMetrics);
      });
      setIsMonitoring(true);
    } catch (error) {
      console.error('Failed to start kernel monitoring:', error);
    }
  };

  const stopKernelMonitoring = async () => {
    try {
      await kernelMonitoringService.stopMonitoring();
      setIsMonitoring(false);
    } catch (error) {
      console.error('Failed to stop kernel monitoring:', error);
    }
  };

  const drawLatencyGraph = () => {
    const canvas = canvasRef.current;
    if (!canvas || !metrics) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw latency visualization
    const collectionLatency = metrics.latency.collection_latency_ns / 1000; // Convert to microseconds
    const processingLatency = metrics.latency.processing_latency_ns / 1000;
    const totalLatency = metrics.latency.total_latency_ns / 1000;

    // Color coding based on performance targets
    const getLatencyColor = (latency: number, target: number) => {
      if (latency <= target) return '#10b981'; // Green
      if (latency <= target * 2) return '#f59e0b'; // Yellow
      return '#ef4444'; // Red
    };

    // Draw collection latency bar
    ctx.fillStyle = getLatencyColor(collectionLatency, 10);
    ctx.fillRect(10, 10, Math.min(collectionLatency / 2, width - 20), 20);

    // Draw processing latency bar
    ctx.fillStyle = getLatencyColor(processingLatency, 1);
    ctx.fillRect(10, 40, Math.min(processingLatency * 10, width - 20), 20);

    // Draw total latency bar
    ctx.fillStyle = getLatencyColor(totalLatency, 16);
    ctx.fillRect(10, 70, Math.min(totalLatency / 2, width - 20), 20);

    // Add labels
    ctx.fillStyle = '#ffffff';
    ctx.font = '12px monospace';
    ctx.fillText(`Collection: ${collectionLatency.toFixed(2)}µs`, 15, 25);
    ctx.fillText(`Processing: ${processingLatency.toFixed(2)}µs`, 15, 55);
    ctx.fillText(`Total: ${totalLatency.toFixed(2)}µs`, 15, 85);
  };

  const getPerformanceStatus = () => {
    if (!metrics) return { status: 'Unknown', color: 'bg-gray-500' };

    const latencyAnalysis = kernelMonitoringService.analyzeLatency(metrics);
    
    if (latencyAnalysis.isOptimal) {
      return { status: 'Optimal', color: 'bg-green-500' };
    } else if (latencyAnalysis.totalLatency < 50) {
      return { status: 'Good', color: 'bg-yellow-500' };
    } else {
      return { status: 'Poor', color: 'bg-red-500' };
    }
  };

  const getCpuEfficiency = () => {
    if (!metrics) return 0;
    const analysis = kernelMonitoringService.analyzeCpuPerformance(metrics);
    return analysis.efficiency * 100;
  };

  const getMemoryHealth = () => {
    if (!metrics) return { health: 0, status: 'Unknown' };
    const analysis = kernelMonitoringService.analyzeMemoryPressure(metrics);
    return {
      health: (1 - analysis.pressure) * 100,
      status: analysis.isHealthy ? 'Healthy' : 'Pressure'
    };
  };

  if (!isMonitoring) {
    return (
      <div className={`bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-colors duration-200 ${className || ''}`}>
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-3">
            <Monitor className="w-6 h-6 text-blue-600 dark:text-blue-400" />
            <h2 className="text-xl font-semibold">Kernel-Level Monitoring</h2>
          </div>
        </div>
        <div className="flex items-center justify-center h-32">
          <div className="text-center">
            <AlertTriangle className="h-8 w-8 text-yellow-500 mx-auto mb-2" />
            <p className="text-sm text-gray-500 dark:text-gray-400">Kernel monitoring not available</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-colors duration-200 ${className || ''}`}>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-3">
          <Monitor className="w-6 h-6 text-blue-600 dark:text-blue-400" />
          <h2 className="text-xl font-semibold">Kernel-Level Monitoring</h2>
        </div>
        <div className={`px-2 py-1 rounded-full text-xs font-medium ${getPerformanceStatus().color} text-white`}>
          {getPerformanceStatus().status}
        </div>
      </div>
      
      <div className="space-y-4">
        {/* Latency Visualization */}
        <div>
          <h4 className="text-sm font-medium mb-2 flex items-center gap-2">
            <Clock className="h-4 w-4" />
            Latency Analysis (Target: &lt;10µs)
          </h4>
          <canvas
            ref={canvasRef}
            width={300}
            height={100}
            style={{ width: '100%', height: '96px' }}
            className="bg-gray-900 rounded border"
          />
        </div>

        {/* CPU Performance */}
        <div>
          <h4 className="text-sm font-medium mb-2 flex items-center gap-2">
            <Cpu className="h-4 w-4" />
            CPU Performance
          </h4>
          <div className="grid grid-cols-2 gap-2 text-xs">
            <div>
              <span className="text-gray-500 dark:text-gray-400">IPC:</span>
              <span className="ml-1 font-mono">
                {metrics?.cpu.instructions && metrics?.cpu.cycles 
                  ? (metrics.cpu.instructions / metrics.cpu.cycles).toFixed(2)
                  : 'N/A'
                }
              </span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">Efficiency:</span>
              <span className="ml-1 font-mono">{getCpuEfficiency().toFixed(1)}%</span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">Cache Miss:</span>
              <span className="ml-1 font-mono">
                {metrics?.cpu.cache_misses && metrics?.cpu.instructions
                  ? ((metrics.cpu.cache_misses / metrics.cpu.instructions) * 100).toFixed(2) + '%'
                  : 'N/A'
                }
              </span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">Branch Miss:</span>
              <span className="ml-1 font-mono">
                {metrics?.cpu.branch_misses && metrics?.cpu.instructions
                  ? ((metrics.cpu.branch_misses / metrics.cpu.instructions) * 100).toFixed(2) + '%'
                  : 'N/A'
                }
              </span>
            </div>
          </div>
        </div>

        {/* Memory Health */}
        <div>
          <h4 className="text-sm font-medium mb-2 flex items-center gap-2">
            <Database className="h-4 w-4" />
            Memory Health
          </h4>
          <div className="space-y-2">
            <div className="flex justify-between text-xs">
              <span>Memory Pressure</span>
              <span className="font-mono">{getMemoryHealth().status}</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2 dark:bg-gray-700">
              <div 
                className="bg-blue-600 h-2 rounded-full transition-all duration-300" 
                style={{ width: `${getMemoryHealth().health}%` }}
              ></div>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div>
                <span className="text-gray-500 dark:text-gray-400">Page Faults:</span>
                <span className="ml-1 font-mono">{metrics?.memory.page_faults.toLocaleString()}</span>
              </div>
              <div>
                <span className="text-gray-500 dark:text-gray-400">NUMA Hits:</span>
                <span className="ml-1 font-mono">{metrics?.memory.numa_hits.toLocaleString()}</span>
              </div>
            </div>
          </div>
        </div>

        {/* I/O Performance */}
        <div>
          <h4 className="text-sm font-medium mb-2 flex items-center gap-2">
            <HardDrive className="h-4 w-4" />
            I/O Performance
          </h4>
          <div className="grid grid-cols-2 gap-2 text-xs">
            <div>
              <span className="text-gray-500 dark:text-gray-400">Disk Latency:</span>
              <span className="ml-1 font-mono">
                {metrics?.disk.latency_ns ? (metrics.disk.latency_ns / 1000).toFixed(2) + 'µs' : 'N/A'}
              </span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">Queue Depth:</span>
              <span className="ml-1 font-mono">{metrics?.disk.queue_depth || 'N/A'}</span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">Network Latency:</span>
              <span className="ml-1 font-mono">
                {metrics?.network.latency_ns ? (metrics.network.latency_ns / 1000).toFixed(2) + 'µs' : 'N/A'}
              </span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">Packet Drops:</span>
              <span className="ml-1 font-mono">
                {metrics?.network.drops_in && metrics?.network.drops_out
                  ? (metrics.network.drops_in + metrics.network.drops_out).toLocaleString()
                  : 'N/A'
                }
              </span>
            </div>
          </div>
        </div>

        {/* Performance Summary */}
        <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between text-xs">
            <span className="text-gray-500 dark:text-gray-400">Performance Score:</span>
            <div className="flex items-center gap-1">
              <TrendingUp className="h-3 w-3" />
              <span className="font-mono">
                {metrics ? Math.round(getCpuEfficiency() * getMemoryHealth().health / 100) : 0}/100
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
} 