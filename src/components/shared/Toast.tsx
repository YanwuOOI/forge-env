import { useEffect } from 'react';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
}

interface ToastItemProps {
  toast: Toast;
  onDismiss: (id: string) => void;
}

const toneMap: Record<ToastType, string> = {
  success: 'bg-[rgba(31,157,104,0.15)] text-[var(--success)] border-[rgba(31,157,104,0.3)]',
  error: 'bg-[rgba(209,75,90,0.15)] text-[var(--danger)] border-[rgba(209,75,90,0.3)]',
  warning: 'bg-[rgba(209,138,29,0.15)] text-[var(--warning)] border-[rgba(209,138,29,0.3)]',
  info: 'bg-[rgba(47,107,255,0.12)] text-[var(--accent-primary)] border-[rgba(47,107,255,0.25)]',
};

function ToastItem({ toast, onDismiss }: ToastItemProps) {
  useEffect(() => {
    const timer = setTimeout(() => onDismiss(toast.id), 5000);
    return () => clearTimeout(timer);
  }, [toast.id, onDismiss]);

  return (
    <div
      className={`${toneMap[toast.type]} flex items-start gap-3 rounded-[var(--radius-md)] border px-4 py-3 text-[13px] shadow-[var(--shadow-raised-sm)] transition-all duration-[var(--motion-panel-enter)] ease-[var(--ease-standard)]`}
      role="alert"
    >
      <span className="mt-0.5 shrink-0">{iconFor(toast.type)}</span>
      <p className="flex-1 leading-5">{toast.message}</p>
      <button
        type="button"
        onClick={() => onDismiss(toast.id)}
        className="shrink-0 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
        aria-label="Dismiss"
      >
        ×
      </button>
    </div>
  );
}

function iconFor(type: ToastType) {
  switch (type) {
    case 'success': return '✓';
    case 'error': return '✕';
    case 'warning': return '!';
    case 'info': return 'i';
  }
}

interface ToastContainerProps {
  toasts: Toast[];
  onDismiss: (id: string) => void;
}

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex max-w-[400px] flex-col gap-2" aria-live="polite" aria-atomic="false">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onDismiss={onDismiss} />
      ))}
    </div>
  );
}
