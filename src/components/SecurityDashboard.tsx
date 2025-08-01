import React, { useState } from 'react';
import { SecurityMetrics } from '../types';
import { Shield, AlertTriangle, Eye, Activity, TrendingUp, Trash2, RotateCcw, Ban, Zap } from 'lucide-react';
import { formatBytes } from '../utils/format';
import { getTauriInvoke } from '../services/tauriDetector';

interface SecurityDashboardProps {
  metrics: SecurityMetrics | null;
}

export const SecurityDashboard: React.FC<SecurityDashboardProps> = ({ metrics }) => {
  const [loading, setLoading] = useState<string | null>(null);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const invoke = getTauriInvoke();

  const handleAction = async (action: string, ...args: any[]) => {
    setLoading(action);
    setMessage(null);
    
    try {
      const result = await invoke(action, ...args);
      setMessage({ type: 'success', text: result });
      // Refresh the page after a short delay to show updated data
      setTimeout(() => window.location.reload(), 2000);
    } catch (error) {
      setMessage({ type: 'error', text: `Error: ${error}` });
    } finally {
      setLoading(null);
    }
  };

  if (!metrics) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <Shield className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400">Loading security metrics...</p>
        </div>
      </div>
    );
  }

  const getSecurityScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-600 dark:text-green-400';
    if (score >= 60) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-red-600 dark:text-red-400';
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'Critical': return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400';
      case 'High': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-400';
      case 'Medium': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400';
      case 'Low': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400';
    }
  };

  return (
    <div className="p-6 space-y-6">
      {/* Message Display */}
      {message && (
        <div className={`p-4 rounded-lg ${
          message.type === 'success' 
            ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400' 
            : 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
        }`}>
          {message.text}
        </div>
      )}

      {/* Quick Actions */}
      {(metrics.suspicious_processes.length > 0 || metrics.network_connections.some(c => c.connection_type.includes('SUSPICIOUS'))) && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Quick Actions</h3>
          <div className="flex flex-wrap gap-3">
            {metrics.suspicious_processes.length > 0 && (
              <button
                onClick={() => handleAction('quarantine_all_suspicious_processes')}
                disabled={loading === 'quarantine_all_suspicious_processes'}
                className="flex items-center px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Zap className="w-4 h-4 mr-2" />
                {loading === 'quarantine_all_suspicious_processes' ? 'Quarantining...' : 'Quarantine All Suspicious Processes'}
              </button>
            )}
            
            {metrics.network_connections.some(c => c.connection_type.includes('SUSPICIOUS')) && (
              <button
                onClick={() => handleAction('block_all_suspicious_connections')}
                disabled={loading === 'block_all_suspicious_connections'}
                className="flex items-center px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Ban className="w-4 h-4 mr-2" />
                {loading === 'block_all_suspicious_connections' ? 'Blocking...' : 'Block All Suspicious Connections'}
              </button>
            )}
          </div>
        </div>
      )}

      {/* Security Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <Shield className="w-8 h-8 text-blue-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Security Score</p>
              <p className={`text-2xl font-bold ${getSecurityScoreColor(metrics.security_score)}`}>
                {Math.round(metrics.security_score)}%
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <AlertTriangle className="w-8 h-8 text-red-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Threats Detected</p>
              <p className="text-2xl font-bold text-red-600 dark:text-red-400">
                {metrics.threats_detected}
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <Activity className="w-8 h-8 text-green-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Total Processes</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                {metrics.total_processes}
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <Eye className="w-8 h-8 text-purple-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Suspicious Processes</p>
              <p className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                {metrics.suspicious_processes.length}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Suspicious Processes */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Suspicious Processes
            </h3>
          </div>
          <div className="p-6">
            {metrics.suspicious_processes.length === 0 ? (
              <p className="text-gray-500 dark:text-gray-400 text-center py-4">
                No suspicious processes detected
              </p>
            ) : (
              <div className="space-y-3">
                {metrics.suspicious_processes.map((process) => (
                  <div key={process.pid} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                    <div className="flex-1">
                      <div className="flex items-center justify-between">
                        <p className="font-medium text-gray-900 dark:text-gray-100">
                          {process.name}
                        </p>
                        <span className="text-sm text-gray-500 dark:text-gray-400">
                          PID: {process.pid}
                        </span>
                      </div>
                      <div className="flex items-center space-x-4 mt-1 text-sm text-gray-600 dark:text-gray-400">
                        <span>CPU: {process.cpu_usage.toFixed(1)}%</span>
                        <span>Memory: {formatBytes(process.memory_usage)}</span>
                        <span className={`font-medium ${process.suspicious_score > 50 ? 'text-red-600 dark:text-red-400' : 'text-yellow-600 dark:text-yellow-400'}`}>
                          Score: {process.suspicious_score.toFixed(1)}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2 ml-4">
                      <button
                        onClick={() => handleAction('quarantine_suspicious_process', process.pid)}
                        disabled={loading === `quarantine_${process.pid}`}
                        className="p-2 text-red-600 hover:bg-red-100 dark:hover:bg-red-900/20 rounded-lg disabled:opacity-50"
                        title="Quarantine Process"
                      >
                        <Zap className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Security Events */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Recent Security Events
            </h3>
          </div>
          <div className="p-6">
            {metrics.recent_events.length === 0 ? (
              <p className="text-gray-500 dark:text-gray-400 text-center py-4">
                No recent security events
              </p>
            ) : (
              <div className="space-y-3">
                {metrics.recent_events.slice(0, 5).map((event, index) => (
                  <div key={index} className="flex items-start space-x-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                    <div className={`px-2 py-1 rounded-full text-xs font-medium ${getSeverityColor(event.severity)}`}>
                      {event.severity}
                    </div>
                    <div className="flex-1">
                      <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                        {event.description}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {new Date(parseInt(event.timestamp) * 1000).toLocaleString()}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Quarantined Processes and Blocked Connections */}
      {(metrics.quarantined_processes.length > 0 || metrics.blocked_connections.length > 0) && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Quarantined Processes */}
          {metrics.quarantined_processes.length > 0 && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  Quarantined Processes ({metrics.quarantined_processes.length})
                </h3>
              </div>
              <div className="p-6">
                <div className="space-y-3">
                  {metrics.quarantined_processes.map((process) => (
                    <div key={process.pid} className="flex items-center justify-between p-3 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
                      <div className="flex-1">
                        <div className="flex items-center justify-between">
                          <p className="font-medium text-gray-900 dark:text-gray-100">
                            {process.name}
                          </p>
                          <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                            process.status === 'Quarantined' ? 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400' :
                            process.status === 'Restored' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' :
                            'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
                          }`}>
                            {process.status}
                          </span>
                        </div>
                        <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                          <p>PID: {process.pid}</p>
                          <p>Reason: {process.reason}</p>
                          <p>Quarantined: {new Date(parseInt(process.quarantine_time) * 1000).toLocaleString()}</p>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2 ml-4">
                        {process.status === 'Quarantined' && (
                          <>
                            <button
                              onClick={() => handleAction('restore_quarantined_process', process.pid)}
                              disabled={loading === `restore_${process.pid}`}
                              className="p-2 text-green-600 hover:bg-green-100 dark:hover:bg-green-900/20 rounded-lg disabled:opacity-50"
                              title="Restore Process"
                            >
                              <RotateCcw className="w-4 h-4" />
                            </button>
                            <button
                              onClick={() => handleAction('delete_quarantined_process', process.pid)}
                              disabled={loading === `delete_${process.pid}`}
                              className="p-2 text-red-600 hover:bg-red-100 dark:hover:bg-red-900/20 rounded-lg disabled:opacity-50"
                              title="Delete Process"
                            >
                              <Trash2 className="w-4 h-4" />
                            </button>
                          </>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Blocked Connections */}
          {metrics.blocked_connections.length > 0 && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  Blocked Connections ({metrics.blocked_connections.length})
                </h3>
              </div>
              <div className="p-6">
                <div className="space-y-3">
                  {metrics.blocked_connections.map((connection, index) => (
                    <div key={index} className="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
                      <div className="flex items-center justify-between">
                        <p className="font-medium text-gray-900 dark:text-gray-100">
                          {connection.remote_address}:{connection.port}
                        </p>
                        <span className="text-sm text-gray-500 dark:text-gray-400">
                          {connection.protocol}
                        </span>
                      </div>
                      <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                        <p>Reason: {connection.reason}</p>
                        <p>Blocked: {new Date(parseInt(connection.block_time) * 1000).toLocaleString()}</p>
                        <p>Rule: {connection.firewall_rule_name}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Recommendations */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Security Recommendations
          </h3>
        </div>
        <div className="p-6">
          {metrics.recommendations.length === 0 ? (
            <p className="text-gray-500 dark:text-gray-400 text-center py-4">
              No recommendations at this time
            </p>
          ) : (
            <div className="space-y-3">
              {metrics.recommendations.map((recommendation, index) => (
                <div key={index} className="flex items-start space-x-3 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <TrendingUp className="w-5 h-5 text-blue-500 mt-0.5" />
                  <p className="text-sm text-gray-900 dark:text-gray-100">
                    {recommendation}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 