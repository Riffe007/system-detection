import React from 'react';
import { SystemInfo, SystemMetrics } from '../types';
import { CpuMonitor } from './monitors/CpuMonitor';
import { MemoryMonitor } from './monitors/MemoryMonitor';
import { DiskMonitor } from './monitors/DiskMonitor';
import { NetworkMonitor } from './monitors/NetworkMonitor';
import { ProcessList } from './monitors/ProcessList';
import { SystemOverview } from './SystemOverview';

interface DashboardProps {
  systemInfo: SystemInfo | null;
  metrics: SystemMetrics | null;
}

export const Dashboard: React.FC<DashboardProps> = ({ systemInfo, metrics }) => {
  if (!systemInfo || !metrics) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-monitor-400 mx-auto mb-4"></div>
          <p className="text-gray-400">Initializing system monitoring...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SystemOverview systemInfo={systemInfo} />
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <CpuMonitor metrics={metrics.cpu} />
        <MemoryMonitor metrics={metrics.memory} />
      </div>
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <DiskMonitor disks={metrics.disks} />
        <NetworkMonitor networks={metrics.networks} />
      </div>
      
      <ProcessList processes={metrics.top_processes} />
    </div>
  );
};