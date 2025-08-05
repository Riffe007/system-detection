import React from 'react';
import { NpuMetrics } from '../../types';
import { Brain, Cpu, Thermometer, Zap, Activity } from 'lucide-react';

interface NpuMonitorProps {
  npus: NpuMetrics[];
}

const NpuMonitor: React.FC<NpuMonitorProps> = ({ npus }) => {
  if (!npus || npus.length === 0) {
    return null; // Don't render if no NPUs detected
  }

  return (
    <div className="bg-card rounded-lg p-6 shadow-sm border">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <Brain className="h-5 w-5 text-purple-500" />
          <h3 className="text-lg font-semibold">Neural Processing Units</h3>
        </div>
        <span className="text-sm text-muted-foreground">{npus.length} detected</span>
      </div>

      <div className="space-y-4">
        {npus.map((npu, index) => (
          <div key={index} className="border rounded-lg p-4 bg-muted/50">
            <div className="flex items-center justify-between mb-3">
              <div>
                <h4 className="font-medium">{npu.name}</h4>
                <p className="text-sm text-muted-foreground">
                  {npu.vendor} {npu.model} • {npu.driver_version}
                </p>
              </div>
              <div className="text-right">
                <div className="text-2xl font-bold text-purple-600">
                  {npu.usage_percent.toFixed(1)}%
                </div>
                <div className="text-xs text-muted-foreground">Usage</div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="flex items-center space-x-2">
                <Cpu className="h-4 w-4 text-green-500" />
                <span>Clock: {npu.clock_mhz.toFixed(0)} MHz</span>
              </div>
              <div className="flex items-center space-x-2">
                <Thermometer className="h-4 w-4 text-red-500" />
                <span>Temp: {npu.temperature_celsius.toFixed(1)}°C</span>
              </div>
              <div className="flex items-center space-x-2">
                <Zap className="h-4 w-4 text-yellow-500" />
                <span>Power: {npu.power_watts.toFixed(1)}W</span>
              </div>
              <div className="flex items-center space-x-2">
                <Activity className="h-4 w-4 text-blue-500" />
                <span>Inference: {(npu.inference_rate / 1000).toFixed(1)}K/s</span>
              </div>
            </div>

            <div className="mt-3 pt-3 border-t">
              <div className="grid grid-cols-3 gap-4 text-xs">
                <div>
                  <div className="text-muted-foreground">Memory</div>
                  <div className="font-medium">
                    {((npu.memory_used_bytes / npu.memory_total_bytes) * 100).toFixed(1)}%
                  </div>
                  <div className="text-muted-foreground">
                    {Math.round(npu.memory_used_bytes / 1024 / 1024 / 1024)}/
                    {Math.round(npu.memory_total_bytes / 1024 / 1024 / 1024)} GB
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Accuracy</div>
                  <div className="font-medium text-green-600">
                    {(npu.model_accuracy * 100).toFixed(1)}%
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Active Models</div>
                  <div className="font-medium">
                    {npu.active_models}
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

export default NpuMonitor; 