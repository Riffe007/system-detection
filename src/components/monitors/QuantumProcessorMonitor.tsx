import React from 'react';
import { QuantumProcessorMetrics } from '../../types';
import { Atom, Thermometer, Zap, Activity, Cpu } from 'lucide-react';

interface QuantumProcessorMonitorProps {
  quantumProcessors: QuantumProcessorMetrics[];
}

const QuantumProcessorMonitor: React.FC<QuantumProcessorMonitorProps> = ({ quantumProcessors }) => {
  if (!quantumProcessors || quantumProcessors.length === 0) {
    return null; // Don't render if no quantum processors detected
  }

  return (
    <div className="bg-card rounded-lg p-6 shadow-sm border">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <Atom className="h-5 w-5 text-indigo-500" />
          <h3 className="text-lg font-semibold">Quantum Processors</h3>
        </div>
        <span className="text-sm text-muted-foreground">{quantumProcessors.length} detected</span>
      </div>

      <div className="space-y-4">
        {quantumProcessors.map((qp, index) => (
          <div key={index} className="border rounded-lg p-4 bg-muted/50">
            <div className="flex items-center justify-between mb-3">
              <div>
                <h4 className="font-medium">{qp.name}</h4>
                <p className="text-sm text-muted-foreground">
                  {qp.vendor} • {qp.qubits} qubits
                </p>
              </div>
              <div className="text-right">
                <div className="text-2xl font-bold text-indigo-600">
                  {qp.active_qubits}
                </div>
                <div className="text-xs text-muted-foreground">Active Qubits</div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="flex items-center space-x-2">
                <Cpu className="h-4 w-4 text-green-500" />
                <span>Total Qubits: {qp.qubits}</span>
              </div>
              <div className="flex items-center space-x-2">
                <Thermometer className="h-4 w-4 text-red-500" />
                <span>Temp: {qp.temperature_mk.toFixed(1)} mK</span>
              </div>
              <div className="flex items-center space-x-2">
                <Zap className="h-4 w-4 text-yellow-500" />
                <span>Power: {qp.power_watts.toFixed(1)}W</span>
              </div>
              <div className="flex items-center space-x-2">
                <Activity className="h-4 w-4 text-blue-500" />
                <span>Coherence: {qp.coherence_time_ms.toFixed(1)}ms</span>
              </div>
            </div>

            <div className="mt-3 pt-3 border-t">
              <div className="grid grid-cols-3 gap-4 text-xs">
                <div>
                  <div className="text-muted-foreground">Gate Fidelity</div>
                  <div className="font-medium text-green-600">
                    {(qp.gate_fidelity * 100).toFixed(2)}%
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Error Rate</div>
                  <div className="font-medium text-red-600">
                    {(qp.error_rate * 100).toFixed(3)}%
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Active Qubits</div>
                  <div className="font-medium">
                    {qp.active_qubits}/{qp.qubits}
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

export default QuantumProcessorMonitor; 