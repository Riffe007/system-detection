import React, { useMemo } from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { Cpu } from 'lucide-react';
import { CpuMetrics } from '../../types';
import { formatPercent, formatFrequency } from '../../utils/format';
import { useMetricsHistory } from '../../hooks/useMetricsHistory';

interface CpuMonitorProps {
  metrics: CpuMetrics;
}

export const CpuMonitor: React.FC<CpuMonitorProps> = ({ metrics }) => {
  const history = useMetricsHistory('cpu', metrics.usage_percent);
  
  const coreData = useMemo(() => {
    return metrics.per_core_usage.map((usage, index) => ({
      core: `Thread ${index + 1}`,
      usage: usage,
    }));
  }, [metrics.per_core_usage]);

  // Dynamically determine grid columns based on thread count
  const gridCols = useMemo(() => {
    const threadCount = metrics.per_core_usage.length;
    if (threadCount <= 4) return 'grid-cols-2 sm:grid-cols-4';
    if (threadCount <= 8) return 'grid-cols-4 sm:grid-cols-4 md:grid-cols-8';
    if (threadCount <= 16) return 'grid-cols-4 sm:grid-cols-8 md:grid-cols-8 lg:grid-cols-8';
    if (threadCount <= 32) return 'grid-cols-8 sm:grid-cols-8 md:grid-cols-16';
    // For very high thread counts, show a simplified view
    return 'grid-cols-8 sm:grid-cols-12 md:grid-cols-16';
  }, [metrics.per_core_usage.length]);

  // For systems with many threads, show a summary instead of individual threads
  const showDetailedThreads = metrics.per_core_usage.length <= 32;

  // Format load average to show actual values
  const formatLoadAverage = (loadAvg: [number, number, number]) => {
    // Check if all values are zero (likely not available on Windows)
    if (loadAvg.every(v => v === 0)) {
      return "Not available";
    }
    return loadAvg.map(v => v.toFixed(2)).join(', ');
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-colors duration-200">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <Cpu className="w-6 h-6 text-blue-600 dark:text-blue-400" />
          <h2 className="text-xl font-semibold">CPU Usage</h2>
        </div>
        <div className="text-right">
          <p className="text-3xl font-bold text-gray-900 dark:text-gray-100">{formatPercent(metrics.usage_percent)}</p>
          <p className="text-sm text-gray-400">{formatFrequency(metrics.frequency_mhz)}</p>
        </div>
      </div>
      
      <div className="space-y-4">
        <div className="h-48">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={history}>
              <defs>
                <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#0ea5e9" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#0ea5e9" stopOpacity={0}/>
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="time" stroke="#9CA3AF" />
              <YAxis domain={[0, 100]} stroke="#9CA3AF" />
              <Tooltip 
                contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151' }}
                labelStyle={{ color: '#9CA3AF' }}
              />
              <Area 
                type="monotone" 
                dataKey="value" 
                stroke="#0ea5e9" 
                fillOpacity={1} 
                fill="url(#cpuGradient)" 
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
        
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="metric-label">Load Average (1min, 5min, 15min)</p>
            <p className="text-sm text-gray-300">
              {formatLoadAverage(metrics.load_average)}
            </p>
            {metrics.load_average.every(v => v === 0) && (
              <p className="text-xs text-gray-500 mt-1">
                Load average not available on Windows systems
              </p>
            )}
          </div>
          <div>
            <p className="metric-label">Processes</p>
            <p className="text-sm text-gray-300">
              {metrics.processes_running} / {metrics.processes_total}
            </p>
          </div>
        </div>
        
        {showDetailedThreads ? (
          <div>
            <p className="metric-label mb-2">Per Thread Usage ({metrics.per_core_usage.length} threads)</p>
            <div className={`grid ${gridCols} gap-1`}>
              {coreData.map((core, index) => (
                <div key={core.core} className="text-center group">
                  <div className="relative h-12 bg-gray-700 rounded overflow-hidden">
                    <div 
                      className="absolute bottom-0 left-0 right-0 bg-monitor-500 transition-all duration-300"
                      style={{ height: `${core.usage}%` }}
                    />
                    <div className="absolute inset-0 flex items-center justify-center">
                      <span className="text-xs font-medium">{formatPercent(core.usage, 0)}</span>
                    </div>
                  </div>
                  <p className="text-xs text-gray-400 mt-1 opacity-0 group-hover:opacity-100 transition-opacity">T{index + 1}</p>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div>
            <p className="metric-label mb-2">Thread Summary ({metrics.per_core_usage.length} threads)</p>
            <div className="grid grid-cols-3 gap-4 text-sm">
              <div>
                <p className="text-gray-400">Average</p>
                <p className="text-lg font-semibold">
                  {formatPercent(
                    metrics.per_core_usage.reduce((a, b) => a + b, 0) / metrics.per_core_usage.length
                  )}
                </p>
              </div>
              <div>
                <p className="text-gray-400">Highest</p>
                <p className="text-lg font-semibold">
                  {formatPercent(Math.max(...metrics.per_core_usage))}
                </p>
              </div>
              <div>
                <p className="text-gray-400">Lowest</p>
                <p className="text-lg font-semibold">
                  {formatPercent(Math.min(...metrics.per_core_usage))}
                </p>
              </div>
            </div>
            <div className="mt-2 text-xs text-gray-400">
              Individual thread view disabled for high thread count systems
            </div>
          </div>
        )}
      </div>
    </div>
  );
};