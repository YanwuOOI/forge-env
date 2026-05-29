import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from 'react';
import { ToastContainer, type Toast, type ToastType } from '../../components/shared/Toast';

interface ToastContextValue {
  addToast: (type: ToastType, message: string) => string;
  success: (message: string) => string;
  error: (message: string) => string;
  warning: (message: string) => string;
  info: (message: string) => string;
  dismiss: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue>({
  addToast: () => '',
  success: () => '',
  error: () => '',
  warning: () => '',
  info: () => '',
  dismiss: () => {},
});

let counter = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const counterRef = useRef(counter);

  const addToast = useCallback((type: ToastType, message: string): string => {
    counterRef.current += 1;
    const id = `toast-${counterRef.current}`;
    const toast: Toast = { id, type, message };
    setToasts((prev) => [...prev, toast]);
    return id;
  }, []);

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const success = useCallback((msg: string) => addToast('success', msg), [addToast]);
  const error = useCallback((msg: string) => addToast('error', msg), [addToast]);
  const warning = useCallback((msg: string) => addToast('warning', msg), [addToast]);
  const info = useCallback((msg: string) => addToast('info', msg), [addToast]);

  const value = useMemo(() => ({ addToast, success, error, warning, info, dismiss }), [addToast, success, error, warning, info, dismiss]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastContainer toasts={toasts} onDismiss={dismiss} />
    </ToastContext.Provider>
  );
}

export function useToast() {
  return useContext(ToastContext);
}
