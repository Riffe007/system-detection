import React from 'react';
import { FpgaMetrics } from '../../types';
import { Cpu, Thermometer, Zap, Settings, Activity } from 'lucide-react';

interface FpgaMonitorProps {
  fpgas: FpgaMetrics[];
}

const FpgaMonitor: React.FC<FpgaMonitorProps> = ({ fpgas }) => {
  if (!fpgas || fpgas.length === 0) {
    return null; // Don't render if no FPGAs detected
  }

  return (
    <div className="bg-card rounded-lg p-6 shadow-sm border">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <Settings className="h-5 w-5 text-orange-500" />
          <h3 className="text-lg font-semibold">Field-Programmable Gate Arrays</h3>
        </div>
        <span className="text-sm text-muted-foreground">{fpgas.length} detected</span>
      </div>

      <div className="space-y-4">
        {fpgas.map((fpga, index) => (
          <div key={index} className="border rounded-lg p-4 bg-muted/50">
            <div className="flex items-center justify-between mb-3">
              <div>
                <h4 className="font-medium">{fpga.name}</h4>
                <p className="text-sm text-muted-foreground">
                  {fpga.vendor} {fpga.model} • {fpga.bitstream_version}
                </p>
              </div>
              <div className="text-right">
                <div className="text-2xl font-bold text-orange-600">
                  {fpga.usage_percent.toFixed(1)}%
                </div>
                <div className="text-xs text-muted-foreground">Usage</div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="flex items-center space-x-2">
                <Cpu className="h-4 w-4 text-green-500" />
                <span>Clock: {fpga.clock_mhz.toFixed(0)} MHz</span>
              </div>
              <div className="flex items-center space-x-2">
                <Thermometer className="h-4 w-4 text-red-500" />
                <span>Temp: {fpga.temperature_celsius.toFixed(1)}°C</span>
              </div>
              <div className="flex items-center space-x-2">
                <Zap className="h-4 w-4 text-yellow-500" />
                <span>Power: {fpga.power_watts.toFixed(1)}W</span>
              </div>
              <div className="flex items-center space-x-2">
                <Activity className="h-4 w-4 text-blue-500" />
                <span>Logic: {fpga.logic_utilization.toFixed(1)}%</span>
              </div>
            </div>

            <div className="mt-3 pt-3 border-t">
              <div className="grid grid-cols-3 gap-4 text-xs">
                <div>
                  <div className="text-muted-foreground">Logic Utilization</div>
                  <div className="font-medium text-green-600">
                    {fpga.logic_utilization.toFixed(1)}%
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Memory Utilization</div>
                  <div className="font-medium text-blue-600">
                    {fpga.memory_utilization.toFixed(1)}%
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">DSP Utilization</div>
                  <div className="font-medium text-purple-600">
                    {fpga.dsp_utilization.toFixed(1)}%
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

export default FpgaMonitor; 