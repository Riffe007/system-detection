import React from 'react';
import { DpuMetrics } from '../../types';
import { Activity, Cpu, Thermometer, Zap, Network } from 'lucide-react';

interface DpuMonitorProps {
  dpus: DpuMetrics[];
}

const DpuMonitor: React.FC<DpuMonitorProps> = ({ dpus }) => {
  if (!dpus || dpus.length === 0) {
    return null; // Don't render if no DPUs detected
  }

  return (
    <div className="bg-card rounded-lg p-6 shadow-sm border">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <Network className="h-5 w-5 text-blue-500" />
          <h3 className="text-lg font-semibold">Data Processing Units</h3>
        </div>
        <span className="text-sm text-muted-foreground">{dpus.length} detected</span>
      </div>

      <div className="space-y-4">
        {dpus.map((dpu, index) => (
          <div key={index} className="border rounded-lg p-4 bg-muted/50">
            <div className="flex items-center justify-between mb-3">
              <div>
                <h4 className="font-medium">{dpu.name}</h4>
                <p className="text-sm text-muted-foreground">
                  {dpu.vendor} {dpu.model} • {dpu.driver_version}
                </p>
              </div>
              <div className="text-right">
                <div className="text-2xl font-bold text-blue-600">
                  {dpu.usage_percent.toFixed(1)}%
                </div>
                <div className="text-xs text-muted-foreground">Usage</div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="flex items-center space-x-2">
                <Cpu className="h-4 w-4 text-green-500" />
                <span>Clock: {dpu.clock_mhz.toFixed(0)} MHz</span>
              </div>
              <div className="flex items-center space-x-2">
                <Thermometer className="h-4 w-4 text-red-500" />
                <span>Temp: {dpu.temperature_celsius.toFixed(1)}°C</span>
              </div>
              <div className="flex items-center space-x-2">
                <Zap className="h-4 w-4 text-yellow-500" />
                <span>Power: {dpu.power_watts.toFixed(1)}W</span>
              </div>
              <div className="flex items-center space-x-2">
                <Activity className="h-4 w-4 text-purple-500" />
                <span>Throughput: {dpu.throughput_gbps.toFixed(1)} Gbps</span>
              </div>
            </div>

            <div className="mt-3 pt-3 border-t">
              <div className="grid grid-cols-3 gap-4 text-xs">
                <div>
                  <div className="text-muted-foreground">Memory</div>
                  <div className="font-medium">
                    {((dpu.memory_used_bytes / dpu.memory_total_bytes) * 100).toFixed(1)}%
                  </div>
                  <div className="text-muted-foreground">
                    {Math.round(dpu.memory_used_bytes / 1024 / 1024 / 1024)}/
                    {Math.round(dpu.memory_total_bytes / 1024 / 1024 / 1024)} GB
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Packets/sec</div>
                  <div className="font-medium">
                    {(dpu.packet_processing_rate / 1000000).toFixed(1)}M
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Active Flows</div>
                  <div className="font-medium">
                    {(dpu.active_flows / 1000).toFixed(1)}K
                  </div>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default DpuMonitor; 