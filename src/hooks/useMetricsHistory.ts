import { useRef, useEffect } from 'react';

interface MetricPoint {
  time: string;
  value: number;
}

const MAX_HISTORY_POINTS = 60; // Keep last 60 data points

export function useMetricsHistory(metricName: string, currentValue: number): MetricPoint[] {
  const historyRef = useRef<MetricPoint[]>([]);
  
  useEffect(() => {
    const now = new Date();
    const timeStr = now.toLocaleTimeString('en-US', { 
      hour12: false, 
      minute: '2-digit', 
      second: '2-digit' 
    });
    
    historyRef.current = [
      ...historyRef.current,
      { time: timeStr, value: currentValue }
    ].slice(-MAX_HISTORY_POINTS);
  }, [currentValue]);
  
  return historyRef.current;
}