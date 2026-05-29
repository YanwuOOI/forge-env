import type { EnvPlan, HostDetail } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass } from '../../lib/constants';

interface EnvPlanPanelProps {
  hostDetail: HostDetail | null;
  envPlan: EnvPlan | null;
  envTargetProfile: string;
  setEnvTargetProfile: (value: string) => void;
  onRefreshEnvPlan: () => void;
  onApplyEnvPlan: () => void;
}

export function EnvPlanPanel({
  hostDetail,
  envPlan,
  envTargetProfile,
  setEnvTargetProfile,
  onRefreshEnvPlan,
  onApplyEnvPlan,
}: EnvPlanPanelProps) {
  return (
    <div className={`${cardClass} p-5`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Environment policy</p>
      <h3 className="mt-2 text-[18px] font-semibold">Preview shell profile changes</h3>
      {envPlan ? (
        <div className="mt-5 space-y-3">
          <div className={`${insetClass} p-4`}>
            <div className="flex items-center justify-between gap-3">
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Target profile</p>
              <button type="button" className={buttonSecondaryClass} onClick={onRefreshEnvPlan}>
                Refresh plan
              </button>
            </div>
            <select
              value={envTargetProfile}
              onChange={(event) => setEnvTargetProfile(event.target.value)}
              className="mt-3 w-full appearance-none rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-elevated)] px-4 py-3 text-[14px] text-[var(--text-primary)] outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
            >
              {envPlan.availableProfiles.map((profile) => (
                <option key={profile.path} value={profile.path}>
                  {profile.path}
                  {profile.managedByForgeEnv ? ' · managed' : profile.exists ? ' · exists' : ' · new file'}
                </option>
              ))}
            </select>
            <button type="button" className={`${buttonPrimaryClass} mt-4 w-full`} onClick={onApplyEnvPlan}>
              Apply shell block
            </button>
          </div>

          <div className={`${insetClass} p-4`}>
            <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Exports</p>
            <div className="mt-3 space-y-3">
              {envPlan.variables.map((variable) => (
                <div key={variable.key} className="rounded-[var(--radius-md)] bg-[var(--bg-elevated)] p-3 shadow-[var(--shadow-raised-sm)]">
                  <p className="font-mono text-[12px] font-semibold text-[var(--text-primary)]">
                    {variable.key}={variable.value}
                  </p>
                  <p className="mt-1 text-[12px] leading-5 text-[var(--text-secondary)]">{variable.reason}</p>
                </div>
              ))}
              {!envPlan.variables.length ? (
                <p className="text-[13px] leading-6 text-[var(--text-secondary)]">
                  No managed runtime roots are ready yet, so Forge Env would not add any exports.
                </p>
              ) : null}
            </div>
          </div>

          <div className={`${insetClass} p-4`}>
            <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">PATH additions</p>
            <div className="mt-3 flex flex-wrap gap-2">
              {envPlan.pathEntries.map((entry) => (
                <span key={entry} className="rounded-[var(--radius-pill)] bg-[var(--bg-elevated)] px-3 py-1 font-mono text-[11px] text-[var(--text-secondary)] shadow-[var(--shadow-raised-sm)]">
                  {entry}
                </span>
              ))}
              {!envPlan.pathEntries.length ? (
                <span className="text-[13px] text-[var(--text-secondary)]">No PATH entries required right now.</span>
              ) : null}
            </div>
          </div>

          <div className={`${insetClass} p-4`}>
            <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Managed block preview</p>
            <textarea
              readOnly
              value={envPlan.managedBlock}
              className="mt-3 h-44 w-full resize-none rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-elevated)] p-4 font-mono text-[11px] leading-5 text-[var(--text-secondary)] outline-none"
            />
            <div className="mt-3 space-y-2">
              {envPlan.notes.map((note) => (
                <p key={note} className="text-[12px] leading-5 text-[var(--text-secondary)]">{note}</p>
              ))}
            </div>
          </div>

          {hostDetail ? (
            <div className={`${insetClass} p-4`}>
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Mirror-capable scopes</p>
              <div className="mt-3 flex flex-wrap gap-2">
                {hostDetail.mirrorsSupported.map((scope) => (
                  <span key={scope} className="rounded-[var(--radius-pill)] border border-[var(--border-soft)] px-3 py-1 text-[12px] text-[var(--text-secondary)]">
                    {scope}
                  </span>
                ))}
              </div>
            </div>
          ) : null}
        </div>
      ) : (
        <p className="mt-4 text-[13px] leading-6 text-[var(--text-secondary)]">Environment plan unavailable.</p>
      )}
    </div>
  );
}
