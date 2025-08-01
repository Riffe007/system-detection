// Alternative Tauri detection service for Tauri v2

export async function detectTauriEnvironment(): Promise<boolean> {
  console.log('=== Tauri v2 Detection Service ===');
  
  // Method 1: Check for Tauri v2 global API structure
  if (typeof window !== 'undefined' && window.__TAURI__) {
    console.log('✓ window.__TAURI__ found');
    console.log('  Available APIs:', Object.keys(window.__TAURI__));
    
    // In Tauri v2, the structure is window.__TAURI__.core.invoke
    if (window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
      console.log('✓ Tauri v2 core.invoke found');
      return true;
    }
    
    // Legacy structure check
    if (window.__TAURI__.tauri && typeof window.__TAURI__.tauri.invoke === 'function') {
      console.log('✓ Legacy Tauri invoke found');
      return true;
    }
  }
  
  // Method 2: Try to import Tauri modules
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    console.log('✓ Method 2: @tauri-apps/api/core imported successfully');
    
    // Try a simple command
    await invoke('get_system_info');
    console.log('✓ Method 2: Successfully called Tauri command');
    return true;
  } catch (e) {
    console.log('✗ Method 2 failed:', e);
  }
  
  // Method 3: Check for Tauri-specific globals
  const globalAny = globalThis as any;
  if (globalAny.__TAURI__ || globalAny.__TAURI_INVOKE__ || globalAny.__TAURI_INTERNALS__) {
    console.log('✓ Method 3: Tauri globals found');
    return true;
  }
  
  // Method 4: Environment detection
  if (typeof window !== 'undefined') {
    const searchParams = new URLSearchParams(window.location.search);
    if (searchParams.has('__TAURI__')) {
      console.log('✓ Method 4: Tauri URL parameter found');
      return true;
    }
  }
  
  console.log('✗ No Tauri environment detected');
  return false;
}

export async function getTauriInvoke() {
  // Try Tauri v2 structure first
  if (window.__TAURI__?.core?.invoke) {
    console.log('Using Tauri v2 core.invoke');
    return window.__TAURI__.core.invoke;
  }
  
  // Try legacy structure
  if (window.__TAURI__?.tauri?.invoke) {
    console.log('Using legacy tauri.invoke');
    return window.__TAURI__.tauri.invoke;
  }
  
  // Try import
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    console.log('Using imported invoke from @tauri-apps/api/core');
    return invoke;
  } catch (e) {
    console.error('Failed to get Tauri invoke:', e);
    return null;
  }
}

export async function getTauriListen() {
  // Try Tauri v2 structure
  if (window.__TAURI__?.event?.listen) {
    console.log('Using Tauri v2 event.listen');
    return window.__TAURI__.event.listen;
  }
  
  // Try import
  try {
    const { listen } = await import('@tauri-apps/api/event');
    console.log('Using imported listen from @tauri-apps/api/event');
    return listen;
  } catch (e) {
    console.error('Failed to get Tauri listen:', e);
    return null;
  }
}