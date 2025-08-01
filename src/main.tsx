import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ErrorBoundary } from './ErrorBoundary';
import './index.css';

console.log('main.tsx loaded');

const root = document.getElementById('root');
if (!root) {
  console.error('Root element not found!');
} else {
  console.log('Rendering React app...');
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </React.StrictMode>,
  );
  console.log('React render called');
}