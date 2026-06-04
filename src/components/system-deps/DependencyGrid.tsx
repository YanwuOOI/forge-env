import type { SystemDependencyState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';

interface DependencyGridProps {
  dependencies: SystemDependencyState[];
  onInstallTemplate: () => void;
}

export function DependencyGrid({ dependencies, onInstallTemplate }: DependencyGridProps) {
  const { t } = useI18n();

  return (
    <div className="grid gap-4 xl:grid-cols-[1fr_0.9fr]">
      <div className={`${cardClass} p-5`}>
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('deps.baseLayer')}</p>
        <h3 className="mt-2 text-[18px] font-semibold">{t('deps.oneClickTemplate')}</h3>
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
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('systemDeps.execution')}</p>
        <h3 className="mt-2 text-[18px] font-semibold">{t('systemDeps.templateEntry')}</h3>
        <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
          {t('systemDeps.templateDesc')}
        </p>
        <button type="button" className={`${buttonPrimaryClass} mt-5`} onClick={onInstallTemplate}>
          {t('systemDeps.installBase')}
        </button>
      </div>
    </div>
  );
}
