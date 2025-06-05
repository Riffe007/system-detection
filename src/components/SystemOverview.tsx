import React from 'react';
import { SystemInfo } from '../types';
import { formatBytes, formatUptime } from '../utils/format';
import { Cpu, HardDrive, Wifi, Activity } from 'lucide-react';

interface SystemOverviewProps {
  systemInfo: SystemInfo;
}

export const SystemOverview: React.FC<SystemOverviewProps> = ({ systemInfo }) => {
  const uptimeSeconds = Math.floor((new Date().getTime() - new Date(systemInfo.boot_time).getTime()) / 1000);
  
  return (
    <div className="bg-gray-800 rounded-lg p-6 mb-6">
      <h2 className="text-2xl font-bold text-white mb-4">System Overview</h2>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center mb-2">
            <Cpu className="w-5 h-5 text-blue-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-300">Processor</h3>
          </div>
          <p className="text-white font-semibold">{systemInfo.cpu_brand}</p>
          <p className="text-sm text-gray-400">
            {systemInfo.cpu_cores} cores, {systemInfo.cpu_threads} threads
          </p>
        </div>
        
        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center mb-2">
            <HardDrive className="w-5 h-5 text-green-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-300">Memory</h3>
          </div>
          <p className="text-white font-semibold">{formatBytes(systemInfo.total_memory)}</p>
          <p className="text-sm text-gray-400">Total System Memory</p>
        </div>
        
        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center mb-2">
            <Wifi className="w-5 h-5 text-purple-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-300">System</h3>
          </div>
          <p className="text-white font-semibold">{systemInfo.hostname}</p>
          <p className="text-sm text-gray-400">
            {systemInfo.os_name} {systemInfo.os_version}
          </p>
        </div>
        
        <div className="bg-gray-700 rounded-lg p-4">
          <div className="flex items-center mb-2">
            <Activity className="w-5 h-5 text-orange-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-300">Uptime</h3>
          </div>
          <p className="text-white font-semibold">{formatUptime(uptimeSeconds)}</p>
          <p className="text-sm text-gray-400">
            {systemInfo.architecture} • Kernel {systemInfo.kernel_version}
          </p>
        </div>
      </div>
    </div>
  );
};