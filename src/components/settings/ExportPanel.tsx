import type { ExportBundle } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';

interface ExportPanelProps {
  exportBundle: ExportBundle | null;
  onExport: () => void;
}

export function ExportPanel({ exportBundle, onExport }: ExportPanelProps) {
  const { t } = useI18n();

  return (
    <div className={`${cardClass} p-5`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('settings.exportTemplate')}</p>
      <h3 className="mt-2 text-[18px] font-semibold">{t('settings.generatePortable')}</h3>
      <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
        {t('settings.exportDescription')}
      </p>
      <button type="button" className={`${buttonPrimaryClass} mt-5`} onClick={onExport}>
        {t('settings.generateExport')}
      </button>

      {exportBundle ? (
        <div className={`${insetClass} mt-5 p-4`}>
          <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            {exportBundle.fileName}
          </p>
          <textarea
            readOnly
            value={exportBundle.payload}
            className="mt-3 h-64 w-full resize-none rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-elevated)] p-4 font-mono text-[11px] leading-5 text-[var(--text-secondary)] outline-none"
          />
        </div>
      ) : null}
    </div>
  );
}
