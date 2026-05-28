import type { Dispatch, SetStateAction } from 'react';
import type {
  EnvPlan,
  ExportBundle,
  HostDetail,
  ImportAction,
  ImportResult,
  ProxySettings,
} from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';
import { groupImportActionsByHost } from '../../lib/utils';

interface SettingsSectionProps {
  hostDetail: HostDetail | null;
  envPlan: EnvPlan | null;
  envTargetProfile: string;
  exportBundle: ExportBundle | null;
  importDraft: string;
  importResult: ImportResult | null;
  proxySettings: ProxySettings;
  setProxySettings: Dispatch<SetStateAction<ProxySettings>>;
  proxyPasswordDraft: string;
  setProxyPasswordDraft: Dispatch<SetStateAction<string>>;
  setEnvTargetProfile: (value: string) => void;
  setImportDraft: (value: string) => void;
  selectedImportActionIds: string[];
  setSelectedImportActionIds: (value: string[]) => void;
  onRefreshEnvPlan: () => void;
  onApplyEnvPlan: () => void;
  onExport: () => void;
  onImport: () => void;
  onApplyImport: () => void;
  onSaveProxySettings: () => void;
  onClearProxySettings: () => void;
  onRepairImportAction: (action: ImportAction) => void;
}

export function SettingsSection({
  hostDetail,
  envPlan,
  envTargetProfile,
  exportBundle,
  importDraft,
  importResult,
  proxySettings,
  setProxySettings,
  proxyPasswordDraft,
  setProxyPasswordDraft,
  setEnvTargetProfile,
  setImportDraft,
  selectedImportActionIds,
  setSelectedImportActionIds,
  onRefreshEnvPlan,
  onApplyEnvPlan,
  onExport,
  onImport,
  onApplyImport,
  onSaveProxySettings,
  onClearProxySettings,
  onRepairImportAction,
}: SettingsSectionProps) {
  const plannedImportActions =
    importResult?.actions.filter((action) => action.status === 'planned') ?? [];
  const plannedImportActionIds = plannedImportActions.map((action) => action.id);
  const selectedImportCount = plannedImportActions.filter((action) =>
    selectedImportActionIds.includes(action.id),
  ).length;
  const groupedImportActions = groupImportActionsByHost(importResult?.actions ?? []);

  return (
    <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
      <div className="grid gap-4">
      <div className={`${cardClass} p-5`}>
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Secure proxy profile</p>
        <h3 className="mt-2 text-[18px] font-semibold">Store proxy credentials in the system vault</h3>
        <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
          Forge Env stores host, port, and username in its managed settings file, while the password is delegated to{' '}
          {proxySettings.secureStore}.
        </p>
        <div className="mt-5 grid gap-3 md:grid-cols-2">
          <label className="flex items-center gap-3 rounded-[var(--radius-md)] bg-[rgba(220,231,255,0.38)] px-4 py-3 text-[12px] text-[var(--text-secondary)]">
            <input
              type="checkbox"
              checked={proxySettings.enabled}
              onChange={(event) =>
                setProxySettings((current) => ({
                  ...current,
                  enabled: event.target.checked,
                }))
              }
              className="size-4 rounded border border-[var(--border-soft)] accent-[var(--accent-primary)]"
            />
            <span>Enable managed proxy profile</span>
          </label>
          <div className={`${insetClass} flex items-center justify-between gap-3 px-4 py-3`}>
            <div>
              <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Credential store</p>
              <p className="mt-1 text-[13px] font-semibold text-[var(--text-primary)]">{proxySettings.secureStore}</p>
            </div>
            <span
              className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold ${
                proxySettings.passwordSaved
                  ? 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]'
                  : 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]'
              }`}
            >
              {proxySettings.passwordSaved ? 'Password saved' : 'No password saved'}
            </span>
          </div>
          <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
            <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Scheme</span>
            <select
              value={proxySettings.scheme}
              onChange={(event) =>
                setProxySettings((current) => ({ ...current, scheme: event.target.value }))
              }
              className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
            >
              <option value="http">http</option>
              <option value="https">https</option>
              <option value="socks5">socks5</option>
            </select>
          </label>
          <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
            <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Host</span>
            <input
              value={proxySettings.host}
              onChange={(event) =>
                setProxySettings((current) => ({ ...current, host: event.target.value }))
              }
              className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
              placeholder="127.0.0.1"
            />
          </label>
          <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
            <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Port</span>
            <input
              value={proxySettings.port}
              onChange={(event) =>
                setProxySettings((current) => ({ ...current, port: event.target.value }))
              }
              className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
              placeholder="7890"
            />
          </label>
          <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
            <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Username</span>
            <input
              value={proxySettings.username}
              onChange={(event) =>
                setProxySettings((current) => ({ ...current, username: event.target.value }))
              }
              className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
              placeholder="Optional proxy account"
            />
          </label>
          <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
            <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Password</span>
            <input
              type="password"
              value={proxyPasswordDraft}
              onChange={(event) => setProxyPasswordDraft(event.target.value)}
              className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
              placeholder={
                proxySettings.passwordSaved ? 'Leave blank to keep saved password' : 'Stored in system vault'
              }
            />
          </label>
        </div>
        <div className="mt-4 flex flex-wrap gap-3">
          <button type="button" className={buttonPrimaryClass} onClick={onSaveProxySettings}>
            Save proxy profile
          </button>
          <button type="button" className={buttonSecondaryClass} onClick={onClearProxySettings}>
            Clear proxy profile
          </button>
        </div>
        <div className="mt-4 space-y-2 text-[12px] leading-5 text-[var(--text-secondary)]">
          <p>Save writes the non-secret proxy profile into Forge Env settings and updates the system credential entry only when a new password is supplied.</p>
          <p>Clear removes the managed proxy profile and asks the secure store to delete the saved password for the current username.</p>
        </div>
      </div>

      <div className={`${cardClass} p-5`}>
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Export template</p>
        <h3 className="mt-2 text-[18px] font-semibold">Generate a portable environment bundle</h3>
        <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
          Runtime families, mirror presets, and per-host snapshots are serialized into a versioned JSON payload.
        </p>
        <button type="button" className={`${buttonPrimaryClass} mt-5`} onClick={onExport}>
          Generate export
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
      </div>

      <div className={`${cardClass} p-5`}>
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Import reconstruction</p>
        <h3 className="mt-2 text-[18px] font-semibold">Plan host-aware rebuild actions from a bundle</h3>
        <textarea
          value={importDraft}
          onChange={(event) => setImportDraft(event.target.value)}
          placeholder="Paste an exported template bundle here"
          className={`${insetClass} mt-5 h-64 w-full resize-none border border-transparent p-4 font-mono text-[11px] leading-5 text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)] focus:ring-2 focus:ring-[var(--focus-ring)]`}
        />
        <div className="mt-4 flex flex-wrap gap-3">
          <button type="button" className={buttonSecondaryClass} onClick={onImport}>
            Validate import
          </button>
          <button
            type="button"
            className={importResult?.readyToApply && selectedImportCount > 0 ? buttonPrimaryClass : buttonDisabledClass}
            onClick={importResult?.readyToApply && selectedImportCount > 0 ? onApplyImport : undefined}
            disabled={!importResult?.readyToApply || selectedImportCount === 0}
          >
            Apply selected actions
          </button>
          <button
            type="button"
            className={plannedImportActionIds.length ? buttonSecondaryClass : buttonDisabledClass}
            onClick={
              plannedImportActionIds.length
                ? () => setSelectedImportActionIds(plannedImportActionIds)
                : undefined
            }
            disabled={!plannedImportActionIds.length}
          >
            Select all planned
          </button>
          <button
            type="button"
            className={selectedImportCount ? buttonSecondaryClass : buttonDisabledClass}
            onClick={selectedImportCount ? () => setSelectedImportActionIds([]) : undefined}
            disabled={!selectedImportCount}
          >
            Clear selection
          </button>
        </div>

        {importResult ? (
          <div className={`${insetClass} mt-5 p-4`}>
            <p className="text-[14px] font-semibold text-[var(--text-primary)]">
              {importResult.accepted ? 'Bundle accepted' : 'Bundle rejected'}
            </p>
            <p className="mt-2 text-[12px] text-[var(--text-secondary)]">
              Hosts: {importResult.hostCount} · Runtime groups: {importResult.runtimeCount} · Planned: {importResult.plannedCount} · Applied: {importResult.appliedCount}
            </p>
            {plannedImportActions.length ? (
              <p className="mt-2 text-[12px] text-[var(--text-secondary)]">
                Selected now: {selectedImportCount} of {plannedImportActions.length} planned actions
              </p>
            ) : null}
            {!importResult.issues.length && !importResult.readyToApply && !importResult.appliedCount ? (
              <p className="mt-3 text-[12px] text-[var(--text-secondary)]">
                The bundle is valid, but there are no matching host actions to apply on this machine.
              </p>
            ) : null}
            {importResult.issues.length ? (
              <div className="mt-4 space-y-2">
                {importResult.issues.map((issue) => (
                  <div key={issue} className="rounded-[var(--radius-md)] bg-[rgba(209,75,90,0.10)] px-3 py-2 text-[12px] text-[var(--danger)]">
                    {issue}
                  </div>
                ))}
              </div>
            ) : null}
            {importResult.actions.length ? (
              <div className="mt-4 space-y-3">
                {groupedImportActions.map(([hostLabel, actions]) => {
                  const hostPlannedIds = actions
                    .filter((action) => action.status === 'planned')
                    .map((action) => action.id);
                  const hostSelectedCount = hostPlannedIds.filter((id) =>
                    selectedImportActionIds.includes(id),
                  ).length;

                  return (
                    <div key={hostLabel} className="space-y-3 rounded-[var(--radius-md)] bg-[rgba(255,255,255,0.28)] p-3">
                      <div className="flex flex-wrap items-center justify-between gap-3">
                        <div>
                          <p className="text-[13px] font-semibold text-[var(--text-primary)]">{hostLabel}</p>
                          <p className="mt-1 text-[12px] text-[var(--text-secondary)]">
                            {hostSelectedCount} selected / {hostPlannedIds.length} planned
                          </p>
                        </div>
                        <div className="flex flex-wrap gap-2">
                          <button
                            type="button"
                            className={hostPlannedIds.length ? buttonSecondaryClass : buttonDisabledClass}
                            onClick={
                              hostPlannedIds.length
                                ? () =>
                                    setSelectedImportActionIds(
                                      Array.from(new Set([...selectedImportActionIds, ...hostPlannedIds])),
                                    )
                                : undefined
                            }
                            disabled={!hostPlannedIds.length}
                          >
                            Select host
                          </button>
                          <button
                            type="button"
                            className={hostSelectedCount ? buttonSecondaryClass : buttonDisabledClass}
                            onClick={
                              hostSelectedCount
                                ? () =>
                                    setSelectedImportActionIds(
                                      selectedImportActionIds.filter((id) => !hostPlannedIds.includes(id)),
                                    )
                                : undefined
                            }
                            disabled={!hostSelectedCount}
                          >
                            Clear host
                          </button>
                        </div>
                      </div>

                      {actions.map((action) => {
                        const tone =
                          action.status === 'applied'
                            ? 'bg-[rgba(31,157,104,0.10)] text-[var(--success)]'
                            : action.status === 'failed'
                              ? 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]'
                              : action.status === 'blocked'
                                ? 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]'
                              : action.status === 'skipped'
                                ? 'bg-[rgba(127,138,154,0.16)] text-[var(--text-secondary)]'
                                : 'bg-[rgba(47,107,255,0.10)] text-[var(--accent-primary)]';
                        const checked = selectedImportActionIds.includes(action.id);
                        const canSelect = action.status === 'planned';

                        return (
                          <div key={action.id} className="rounded-[var(--radius-md)] bg-[var(--bg-elevated)] p-3 shadow-[var(--shadow-raised-sm)]">
                            <div className="flex items-start justify-between gap-3">
                              <label className={`flex items-start gap-3 ${canSelect ? 'cursor-pointer' : 'cursor-default'}`}>
                                <input
                                  type="checkbox"
                                  checked={checked}
                                  disabled={!canSelect}
                                  onChange={(event) => {
                                    if (!canSelect) return;
                                    setSelectedImportActionIds(
                                      event.target.checked
                                        ? [...selectedImportActionIds, action.id]
                                        : selectedImportActionIds.filter((id) => id !== action.id),
                                    );
                                  }}
                                  className="mt-1 size-4 rounded border border-[var(--border-soft)] accent-[var(--accent-primary)]"
                                />
                                <span>
                                  <p className="text-[13px] font-semibold text-[var(--text-primary)]">{action.label}</p>
                                  <p className="mt-1 text-[12px] text-[var(--text-secondary)]">{action.kind}</p>
                                  {action.outcomeTitle ? (
                                    <p className="mt-1 text-[12px] font-semibold text-[var(--text-primary)]">{action.outcomeTitle}</p>
                                  ) : null}
                                </span>
                              </label>
                              <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] ${tone}`}>
                                {action.status}
                              </span>
                            </div>
                            {action.reason ? (
                              <p className="mt-3 text-[12px] leading-5 text-[var(--text-secondary)]">{action.reason}</p>
                            ) : null}
                            {action.remediation ? (
                              <p className="mt-2 text-[12px] leading-5 text-[var(--warning)]">Fix suggestion: {action.remediation}</p>
                            ) : null}
                            {action.nextStep ? (
                              <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">Next step: {action.nextStep}</p>
                            ) : null}
                            {action.status === 'blocked' && action.repairAvailable && action.family ? (
                              <div className="mt-3">
                                <button type="button" className={buttonSecondaryClass} onClick={() => onRepairImportAction(action)}>
                                  Repair provider
                                </button>
                              </div>
                            ) : null}
                          </div>
                        );
                      })}
                    </div>
                  );
                })}
              </div>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  );
}
