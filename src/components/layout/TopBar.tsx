import type { NavKey, HostSummary } from '../../lib/types';
import { navItems, cardClass, insetClass, buttonSecondaryClass } from '../../lib/constants';

interface TopBarProps {
  activeView: NavKey;
  activeHost: HostSummary | undefined;
  onRefresh: () => void;
}

export function TopBar({ activeView, activeHost, onRefresh }: TopBarProps) {
  return (
    <header className={`${cardClass} flex flex-col gap-4 p-4 md:flex-row md:items-center md:justify-between`}>
      <div>
        <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--text-muted)]">
          {navItems.find((item) => item.key === activeView)?.eyebrow}
        </p>
        <h2 className="mt-2 text-[22px] font-semibold text-[var(--text-primary)]">
          {navItems.find((item) => item.key === activeView)?.label}
        </h2>
        <p className="mt-2 max-w-3xl text-[13px] leading-6 text-[var(--text-secondary)]">
          Current build covers host discovery, runtime state, project inspection, mirror presets,
          system dependency checks, and host-aware export/import workflows. Runtime, mirror,
          dependency, and environment actions now call real local tools, with import plans able
          to reconstruct matching hosts from an exported bundle. .NET SDK install plus project pinning,
          and C/C++ toolchain template install plus preset hints, are now included in the runtime surface.
        </p>
      </div>

      <div className="grid gap-3 md:min-w-[360px] md:grid-cols-[minmax(0,1fr)_auto]">
        <div className={`${insetClass} flex items-center gap-3 px-4 py-3`}>
          <div className="size-2 rounded-full bg-[var(--accent-primary)]" />
          <div className="min-w-0">
            <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Active host</p>
            <p className="truncate text-[14px] font-semibold text-[var(--text-primary)]">
              {activeHost?.label ?? 'No host loaded'}
            </p>
          </div>
        </div>
        <button type="button" className={buttonSecondaryClass} onClick={onRefresh}>
          Refresh snapshot
        </button>
      </div>
    </header>
  );
}
