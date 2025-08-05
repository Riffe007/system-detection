import React from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar } from 'recharts';
import { Monitor } from 'lucide-react';
import { GpuMetrics } from '../../types';
import { formatPercent, formatBytes, formatFrequency } from '../../utils/format';
import { useMetricsHistory } from '../../hooks/useMetricsHistory';

interface GpuMonitorProps {
  gpus: GpuMetrics[];
}

export const GpuMonitor: React.FC<GpuMonitorProps> = ({ gpus }) => {
  const history = useMetricsHistory('gpu', gpus.length > 0 ? gpus[0].usage_percent : 0);
  
  if (gpus.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-colors duration-200">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center space-x-3">
            <Monitor className="w-6 h-6 text-purple-600 dark:text-purple-400" />
            <h2 className="text-xl font-semibold">GPU Monitor</h2>
          </div>
        </div>
        <div className="text-center py-8">
          <Monitor className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-500 dark:text-gray-400">No GPU detected</p>
          <p className="text-sm text-gray-400 mt-2">
            GPU monitoring requires NVIDIA drivers and nvml-wrapper
          </p>
        </div>
      </div>
    );
  }

  const primaryGpu = gpus[0];
  const gpuData = gpus.map((gpu) => ({
    name: gpu.name,
    usage: gpu.usage_percent,
    memory: gpu.memory_usage_percent,
    temperature: gpu.temperature_celsius,
    power: gpu.power_watts,
  }));

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-colors duration-200">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <Monitor className="w-6 h-6 text-purple-600 dark:text-purple-400" />
          <h2 className="text-xl font-semibold">GPU Monitor</h2>
        </div>
        <div className="text-right">
          <p className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            {formatPercent(primaryGpu.usage_percent)}
          </p>
          <p className="text-sm text-gray-400">{primaryGpu.name}</p>
        </div>
      </div>
      
      <div className="space-y-6">
        {/* GPU Usage History */}
        <div className="h-48">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={history}>
              <defs>
                <linearGradient id="gpuGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#9333ea" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#9333ea" stopOpacity={0}/>
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
                stroke="#9333ea" 
                fillOpacity={1} 
                fill="url(#gpuGradient)" 
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* GPU Details Grid */}
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="metric-label">Temperature</p>
            <p className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
              {primaryGpu.temperature_celsius.toFixed(1)}°C
            </p>
          </div>
          <div>
            <p className="metric-label">Memory Usage</p>
            <p className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
              {formatPercent(primaryGpu.memory_usage_percent)}
            </p>
            <p className="text-sm text-gray-400">
              {formatBytes(primaryGpu.memory_used_bytes)} / {formatBytes(primaryGpu.memory_total_bytes)}
            </p>
          </div>
          <div>
            <p className="metric-label">Power Draw</p>
            <p className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
              {primaryGpu.power_watts.toFixed(1)}W
            </p>
          </div>
          <div>
            <p className="metric-label">Clock Speed</p>
            <p className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
              {formatFrequency(primaryGpu.clock_mhz * 1000000)}
            </p>
          </div>
        </div>

        {/* Multiple GPUs Bar Chart */}
        {gpus.length > 1 && (
          <div>
            <p className="metric-label mb-4">Multi-GPU Usage</p>
            <div className="h-32">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={gpuData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                  <XAxis dataKey="name" stroke="#9CA3AF" />
                  <YAxis domain={[0, 100]} stroke="#9CA3AF" />
                  <Tooltip 
                    contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151' }}
                    labelStyle={{ color: '#9CA3AF' }}
                  />
                  <Bar dataKey="usage" fill="#9333ea" />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </div>
        )}

        {/* GPU Details Table */}
        <div>
          <p className="metric-label mb-3">GPU Details</p>
          <div className="space-y-2">
            {gpus.map((gpu, index) => (
              <div key={index} className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
                <div className="flex justify-between items-center mb-2">
                  <h4 className="font-medium text-gray-900 dark:text-gray-100">{gpu.name}</h4>
                  <span className="text-sm text-gray-500 dark:text-gray-400">
                    Driver: {gpu.driver_version}
                  </span>
                </div>
                <div className="grid grid-cols-3 gap-2 text-sm">
                  <div>
                    <span className="text-gray-500 dark:text-gray-400">Usage:</span>
                    <span className="ml-1 font-medium">{formatPercent(gpu.usage_percent)}</span>
                  </div>
                  <div>
                    <span className="text-gray-500 dark:text-gray-400">Memory:</span>
                    <span className="ml-1 font-medium">{formatPercent(gpu.memory_usage_percent)}</span>
                  </div>
                  <div>
                    <span className="text-gray-500 dark:text-gray-400">Temp:</span>
                    <span className="ml-1 font-medium">{gpu.temperature_celsius.toFixed(1)}°C</span>
                  </div>
                </div>
                {gpu.fan_speed_percent && (
                  <div className="mt-2 text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Fan Speed:</span>
                    <span className="ml-1 font-medium">
                      {gpu.fan_speed_percent.toFixed(0)}%
                    </span>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}; 