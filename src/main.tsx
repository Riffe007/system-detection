import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ErrorBoundary } from './ErrorBoundary';
import './index.css';

console.log('=== main.tsx loaded ===');
console.log('Document ready state:', document.readyState);
console.log('Window location:', window.location.href);
console.log('Window __TAURI__:', window.__TAURI__);
console.log('Document title:', document.title);

const root = document.getElementById('root');
if (!root) {
  console.error('Root element not found!');
} else {
  console.log('Rendering React app...');
  console.log('Root element:', root);
  console.log('App component:', App);
  
  try {
    ReactDOM.createRoot(root).render(
      <React.StrictMode>
        <ErrorBoundary>
          <App />
        </ErrorBoundary>
      </React.StrictMode>,
    );
    console.log('React render called successfully');
  } catch (error) {
    console.error('React render failed:', error);
  }
}