import React from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { MemoryStick } from 'lucide-react';
import { MemoryMetrics } from '../../types';
import { formatBytes, formatPercent } from '../../utils/format';
import { useMetricsHistory } from '../../hooks/useMetricsHistory';

interface MemoryMonitorProps {
  metrics: MemoryMetrics;
}

export const MemoryMonitor: React.FC<MemoryMonitorProps> = ({ metrics }) => {
  const history = useMetricsHistory('memory', metrics.usage_percent);
  
  const pieData = [
    { name: 'Used', value: metrics.used_bytes, color: '#0ea5e9' },
    { name: 'Available', value: metrics.available_bytes, color: '#1e293b' },
  ];

  return (
    <div className="metric-card">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <MemoryStick className="w-6 h-6 text-monitor-400" />
          <h2 className="text-xl font-semibold">Memory Usage</h2>
        </div>
        <div className="text-right">
          <p className="metric-value">{formatPercent(metrics.usage_percent)}</p>
          <p className="text-sm text-gray-400">
            {formatBytes(metrics.used_bytes)} / {formatBytes(metrics.total_bytes)}
          </p>
        </div>
      </div>
      
      <div className="space-y-4">
        <div className="h-48">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={history}>
              <defs>
                <linearGradient id="memoryGradient" x1="0" y1="0" x2="0" y2="1">
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
                fill="url(#memoryGradient)" 
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
        
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-3">
            <div>
              <p className="metric-label">Physical Memory</p>
              <div className="mt-1 space-y-1">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-400">Used</span>
                  <span>{formatBytes(metrics.used_bytes)}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-400">Available</span>
                  <span>{formatBytes(metrics.available_bytes)}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-400">Total</span>
                  <span>{formatBytes(metrics.total_bytes)}</span>
                </div>
              </div>
            </div>
            
            {metrics.swap_total_bytes > 0 && (
              <div>
                <p className="metric-label">Swap Memory</p>
                <div className="mt-1 space-y-1">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-400">Used</span>
                    <span>{formatBytes(metrics.swap_used_bytes)}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-400">Total</span>
                    <span>{formatBytes(metrics.swap_total_bytes)}</span>
                  </div>
                </div>
              </div>
            )}
          </div>
          
          <div className="flex items-center justify-center">
            <div className="relative">
              <ResponsiveContainer width={120} height={120}>
                <PieChart>
                  <Pie
                    data={pieData}
                    cx={60}
                    cy={60}
                    innerRadius={35}
                    outerRadius={50}
                    paddingAngle={2}
                    dataKey="value"
                  >
                    {pieData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                </PieChart>
              </ResponsiveContainer>
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-lg font-bold">{formatPercent(metrics.usage_percent, 0)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};