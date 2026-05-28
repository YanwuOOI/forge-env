import { useEffect, useRef } from 'react';
import { buttonPrimaryClass, buttonSecondaryClass } from '../../lib/constants';

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  danger = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    const handleClose = () => {
      if (open) onCancel();
    };
    dialog.addEventListener('close', handleClose);
    return () => dialog.removeEventListener('close', handleClose);
  }, [open, onCancel]);

  if (!open) return null;

  return (
    <dialog
      ref={dialogRef}
      className="m-auto max-w-[480px] rounded-[var(--radius-lg)] bg-[var(--bg-elevated)] p-0 shadow-[var(--shadow-raised-md)] backdrop:bg-[var(--scrim)]"
      onClick={(e) => {
        if (e.target === dialogRef.current) onCancel();
      }}
    >
      <div className="p-6">
        <h3 className="text-[18px] font-semibold text-[var(--text-primary)]">{title}</h3>
        <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)] whitespace-pre-wrap">{message}</p>
        <div className="mt-6 flex justify-end gap-3">
          <button type="button" className={buttonSecondaryClass} onClick={onCancel}>
            {cancelLabel}
          </button>
          <button
            type="button"
            className={danger ? dangerButtonClass : buttonPrimaryClass}
            onClick={onConfirm}
            autoFocus
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}

const dangerButtonClass =
  'rounded-[var(--radius-md)] bg-[rgba(209,75,90,0.15)] px-4 py-2 text-[13px] font-semibold text-[var(--danger)] shadow-[var(--shadow-raised-sm)] transition-[transform,box-shadow,background-color] duration-[var(--motion-hover)] ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)] active:translate-y-0 active:shadow-[var(--shadow-inset)]';
