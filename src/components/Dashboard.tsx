import React from 'react';
import { SystemInfo, SystemMetrics } from '../types';
import { TabbedDashboard } from './TabbedDashboard';

interface DashboardProps {
  systemInfo: SystemInfo | null;
  metrics: SystemMetrics | null;
}

export const Dashboard: React.FC<DashboardProps> = ({ systemInfo, metrics }) => {
  return <TabbedDashboard systemInfo={systemInfo} metrics={metrics} />;
};