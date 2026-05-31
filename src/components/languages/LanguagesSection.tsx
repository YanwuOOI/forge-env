import { memo } from 'react';
import type { RuntimeFamilyState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';
import { RuntimeHealthBadge } from '../shared/RuntimeHealthBadge';

interface LanguagesSectionProps {
  runtimes: RuntimeFamilyState[];
  onInstall: (family: string, version: string) => void;
  onSwitch: (family: string, version: string) => void;
  onRemove: (family: string, version: string) => void;
}

export const LanguagesSection = memo(function LanguagesSection({ runtimes, onInstall, onSwitch, onRemove }: LanguagesSectionProps) {
  const totalInstalled = runtimes.reduce((sum, r) => sum + r.installed.length, 0);

  return (
    <div className="grid gap-4">
      {totalInstalled === 0 ? (
        <div className={`${cardClass} p-6 text-center`}>
          <p className="text-[48px]" role="img" aria-hidden="true">📦</p>
          <h3 className="mt-3 text-[18px] font-semibold text-[var(--text-primary)]">No runtimes installed yet</h3>
          <p className="mt-2 text-[13px] leading-6 text-[var(--text-secondary)]">
            Install your first language runtime to get started. Forge Env supports Python, Node.js, Rust, Java, Go, .NET, PHP, Ruby, and C/C++.
          </p>
        </div>
      ) : null}
      {runtimes.map((runtime) => {
        const canInstall = runtime.capabilities.canInstall;
        const canActivate = runtime.capabilities.canActivate;
        const canRemove = runtime.capabilities.canRemove;
        const providerHint = canInstall || canActivate || canRemove
          ? null
          : runtime.providerStatus === 'inspect-only'
            ? `${runtime.family} is currently inspect-only in Forge Env. Detection is live, but install and switch flows are not wired yet.`
            : `Install ${runtime.provider} first to manage ${runtime.family} through Forge Env.`;

        return (
        <div key={runtime.family} className={`${cardClass} p-5`}>
          <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
            <div>
              <p className="text-[11px] uppercase tracking-[0.18em] text-[var(--text-muted)]">{runtime.provider}</p>
              <h3 className="mt-2 text-[18px] font-semibold">{runtime.family}</h3>
              <p className="mt-3 max-w-3xl text-[13px] leading-6 text-[var(--text-secondary)]">
                {runtime.notes[0]}
              </p>
              {providerHint ? (
                <p className="mt-3 text-[12px] leading-5 text-[var(--warning)]">{providerHint}</p>
              ) : null}
            </div>

            <div className="grid gap-2 md:grid-cols-3">
              {runtime.recommendedVersions.map((version) => (
                <button
                  key={version}
                  type="button"
                  disabled={!canInstall}
                  className={canInstall ? buttonPrimaryClass : buttonDisabledClass}
                  onClick={() => onInstall(runtime.family, version)}
                >
                  Install {version}
                </button>
              ))}
            </div>
          </div>

          <div className="mt-5 grid gap-3 xl:grid-cols-[1.2fr_0.8fr]">
            <div className={`${insetClass} p-4`}>
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Installed versions</p>
              <div className="mt-4 space-y-3">
                {runtime.installed.map((installation) => (
                  <div
                    key={installation.version}
                    className="flex flex-col gap-3 rounded-[var(--radius-md)] bg-[var(--bg-elevated)] p-4 shadow-[var(--shadow-raised-sm)] md:flex-row md:items-center md:justify-between"
                  >
                    <div>
                      <p className="font-mono text-[14px] font-semibold text-[var(--text-primary)]">
                        {installation.version}
                      </p>
                      <p className="mt-1 text-[12px] text-[var(--text-secondary)]">
                        {installation.source} · {installation.tools.join(' / ')}
                      </p>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {installation.active ? (
                        <span className="rounded-[var(--radius-pill)] bg-[rgba(31,157,104,0.12)] px-3 py-1 text-[11px] font-semibold text-[var(--success)]">
                          Active
                        </span>
                      ) : (
                        <button
                          type="button"
                          disabled={!canActivate}
                          className={canActivate ? buttonSecondaryClass : buttonDisabledClass}
                          onClick={() => onSwitch(runtime.family, installation.version)}
                        >
                          Activate
                        </button>
                      )}
                      <button
                        type="button"
                        disabled={!canRemove}
                        className={canRemove ? buttonSecondaryClass : buttonDisabledClass}
                        onClick={() => onRemove(runtime.family, installation.version)}
                      >
                        Remove
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className={`${insetClass} p-4`}>
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Policy hooks</p>
              <div className="mt-4 flex flex-wrap gap-2">
                <RuntimeHealthBadge runtime={runtime} />
                <span className="rounded-[var(--radius-pill)] border border-[var(--border-soft)] px-3 py-1 font-mono text-[11px] text-[var(--text-secondary)]">
                  {runtime.detectedBinary ?? 'binary unavailable'}
                </span>
              </div>
              <div className="mt-4 flex flex-wrap gap-2">
                {runtime.packageTools.map((tool) => (
                  <span
                    key={tool}
                    className="rounded-[var(--radius-pill)] bg-[var(--bg-elevated)] px-3 py-1 text-[12px] font-semibold text-[var(--text-secondary)] shadow-[var(--shadow-raised-sm)]"
                  >
                    {tool}
                  </span>
                ))}
              </div>
              <div className="mt-4 flex flex-wrap gap-2">
                {runtime.mirrors.map((mirror) => (
                  <span
                    key={mirror}
                    className="rounded-[var(--radius-pill)] border border-[var(--border-soft)] px-3 py-1 text-[12px] text-[var(--text-secondary)]"
                  >
                    {mirror}
                  </span>
                ))}
              </div>
            </div>
          </div>
        </div>
      )})}
    </div>
  );
});
