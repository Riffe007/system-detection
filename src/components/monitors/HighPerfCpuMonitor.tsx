import React, { useMemo, useRef, useEffect } from 'react';
import { XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line } from 'recharts';
import { Cpu, Zap, Thermometer, Activity } from 'lucide-react';
import { HighPerfCpuMetrics } from '../../types';
import { formatPercent, formatFrequency } from '../../utils/format';
import { CircularBuffer, PerformanceMonitor } from '../../services/highPerfService';

interface HighPerfCpuMonitorProps {
  metrics: HighPerfCpuMetrics;
  history?: CircularBuffer<HighPerfCpuMetrics>;
}

export const HighPerfCpuMonitor: React.FC<HighPerfCpuMonitorProps> = ({ 
  metrics, 
  history 
}) => {
  const perfMonitor = PerformanceMonitor.getInstance();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();
  const lastUpdateRef = useRef<number>(0);



  // Optimized data processing
  const processedData = useMemo(() => {
    const stopTimer = perfMonitor.startTimer('cpu-data-processing');
    
    const coreData = metrics.per_core_usage.map((usage, index) => ({
      core: `Thread ${index + 1}`,
      usage: usage,
      frequency: metrics.frequency_mhz[index] || 0,
    }));

    const result = {
      coreData,
      globalUsage: metrics.global_usage,
      loadAverage: metrics.load_average,
      temperature: metrics.temperature,
      contextSwitches: metrics.context_switches,
      interrupts: metrics.interrupts,
      cacheMisses: metrics.cache_misses,
      cacheHits: metrics.cache_hits,
    };

    stopTimer();
    return result;
  }, [metrics, perfMonitor]);

  // High-performance chart data
  const chartData = useMemo(() => {
    if (!history) return [];

    const stopTimer = perfMonitor.startTimer('chart-data-processing');
    const data = history.toArray().slice(-50); // Last 50 data points
    
    const result = data.map((item, index) => ({
      time: index,
      global: item.global_usage,
      avg: item.per_core_usage.reduce((sum: number, usage: number) => sum + usage, 0) / item.per_core_usage.length,
      max: Math.max(...item.per_core_usage),
    }));

    stopTimer();
    return result;
  }, [history, perfMonitor]);

  // Real-time canvas rendering for ultra-high-frequency updates
  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const renderCanvas = () => {
      const now = performance.now();
      const deltaTime = now - lastUpdateRef.current;
      
      if (deltaTime < 16) { // Cap at ~60 FPS
        animationRef.current = requestAnimationFrame(renderCanvas);
        return;
      }

      lastUpdateRef.current = now;
      
      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      
      // Draw real-time CPU usage bars
      const barWidth = canvas.width / processedData.coreData.length;
      const maxHeight = canvas.height * 0.8;
      
      processedData.coreData.forEach((core, index) => {
        const x = index * barWidth;
        const height = (core.usage / 100) * maxHeight;
        const y = canvas.height - height;
        
        // Gradient fill
        const gradient = ctx.createLinearGradient(x, y, x, canvas.height);
        gradient.addColorStop(0, '#3b82f6');
        gradient.addColorStop(1, '#1d4ed8');
        
        ctx.fillStyle = gradient;
        ctx.fillRect(x + 2, y, barWidth - 4, height);
        
        // Core label
        ctx.fillStyle = '#ffffff';
        ctx.font = '10px Arial';
        ctx.textAlign = 'center';
        ctx.fillText(core.core, x + barWidth / 2, canvas.height - 5);
      });
    };

    renderCanvas();

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [processedData]);

  // Performance metrics display
  const performanceStats = useMemo(() => {
    const cpuProcessingTime = perfMonitor.getAverage('cpu-data-processing');
    const chartProcessingTime = perfMonitor.getAverage('chart-data-processing');
    
    return {
      cpuProcessingTime: cpuProcessingTime.toFixed(2),
      chartProcessingTime: chartProcessingTime.toFixed(2),
      fps: (1000 / (cpuProcessingTime + chartProcessingTime)).toFixed(1),
    };
  }, [perfMonitor]);

  const formatLoadAverage = (loadAvg: [number, number, number]) => {
    if (loadAvg.every(v => v === 0)) {
      return "Not available";
    }
    return loadAvg.map(v => v.toFixed(2)).join(', ');
  };

  const cacheHitRate = metrics.cache_hits > 0 || metrics.cache_misses > 0
    ? (metrics.cache_hits / (metrics.cache_hits + metrics.cache_misses) * 100).toFixed(1)
    : '0.0';

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-colors duration-200">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <Cpu className="w-6 h-6 text-blue-600 dark:text-blue-400" />
          <h2 className="text-xl font-semibold">High-Performance CPU Monitor</h2>
        </div>
        <div className="text-right">
          <p className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            {formatPercent(processedData.globalUsage)}
          </p>
          <p className="text-sm text-gray-400">Global Usage</p>
        </div>
      </div>

      {/* Performance Stats */}
      <div className="mb-4 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
        <div className="grid grid-cols-3 gap-4 text-xs">
          <div>
            <span className="text-gray-500 dark:text-gray-400">Processing:</span>
            <span className="ml-1 font-mono">{performanceStats.cpuProcessingTime}ms</span>
          </div>
          <div>
            <span className="text-gray-500 dark:text-gray-400">Chart:</span>
            <span className="ml-1 font-mono">{performanceStats.chartProcessingTime}ms</span>
          </div>
          <div>
            <span className="text-gray-500 dark:text-gray-400">FPS:</span>
            <span className="ml-1 font-mono">{performanceStats.fps}</span>
          </div>
        </div>
      </div>

      {/* Real-time Canvas Display */}
      <div className="mb-6">
        <p className="metric-label mb-2">Real-time Thread Usage</p>
        <canvas
          ref={canvasRef}
          width={800}
          height={200}
          className="w-full h-48 bg-gray-900 rounded-lg"
        />
      </div>

      {/* High-performance Chart */}
      <div className="mb-6">
        <p className="metric-label mb-2">Usage History (Last 50 samples)</p>
        <div className="h-48">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={chartData}>
              <defs>
                <linearGradient id="globalGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="time" stroke="#9CA3AF" />
              <YAxis domain={[0, 100]} stroke="#9CA3AF" />
              <Tooltip 
                contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151' }}
                labelStyle={{ color: '#9CA3AF' }}
              />
              <Line 
                type="monotone" 
                dataKey="global" 
                stroke="#3b82f6" 
                strokeWidth={2}
                dot={false}
              />
              <Line 
                type="monotone" 
                dataKey="avg" 
                stroke="#10b981" 
                strokeWidth={1}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Detailed Metrics Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
          <div className="flex items-center space-x-2 mb-2">
            <Activity className="w-4 h-4 text-blue-600" />
            <span className="text-sm font-medium">Load Average</span>
          </div>
          <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {formatLoadAverage(processedData.loadAverage)}
          </p>
          <p className="text-xs text-gray-500">(1min, 5min, 15min)</p>
        </div>

        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
          <div className="flex items-center space-x-2 mb-2">
            <Thermometer className="w-4 h-4 text-red-600" />
            <span className="text-sm font-medium">Temperature</span>
          </div>
          <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {processedData.temperature ? `${processedData.temperature.toFixed(1)}°C` : 'N/A'}
          </p>
        </div>

        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
          <div className="flex items-center space-x-2 mb-2">
            <Zap className="w-4 h-4 text-yellow-600" />
            <span className="text-sm font-medium">Cache Hit Rate</span>
          </div>
          <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {cacheHitRate}%
          </p>
          <p className="text-xs text-gray-500">
            {metrics.cache_hits.toLocaleString()} hits
          </p>
        </div>

        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
          <div className="flex items-center space-x-2 mb-2">
            <Cpu className="w-4 h-4 text-purple-600" />
            <span className="text-sm font-medium">Context Switches</span>
          </div>
          <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {processedData.contextSwitches.toLocaleString()}
          </p>
          <p className="text-xs text-gray-500">
            {processedData.interrupts.toLocaleString()} interrupts
          </p>
        </div>
      </div>

      {/* Thread Details */}
      <div className="mt-6">
        <p className="metric-label mb-3">Thread Details ({processedData.coreData.length} threads)</p>
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-2">
          {processedData.coreData.map((core, index) => (
            <div key={index} className="bg-gray-50 dark:bg-gray-700 rounded p-2 text-center">
              <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                {core.core}
              </div>
              <div className="text-lg font-bold text-blue-600">
                {formatPercent(core.usage)}
              </div>
              <div className="text-xs text-gray-500">
                {formatFrequency(core.frequency * 1000000)}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}; 