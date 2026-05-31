import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ConfirmProvider } from './lib/hooks/useConfirm';
import { ToastProvider } from './lib/hooks/useToast';
import { loadTranslations } from './lib/i18n';
import en from './locales/en.json';
import zhCN from './locales/zh-CN.json';
import './styles/index.css';

loadTranslations('en', en);
loadTranslations('zh-CN', zhCN);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ToastProvider>
      <ConfirmProvider>
        <App />
      </ConfirmProvider>
    </ToastProvider>
  </React.StrictMode>,
);
