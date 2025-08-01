import React, { useState, useEffect } from 'react';
import { DraggableDashboard } from './DraggableDashboard';
import { SecurityDashboard } from './SecurityDashboard';
import { OptimizationDashboard } from './OptimizationDashboard';
import { SystemInfo, SystemMetrics, SecurityMetrics, OptimizationMetrics } from '../types';
import { getTauriInvoke } from '../services/tauriDetector';
import { Shield, Zap, Monitor } from 'lucide-react';

interface TabbedDashboardProps {
  systemInfo: SystemInfo | null;
  metrics: SystemMetrics | null;
}

type TabType = 'system' | 'security' | 'optimization';

export const TabbedDashboard: React.FC<TabbedDashboardProps> = ({ systemInfo, metrics }) => {
  const [activeTab, setActiveTab] = useState<TabType>('system');
  const [securityMetrics, setSecurityMetrics] = useState<SecurityMetrics | null>(null);
  const [optimizationMetrics, setOptimizationMetrics] = useState<OptimizationMetrics | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchSecurityMetrics = async () => {
    try {
      const invoke = await getTauriInvoke();
      if (invoke) {
        const data = await invoke('get_security_metrics');
        setSecurityMetrics(data);
      }
    } catch (error) {
      console.error('Failed to fetch security metrics:', error);
    }
  };

  const fetchOptimizationMetrics = async () => {
    try {
      console.log('Fetching optimization metrics...');
      const invoke = await getTauriInvoke();
      if (invoke) {
        const data = await invoke('get_optimization_metrics');
        console.log('Optimization metrics received:', data);
        setOptimizationMetrics(data);
      }
    } catch (error) {
      console.error('Failed to fetch optimization metrics:', error);
    }
  };

  useEffect(() => {
    if (activeTab === 'security' && !securityMetrics) {
      fetchSecurityMetrics();
    }
    if (activeTab === 'optimization' && !optimizationMetrics) {
      fetchOptimizationMetrics();
    }
  }, [activeTab, securityMetrics, optimizationMetrics]);

  const handleTabChange = async (tab: TabType) => {
    setActiveTab(tab);
    setLoading(true);
    
    if (tab === 'security') {
      await fetchSecurityMetrics();
    } else if (tab === 'optimization') {
      await fetchOptimizationMetrics();
    }
    
    setLoading(false);
  };

  const tabs = [
    {
      id: 'system' as TabType,
      label: 'System Monitor',
      icon: Monitor,
      description: 'Real-time system monitoring'
    },
    {
      id: 'security' as TabType,
      label: 'Security',
      icon: Shield,
      description: 'Security monitoring and threat detection'
    },
    {
      id: 'optimization' as TabType,
      label: 'Optimization',
      icon: Zap,
      description: 'Performance optimization and recommendations'
    }
  ];

  return (
    <div className="flex flex-col h-full">
      {/* Tab Navigation */}
      <div className="flex border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          
          return (
            <button
              key={tab.id}
              onClick={() => handleTabChange(tab.id)}
              className={`flex items-center px-6 py-3 text-sm font-medium border-b-2 transition-colors duration-200 ${
                isActive
                  ? 'border-blue-500 text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
              }`}
            >
              <Icon className="w-4 h-4 mr-2" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-hidden">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
            <span className="ml-2 text-gray-600 dark:text-gray-400">Loading...</span>
          </div>
        ) : (
          <>
            {activeTab === 'system' && (
              <DraggableDashboard systemInfo={systemInfo} metrics={metrics} />
            )}
            {activeTab === 'security' && (
              <SecurityDashboard metrics={securityMetrics} />
            )}
            {activeTab === 'optimization' && (
              <OptimizationDashboard metrics={optimizationMetrics} />
            )}
          </>
        )}
      </div>
    </div>
  );
}; 