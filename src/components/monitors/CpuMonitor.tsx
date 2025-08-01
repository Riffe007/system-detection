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
      core: `Core ${index}`,
      usage: usage,
    }));
  }, [metrics.per_core_usage]);

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
            <p className="metric-label">Load Average</p>
            <p className="text-sm text-gray-300">
              {metrics.load_average.map(v => v.toFixed(2)).join(', ')}
            </p>
          </div>
          <div>
            <p className="metric-label">Processes</p>
            <p className="text-sm text-gray-300">
              {metrics.processes_running} / {metrics.processes_total}
            </p>
          </div>
        </div>
        
        <div>
          <p className="metric-label mb-2">Per Core Usage</p>
          <div className="grid grid-cols-4 gap-2">
            {coreData.map((core) => (
              <div key={core.core} className="text-center">
                <div className="relative h-20 bg-gray-700 rounded overflow-hidden">
                  <div 
                    className="absolute bottom-0 left-0 right-0 bg-monitor-500 transition-all duration-300"
                    style={{ height: `${core.usage}%` }}
                  />
                  <div className="absolute inset-0 flex items-center justify-center">
                    <span className="text-xs font-medium">{formatPercent(core.usage, 0)}</span>
                  </div>
                </div>
                <p className="text-xs text-gray-400 mt-1">{core.core}</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};