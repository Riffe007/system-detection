import React from 'react';
import { SystemInfo } from '../types';
import { formatBytes, formatUptime } from '../utils/format';
import { Cpu, HardDrive, Wifi, Activity } from 'lucide-react';

interface SystemOverviewProps {
  systemInfo: SystemInfo;
}

export const SystemOverview: React.FC<SystemOverviewProps> = ({ systemInfo }) => {
  const uptimeSeconds = Math.floor(Date.now() / 1000) - systemInfo.boot_time;
  
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-6 mb-6 shadow-lg border border-gray-200 dark:border-gray-700 transition-colors duration-200">
      <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">System Overview</h2>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 transition-colors duration-200">
          <div className="flex items-center mb-2">
            <Cpu className="w-5 h-5 text-blue-500 dark:text-blue-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">Processor</h3>
          </div>
          <p className="text-gray-900 dark:text-white font-semibold">{systemInfo.cpu_brand}</p>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            {systemInfo.cpu_cores} cores, {systemInfo.cpu_threads} threads
          </p>
        </div>
        
        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 transition-colors duration-200">
          <div className="flex items-center mb-2">
            <HardDrive className="w-5 h-5 text-green-500 dark:text-green-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">Memory</h3>
          </div>
          <p className="text-gray-900 dark:text-white font-semibold">{formatBytes(systemInfo.total_memory)}</p>
          <p className="text-sm text-gray-600 dark:text-gray-400">Total System Memory</p>
        </div>
        
        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 transition-colors duration-200">
          <div className="flex items-center mb-2">
            <Wifi className="w-5 h-5 text-purple-500 dark:text-purple-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">System</h3>
          </div>
          <p className="text-gray-900 dark:text-white font-semibold">{systemInfo.hostname}</p>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            {systemInfo.os_name} {systemInfo.os_version}
          </p>
        </div>
        
        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 transition-colors duration-200">
          <div className="flex items-center mb-2">
            <Activity className="w-5 h-5 text-orange-500 dark:text-orange-400 mr-2" />
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">Uptime</h3>
          </div>
          <p className="text-gray-900 dark:text-white font-semibold">{formatUptime(uptimeSeconds)}</p>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            {systemInfo.architecture} • Kernel {systemInfo.kernel_version}
          </p>
        </div>
      </div>
    </div>
  );
};