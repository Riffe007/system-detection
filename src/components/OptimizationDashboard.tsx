import React from 'react';
import { OptimizationMetrics } from '../types';
import { Zap, AlertTriangle, Gauge, Lightbulb } from 'lucide-react';
import { formatBytes } from '../utils/format';

interface OptimizationDashboardProps {
  metrics: OptimizationMetrics | null;
}

export const OptimizationDashboard: React.FC<OptimizationDashboardProps> = ({ metrics }) => {
  if (!metrics) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <Zap className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400">Loading optimization metrics...</p>
        </div>
      </div>
    );
  }

  const getPerformanceScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-600 dark:text-green-400';
    if (score >= 60) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-red-600 dark:text-red-400';
  };

  const getHealthColor = (health: number) => {
    if (health >= 80) return 'text-green-600 dark:text-green-400';
    if (health >= 60) return 'text-yellow-600 dark:text-yellow-400';
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
      {/* Performance Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <Zap className="w-8 h-8 text-blue-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Performance Score</p>
              <p className={`text-2xl font-bold ${getPerformanceScoreColor(metrics.overall_performance_score)}`}>
                {Math.round(metrics.overall_performance_score)}%
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <Gauge className="w-8 h-8 text-green-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">System Health</p>
              <p className={`text-2xl font-bold ${getHealthColor(metrics.system_health.overall_health)}`}>
                {Math.round(metrics.system_health.overall_health)}%
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <AlertTriangle className="w-8 h-8 text-red-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Bottlenecks</p>
              <p className="text-2xl font-bold text-red-600 dark:text-red-400">
                {metrics.bottlenecks?.length || 0}
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div className="flex items-center">
            <Lightbulb className="w-8 h-8 text-purple-500" />
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Recommendations</p>
              <p className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                {metrics.recommendations?.length || 0}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Resource Usage */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Resource Usage
            </h3>
          </div>
          <div className="p-6 space-y-4">
            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">CPU Usage</span>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {metrics.resource_usage?.cpu_usage?.toFixed(1) || '0.0'}%
                </span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div 
                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${metrics.resource_usage?.cpu_usage || 0}%` }}
                ></div>
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Memory Usage</span>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {metrics.resource_usage?.memory_usage?.toFixed(1) || '0.0'}%
                </span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div 
                  className="bg-green-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${metrics.resource_usage?.memory_usage || 0}%` }}
                ></div>
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Disk Usage</span>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {metrics.resource_usage?.disk_usage?.toFixed(1) || '0.0'}%
                </span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div 
                  className="bg-yellow-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${metrics.resource_usage?.disk_usage || 0}%` }}
                ></div>
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Network Usage</span>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {metrics.resource_usage?.network_usage?.toFixed(1) || '0.0'}%
                </span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                <div 
                  className="bg-purple-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${metrics.resource_usage?.network_usage || 0}%` }}
                ></div>
              </div>
            </div>
          </div>
        </div>

        {/* System Health */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              System Health
            </h3>
          </div>
          <div className="p-6 space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">CPU Health</span>
              <span className={`text-sm font-medium ${getHealthColor(metrics.system_health?.cpu_health || 0)}`}>
                {Math.round(metrics.system_health?.cpu_health || 0)}%
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Memory Health</span>
              <span className={`text-sm font-medium ${getHealthColor(metrics.system_health?.memory_health || 0)}`}>
                {Math.round(metrics.system_health?.memory_health || 0)}%
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Disk Health</span>
              <span className={`text-sm font-medium ${getHealthColor(metrics.system_health?.disk_health || 0)}`}>
                {Math.round(metrics.system_health?.disk_health || 0)}%
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Network Health</span>
              <span className={`text-sm font-medium ${getHealthColor(metrics.system_health?.network_health || 0)}`}>
                {Math.round(metrics.system_health?.network_health || 0)}%
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Performance Bottlenecks */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Performance Bottlenecks
          </h3>
        </div>
        <div className="p-6">
          {!metrics.bottlenecks || metrics.bottlenecks.length === 0 ? (
            <p className="text-gray-500 dark:text-gray-400 text-center py-4">
              No performance bottlenecks detected
            </p>
          ) : (
            <div className="space-y-3">
              {metrics.bottlenecks.map((bottleneck, index) => (
                <div key={index} className="flex items-start space-x-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                  <div className={`px-2 py-1 rounded-full text-xs font-medium ${getSeverityColor(bottleneck.severity)}`}>
                    {bottleneck.severity}
                  </div>
                  <div className="flex-1">
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {bottleneck.description}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      Current: {bottleneck.current_value?.toFixed(1) || '0.0'} | Threshold: {bottleneck.threshold?.toFixed(1) || '0.0'}
                    </p>
                    <p className="text-xs text-blue-600 dark:text-blue-400 mt-1">
                      {bottleneck.recommendation}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Top Performance Processes */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Top Performance Processes
          </h3>
        </div>
        <div className="p-6">
          {!metrics.top_performance_processes || metrics.top_performance_processes.length === 0 ? (
            <p className="text-gray-500 dark:text-gray-400 text-center py-4">
              No process data available
            </p>
          ) : (
            <div className="space-y-3">
              {metrics.top_performance_processes.map((process, index) => (
                <div key={index} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
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
                      <span>CPU: {process.cpu_usage?.toFixed(1) || '0.0'}%</span>
                      <span>Memory: {formatBytes(process.memory_usage || 0)}</span>
                      <span className="font-medium text-green-600 dark:text-green-400">
                        Score: {process.performance_score?.toFixed(1) || '0.0'}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Optimization Recommendations */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Optimization Recommendations
          </h3>
        </div>
        <div className="p-6">
          {!metrics.recommendations || metrics.recommendations.length === 0 ? (
            <p className="text-gray-500 dark:text-gray-400 text-center py-4">
              No optimization recommendations at this time
            </p>
          ) : (
            <div className="space-y-4">
              {metrics.recommendations.map((recommendation, index) => (
                <div key={index} className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <div className="flex items-start space-x-3">
                    <Lightbulb className="w-5 h-5 text-blue-500 mt-0.5" />
                    <div className="flex-1">
                      <h4 className="font-medium text-gray-900 dark:text-gray-100">
                        {recommendation.title}
                      </h4>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                        {recommendation.description}
                      </p>
                      <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500 dark:text-gray-400">
                        <span>Priority: {recommendation.priority}</span>
                        <span>Impact: {recommendation.estimated_impact}</span>
                        <span>Difficulty: {recommendation.implementation_difficulty}</span>
                      </div>
                      {recommendation.actions && recommendation.actions.length > 0 && (
                        <div className="mt-3">
                          <p className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Actions:</p>
                          <ul className="text-xs text-gray-600 dark:text-gray-400 space-y-1">
                            {recommendation.actions.map((action, actionIndex) => (
                              <li key={actionIndex} className="flex items-start">
                                <span className="text-blue-500 mr-2">•</span>
                                {action}
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 