import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from 'react';
import { ConfirmDialog } from '../../components/shared/ConfirmDialog';

interface ConfirmOptions {
  title?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}

interface ConfirmState {
  open: boolean;
  message: string;
  title: string;
  confirmLabel: string;
  cancelLabel: string;
  danger: boolean;
  resolve: (value: boolean) => void;
}

const ConfirmContext = createContext<(message: string, options?: ConfirmOptions) => Promise<boolean>>(
  () => Promise.resolve(true),
);

export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<ConfirmState | null>(null);
  const resolveRef = useRef<((value: boolean) => void) | null>(null);

  const confirm = useCallback((message: string, options?: ConfirmOptions): Promise<boolean> => {
    return new Promise<boolean>((resolve) => {
      resolveRef.current = resolve;
      setState({
        open: true,
        message,
        title: options?.title ?? 'Confirm action',
        confirmLabel: options?.confirmLabel ?? 'Confirm',
        cancelLabel: options?.cancelLabel ?? 'Cancel',
        danger: options?.danger ?? false,
        resolve,
      });
    });
  }, []);

  const handleConfirm = useCallback(() => {
    state?.resolve(true);
    resolveRef.current = null;
    setState(null);
  }, [state]);

  const handleCancel = useCallback(() => {
    state?.resolve(false);
    resolveRef.current = null;
    setState(null);
  }, [state]);

  const contextValue = useMemo(() => confirm, [confirm]);

  return (
    <ConfirmContext.Provider value={contextValue}>
      {children}
      <ConfirmDialog
        open={state?.open ?? false}
        title={state?.title ?? ''}
        message={state?.message ?? ''}
        confirmLabel={state?.confirmLabel}
        cancelLabel={state?.cancelLabel}
        danger={state?.danger}
        onConfirm={handleConfirm}
        onCancel={handleCancel}
      />
    </ConfirmContext.Provider>
  );
}

export function useConfirm() {
  return useContext(ConfirmContext);
}
