import React from 'react';
import { Network, WifiOff, Wifi } from 'lucide-react';
import { NetworkMetrics } from '../../types';
import { formatBytes } from '../../utils/format';

interface NetworkMonitorProps {
  networks: NetworkMetrics[];
}

export const NetworkMonitor: React.FC<NetworkMonitorProps> = ({ networks }) => {
  if (networks.length === 0) {
    return (
      <div className="metric-card">
        <div className="flex items-center space-x-3 mb-6">
          <Network className="w-6 h-6 text-monitor-400" />
          <h2 className="text-xl font-semibold">Network Interfaces</h2>
        </div>
        <p className="text-gray-400">No network interfaces found</p>
      </div>
    );
  }

  return (
    <div className="metric-card">
      <div className="flex items-center space-x-3 mb-6">
        <Network className="w-6 h-6 text-monitor-400" />
        <h2 className="text-xl font-semibold">Network Interfaces</h2>
      </div>
      
      <div className="space-y-4">
        {networks.map((network) => (
          <div key={network.interface_name} className="p-4 bg-gray-700/50 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-2">
                {network.is_up ? (
                  <Wifi className="w-4 h-4 text-green-400" />
                ) : (
                  <WifiOff className="w-4 h-4 text-red-400" />
                )}
                <div>
                  <p className="font-medium">{network.interface_name}</p>
                  <p className="text-sm text-gray-400">{network.mac_address}</p>
                </div>
              </div>
              <div className="text-right">
                <span className={`text-sm px-2 py-1 rounded ${
                  network.is_up ? 'bg-green-900/50 text-green-400' : 'bg-red-900/50 text-red-400'
                }`}>
                  {network.is_up ? 'UP' : 'DOWN'}
                </span>
              </div>
            </div>
            
            {network.ip_addresses.length > 0 && (
              <div className="mb-2">
                <p className="text-sm text-gray-400">IP Addresses:</p>
                <p className="text-sm">{network.ip_addresses.join(', ')}</p>
              </div>
            )}
            
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-gray-400">Download</p>
                <p className="font-medium">{formatBytes(network.bytes_received)}</p>
                <p className="text-xs text-gray-500">{network.packets_received.toLocaleString()} packets</p>
              </div>
              <div>
                <p className="text-gray-400">Upload</p>
                <p className="font-medium">{formatBytes(network.bytes_sent)}</p>
                <p className="text-xs text-gray-500">{network.packets_sent.toLocaleString()} packets</p>
              </div>
            </div>
            
            {(network.errors_received > 0 || network.errors_sent > 0) && (
              <div className="mt-2 text-sm text-red-400">
                Errors: {network.errors_received} received, {network.errors_sent} sent
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};