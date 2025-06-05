export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

export function formatPercent(value: number, decimals = 1): string {
  return `${value.toFixed(decimals)}%`;
}

export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (days > 0) {
    return `${days}d ${hours}h ${minutes}m`;
  } else if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else {
    return `${minutes}m`;
  }
}

export function formatFrequency(mhz: number): string {
  if (mhz >= 1000) {
    return `${(mhz / 1000).toFixed(2)} GHz`;
  }
  return `${mhz} MHz`;
}

export function formatBytesPerSecond(bytesPerSec: number): string {
  if (bytesPerSec === 0) return '0 B/s';
  
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  
  const i = Math.floor(Math.log(bytesPerSec) / Math.log(k));
  
  return parseFloat((bytesPerSec / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function formatTemperature(celsius: number): string {
  return `${celsius.toFixed(1)}°C`;
}

export function formatPower(watts: number): string {
  return `${watts.toFixed(1)} W`;
}

export function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num);
}

export function getStatusColor(status: string): string {
  const statusColors: Record<string, string> = {
    'Running': 'text-green-500',
    'Sleeping': 'text-blue-500',
    'Stopped': 'text-red-500',
    'Zombie': 'text-purple-500',
    'Dead': 'text-gray-500',
  };
  
  return statusColors[status] || 'text-gray-400';
}

export function getUsageColor(percent: number): string {
  if (percent >= 90) return 'text-red-500';
  if (percent >= 75) return 'text-orange-500';
  if (percent >= 50) return 'text-yellow-500';
  return 'text-green-500';
}

export function getUsageGradient(percent: number): string {
  if (percent >= 90) return 'from-red-500 to-red-600';
  if (percent >= 75) return 'from-orange-500 to-orange-600';
  if (percent >= 50) return 'from-yellow-500 to-yellow-600';
  return 'from-green-500 to-green-600';
}