import React, { useState } from 'react';
import { Activity } from 'lucide-react';
import { ProcessMetrics } from '../../types';
import { formatBytes, formatPercent } from '../../utils/format';

interface ProcessListProps {
  processes: ProcessMetrics[];
}

export const ProcessList: React.FC<ProcessListProps> = ({ processes }) => {
  const [sortBy, setSortBy] = useState<'cpu' | 'memory'>('cpu');
  
  const sortedProcesses = [...processes].sort((a, b) => {
    if (sortBy === 'cpu') {
      return b.cpu_usage_percent - a.cpu_usage_percent;
    }
    return b.memory_bytes - a.memory_bytes;
  });

  return (
    <div className="metric-card">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <Activity className="w-6 h-6 text-monitor-400" />
          <h2 className="text-xl font-semibold">Top Processes</h2>
        </div>
        <div className="flex space-x-2">
          <button
            onClick={() => setSortBy('cpu')}
            className={`px-3 py-1 text-sm rounded transition-colors ${
              sortBy === 'cpu' 
                ? 'bg-monitor-600 text-white' 
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            Sort by CPU
          </button>
          <button
            onClick={() => setSortBy('memory')}
            className={`px-3 py-1 text-sm rounded transition-colors ${
              sortBy === 'memory' 
                ? 'bg-monitor-600 text-white' 
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            Sort by Memory
          </button>
        </div>
      </div>
      
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="text-left border-b border-gray-700">
              <th className="pb-2 text-sm font-medium text-gray-400">PID</th>
              <th className="pb-2 text-sm font-medium text-gray-400">Name</th>
              <th className="pb-2 text-sm font-medium text-gray-400 text-right">CPU</th>
              <th className="pb-2 text-sm font-medium text-gray-400 text-right">Memory</th>
              <th className="pb-2 text-sm font-medium text-gray-400 text-center">Status</th>
              <th className="pb-2 text-sm font-medium text-gray-400 text-right">Threads</th>
            </tr>
          </thead>
          <tbody>
            {sortedProcesses.slice(0, 10).map((process) => (
              <tr key={process.pid} className="border-b border-gray-700/50 hover:bg-gray-700/30">
                <td className="py-2 text-sm">{process.pid}</td>
                <td className="py-2 text-sm font-medium max-w-xs truncate" title={process.name}>
                  {process.name}
                </td>
                <td className="py-2 text-sm text-right">
                  <span className={process.cpu_usage_percent > 50 ? 'text-orange-400' : ''}>
                    {formatPercent(process.cpu_usage_percent)}
                  </span>
                </td>
                <td className="py-2 text-sm text-right">
                  <div>
                    <span className={process.memory_percent > 20 ? 'text-orange-400' : ''}>
                      {formatBytes(process.memory_bytes)}
                    </span>
                    <span className="text-gray-500 ml-1">
                      ({formatPercent(process.memory_percent)})
                    </span>
                  </div>
                </td>
                <td className="py-2 text-sm text-center">
                  <span className={`px-2 py-1 rounded text-xs ${
                    process.status === 'Running' 
                      ? 'bg-green-900/50 text-green-400' 
                      : 'bg-gray-700 text-gray-400'
                  }`}>
                    {process.status}
                  </span>
                </td>
                <td className="py-2 text-sm text-right">{process.threads}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};