import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import userEvent from '@testing-library/user-event';
import App from '../App';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// Mock data
const mockSystemInfo = {
  hostname: 'test-host',
  os_name: 'Linux',
  os_version: '5.0',
  kernel_version: '5.0.0',
  architecture: 'x86_64',
  cpu_brand: 'Intel Core i7',
  cpu_cores: 4,
  cpu_threads: 8,
  total_memory: 16 * 1024 * 1024 * 1024,
  boot_time: new Date().toISOString(),
};

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    
    // Mock Tauri API calls
    vi.mocked(invoke).mockImplementation((cmd) => {
      switch (cmd) {
        case 'get_system_info':
          return Promise.resolve(mockSystemInfo);
        case 'start_monitoring':
          return Promise.resolve();
        case 'stop_monitoring':
          return Promise.resolve();
        default:
          return Promise.reject(new Error(`Unknown command: ${cmd}`));
      }
    });
    
    // Mock event listener
    vi.mocked(listen).mockResolvedValue(() => {});
  });

  it('renders the app and loads system info', async () => {
    render(<App />);
    
    await waitFor(() => {
      expect(screen.getByText(/System Monitor/i)).toBeInTheDocument();
    });
    
    expect(invoke).toHaveBeenCalledWith('get_system_info');
    expect(invoke).toHaveBeenCalledWith('start_monitoring');
  });

  it('handles monitoring toggle', async () => {
    const user = userEvent.setup();
    render(<App />);
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByRole('button')).toBeInTheDocument();
    });
    
    // Find and click the monitoring toggle button
    const toggleButton = screen.getByRole('button', { name: /monitoring/i });
    await user.click(toggleButton);
    
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('stop_monitoring');
    });
  });

  it('displays error messages', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Test error'));
    
    render(<App />);
    
    await waitFor(() => {
      expect(screen.getByText(/Failed to load system info/i)).toBeInTheDocument();
    });
  });

  it('subscribes to system metrics events', () => {
    render(<App />);
    
    expect(listen).toHaveBeenCalledWith('system-metrics', expect.any(Function));
  });
});