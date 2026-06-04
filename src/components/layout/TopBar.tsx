import type { NavKey, HostSummary } from '../../lib/types';
import { cardClass, insetClass, buttonSecondaryClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';

interface TopBarProps {
  activeView: NavKey;
  activeHost: HostSummary | undefined;
  onRefresh: () => void;
}

export function TopBar({ activeView, activeHost, onRefresh }: TopBarProps) {
  const { t } = useI18n();

  return (
    <header className={`${cardClass} flex flex-col gap-4 p-4 md:flex-row md:items-center md:justify-between`}>
      <div>
        <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--text-muted)]">
          {t(`nav.${activeView}Eyebrow`)}
        </p>
        <h2 className="mt-2 text-[22px] font-semibold text-[var(--text-primary)]">
          {t(`nav.${activeView}`)}
        </h2>
        <p className="mt-2 max-w-3xl text-[13px] leading-6 text-[var(--text-secondary)]">
          {t('topbar.description')}
        </p>
      </div>

      <div className="grid gap-3 md:min-w-[360px] md:grid-cols-[minmax(0,1fr)_auto]">
        <div className={`${insetClass} flex items-center gap-3 px-4 py-3`}>
          <div className="size-2 rounded-full bg-[var(--accent-primary)]" />
          <div className="min-w-0">
            <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('topbar.activeHost')}</p>
            <p className="truncate text-[14px] font-semibold text-[var(--text-primary)]">
              {activeHost?.label ?? t('topbar.noHost')}
            </p>
          </div>
        </div>
        <button type="button" className={buttonSecondaryClass} onClick={onRefresh}>
          {t('topbar.refresh')}
        </button>
      </div>
    </header>
  );
}
