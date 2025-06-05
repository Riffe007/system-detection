import React from 'react';
import { HardDrive } from 'lucide-react';
import { DiskMetrics } from '../../types';
import { formatBytes, formatPercent } from '../../utils/format';

interface DiskMonitorProps {
  disks: DiskMetrics[];
}

export const DiskMonitor: React.FC<DiskMonitorProps> = ({ disks }) => {
  if (disks.length === 0) {
    return (
      <div className="metric-card">
        <div className="flex items-center space-x-3 mb-6">
          <HardDrive className="w-6 h-6 text-monitor-400" />
          <h2 className="text-xl font-semibold">Disk Usage</h2>
        </div>
        <p className="text-gray-400">No disk information available</p>
      </div>
    );
  }

  return (
    <div className="metric-card">
      <div className="flex items-center space-x-3 mb-6">
        <HardDrive className="w-6 h-6 text-monitor-400" />
        <h2 className="text-xl font-semibold">Disk Usage</h2>
      </div>
      
      <div className="space-y-4">
        {disks.map((disk) => (
          <div key={disk.mount_point} className="p-4 bg-gray-700/50 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <div>
                <p className="font-medium">{disk.mount_point}</p>
                <p className="text-sm text-gray-400">{disk.device_name} • {disk.fs_type}</p>
              </div>
              <div className="text-right">
                <p className="font-bold text-monitor-400">{formatPercent(disk.usage_percent)}</p>
                <p className="text-sm text-gray-400">
                  {formatBytes(disk.used_bytes)} / {formatBytes(disk.total_bytes)}
                </p>
              </div>
            </div>
            
            <div className="w-full bg-gray-600 rounded-full h-2">
              <div 
                className="bg-monitor-500 h-2 rounded-full transition-all duration-300"
                style={{ width: `${disk.usage_percent}%` }}
              />
            </div>
            
            {(disk.read_bytes_per_sec > 0 || disk.write_bytes_per_sec > 0) && (
              <div className="mt-2 flex justify-between text-sm">
                <span className="text-gray-400">
                  Read: {formatBytes(disk.read_bytes_per_sec)}/s
                </span>
                <span className="text-gray-400">
                  Write: {formatBytes(disk.write_bytes_per_sec)}/s
                </span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};