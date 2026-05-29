import type { SystemDependencyState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass } from '../../lib/constants';

interface DependencyGridProps {
  dependencies: SystemDependencyState[];
  onInstallTemplate: () => void;
}

export function DependencyGrid({ dependencies, onInstallTemplate }: DependencyGridProps) {
  return (
    <div className="grid gap-4 xl:grid-cols-[1fr_0.9fr]">
      <div className={`${cardClass} p-5`}>
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Base layer</p>
        <h3 className="mt-2 text-[18px] font-semibold">One-click dependency template</h3>
        <div className="mt-5 grid gap-3 md:grid-cols-2">
          {dependencies.map((dependency) => (
            <div key={dependency.name} className={`${insetClass} px-4 py-4`}>
              <div className="flex items-center justify-between gap-3">
                <p className="text-[14px] font-semibold text-[var(--text-primary)]">{dependency.name}</p>
                <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold ${dependency.installed ? 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]' : 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]'}`}>
                  {dependency.installed ? 'Installed' : 'Missing'}
                </span>
              </div>
              <p className="mt-2 font-mono text-[11px] text-[var(--text-muted)]">{dependency.command}</p>
              <p className="mt-2 text-[12px] text-[var(--text-secondary)]">
                {dependency.version ?? dependency.sourceHint}
              </p>
              <p className="mt-1 text-[11px] leading-5 text-[var(--text-muted)]">{dependency.sourceHint}</p>
            </div>
          ))}
        </div>
      </div>

      <div className={`${cardClass} p-5`}>
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Execution</p>
        <h3 className="mt-2 text-[18px] font-semibold">Template installer entry point</h3>
        <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
          This entry point now calls the detected package manager directly. It does not try to elevate
          privileges for you, so native package manager permission rules still apply.
        </p>
        <button type="button" className={`${buttonPrimaryClass} mt-5`} onClick={onInstallTemplate}>
          Install base template
        </button>
      </div>
    </div>
  );
}
