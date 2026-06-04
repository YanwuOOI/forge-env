import { startTransition } from 'react';
import type { NavKey } from '../../lib/types';
import { navItems, panelClass, insetClass, cardClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';
import { MetricTile } from '../shared/MetricTile';
import { NavGlyph } from '../shared/NavGlyph';
import { ThemeToggle } from '../shared/ThemeToggle';

interface SidebarProps {
  activeView: NavKey;
  setActiveView: (view: NavKey) => void;
  hostsCount: number;
  totalInstalledRuntimes: number;
  busyLabel: string | null;
  theme: 'light' | 'dark';
  onToggleTheme: () => void;
}

export function Sidebar({ activeView, setActiveView, hostsCount, totalInstalledRuntimes, busyLabel, theme, onToggleTheme }: SidebarProps) {
  const { t } = useI18n();

  return (
    <aside className={`${panelClass} flex flex-col gap-4 p-4`}>
      <div className="rounded-[var(--radius-md)] bg-[linear-gradient(145deg,rgba(255,255,255,0.55),rgba(220,231,255,0.55))] p-4 shadow-[var(--shadow-raised-sm)]">
        <div className="flex items-center justify-between">
          <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--text-muted)]">
            {t('app.name')}
          </p>
          <ThemeToggle theme={theme} onToggle={onToggleTheme} />
        </div>
        <h1 className="mt-2 text-[28px] font-semibold leading-[1.05] text-[var(--text-primary)]">
          {t('app.tagline')}
        </h1>
        <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
          {t('app.description')}
        </p>
      </div>

      <nav className={`${insetClass} flex flex-col gap-2 p-2`} aria-label={t('nav.mainNavigation')}>
        {navItems.map((item) => {
          const active = activeView === item.key;
          return (
            <button
              key={item.key}
              type="button"
              onClick={() => {
                startTransition(() => setActiveView(item.key));
              }}
              aria-current={active ? 'page' : undefined}
              aria-pressed={active}
              className={`flex items-center gap-3 rounded-[var(--radius-md)] px-3 py-3 text-left transition-[transform,box-shadow,background-color] duration-[var(--motion-hover)] ease-[var(--ease-standard)] ${
                active
                  ? 'bg-[var(--bg-elevated)] shadow-[var(--shadow-raised-sm)]'
                  : 'hover:bg-[rgba(255,255,255,0.35)]'
              }`}
            >
              <NavGlyph name={item.key} active={active} />
              <span className="min-w-0">
                <span className="block text-[11px] uppercase tracking-[0.18em] text-[var(--text-muted)]">
                  {t(`nav.${item.key}Eyebrow`)}
                </span>
                <span className="block truncate text-[15px] font-semibold text-[var(--text-primary)]">
                  {t(`nav.${item.key}`)}
                </span>
              </span>
            </button>
          );
        })}
      </nav>

      <div className={`${cardClass} mt-auto space-y-3 p-4`}>
        <div className="flex items-center justify-between">
          <span className="text-[12px] font-semibold uppercase tracking-[0.16em] text-[var(--text-muted)]">
            {t('sidebar.liveStatus')}
          </span>
          <span className="rounded-[var(--radius-pill)] bg-[var(--accent-soft)] px-3 py-1 text-[11px] font-semibold text-[var(--accent-primary)]">
            {busyLabel ? t('sidebar.busy') : t('sidebar.ready')}
          </span>
        </div>
        <p className="text-[13px] leading-6 text-[var(--text-secondary)]">
          {busyLabel ?? t('sidebar.noTask')}
        </p>
        <div className="grid grid-cols-2 gap-3">
          <MetricTile label={t('sidebar.hosts')} value={String(hostsCount)} />
          <MetricTile label={t('sidebar.runtimes')} value={String(totalInstalledRuntimes)} />
        </div>
      </div>
    </aside>
  );
}
