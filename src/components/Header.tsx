import React from 'react';
import { Activity, Pause, Play } from 'lucide-react';
import { SystemInfo } from '../types';
import { formatBytes } from '../utils/format';

interface HeaderProps {
  systemInfo: SystemInfo | null;
  isMonitoring: boolean;
  onToggleMonitoring: () => void;
}

export const Header: React.FC<HeaderProps> = ({ systemInfo, isMonitoring, onToggleMonitoring }) => {
  return (
    <header className="bg-gray-800 border-b border-gray-700">
      <div className="container mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <Activity className="w-8 h-8 text-monitor-400" />
            <div>
              <h1 className="text-2xl font-bold">System Monitor</h1>
              {systemInfo && (
                <p className="text-sm text-gray-400">
                  {systemInfo.hostname} • {systemInfo.os_name} {systemInfo.os_version} • {systemInfo.cpu_brand}
                </p>
              )}
            </div>
          </div>
          
          <button
            onClick={onToggleMonitoring}
            className={`
              flex items-center space-x-2 px-4 py-2 rounded-lg font-medium transition-colors
              ${isMonitoring 
                ? 'bg-red-600 hover:bg-red-700 text-white' 
                : 'bg-monitor-600 hover:bg-monitor-700 text-white'
              }
            `}
          >
            {isMonitoring ? (
              <>
                <Pause className="w-4 h-4" />
                <span>Pause Monitoring</span>
              </>
            ) : (
              <>
                <Play className="w-4 h-4" />
                <span>Start Monitoring</span>
              </>
            )}
          </button>
        </div>
      </div>
    </header>
  );
};