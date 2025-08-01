import { useEffect, useRef, useState } from 'react';
import { SystemMetrics } from '../types';

/**
 * Throttles metric updates to prevent overwhelming the UI on high-performance systems
 * @param metrics The raw metrics from the backend
 * @param throttleMs Minimum time between updates (default: 100ms)
 * @returns Throttled metrics
 */
export function useThrottledMetrics(
  metrics: SystemMetrics | null,
  throttleMs: number = 100
): SystemMetrics | null {
  const [throttledMetrics, setThrottledMetrics] = useState<SystemMetrics | null>(metrics);
  const lastUpdateRef = useRef<number>(0);
  const pendingUpdateRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (!metrics) {
      setThrottledMetrics(null);
      return;
    }

    const now = Date.now();
    const timeSinceLastUpdate = now - lastUpdateRef.current;

    // Clear any pending update
    if (pendingUpdateRef.current) {
      clearTimeout(pendingUpdateRef.current);
    }

    if (timeSinceLastUpdate >= throttleMs) {
      // Update immediately if enough time has passed
      setThrottledMetrics(metrics);
      lastUpdateRef.current = now;
    } else {
      // Schedule update for later
      const delay = throttleMs - timeSinceLastUpdate;
      pendingUpdateRef.current = setTimeout(() => {
        setThrottledMetrics(metrics);
        lastUpdateRef.current = Date.now();
      }, delay);
    }

    return () => {
      if (pendingUpdateRef.current) {
        clearTimeout(pendingUpdateRef.current);
      }
    };
  }, [metrics, throttleMs]);

  return throttledMetrics;
}