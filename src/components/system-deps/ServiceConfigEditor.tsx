import type { Dispatch, SetStateAction } from 'react';
import type { ServiceConfigState, ServiceState } from '../../lib/types';
import { cardClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';

interface ServiceConfigEditorProps {
  service: ServiceState;
  config: ServiceConfigState;
  draft: { port: string; dataDir: string };
  setServiceConfigDrafts: Dispatch<SetStateAction<Record<string, { port: string; dataDir: string }>>>;
  canApplyConfig: boolean;
  canApplyAndReconcile: boolean;
  reconcileLabel: string;
  onApplyServiceConfig: (name: string, port?: number | null, dataDir?: string | null) => void;
  onApplyServiceConfigAndRestart: (name: string, port?: number | null, dataDir?: string | null) => void;
}

export function ServiceConfigEditor({
  service,
  config,
  draft,
  setServiceConfigDrafts,
  canApplyConfig,
  canApplyAndReconcile,
  reconcileLabel,
  onApplyServiceConfig,
  onApplyServiceConfigAndRestart,
}: ServiceConfigEditorProps) {
  return (
    <div className={`${cardClass} mt-4 space-y-4 p-4`}>
      <div>
        <p className="text-[11px] uppercase tracking-[0.12em] text-[var(--text-muted)]">Managed config</p>
        <p className="mt-2 font-mono text-[11px] leading-5 text-[var(--text-secondary)]">
          {config.configPath ?? 'No writable config path detected yet.'}
        </p>
      </div>

      <div className="grid gap-3">
        <label className="grid gap-2">
          <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Override port</span>
          <input
            type="number"
            inputMode="numeric"
            value={draft.port}
            disabled={!config.canEditPort}
            onChange={(event) =>
              setServiceConfigDrafts((current) => ({
                ...current,
                [service.name]: {
                  port: event.target.value,
                  dataDir: current[service.name]?.dataDir ?? draft.dataDir,
                },
              }))
            }
            className={`rounded-[var(--radius-md)] border border-[var(--border-soft)] px-4 py-3 text-[14px] outline-none focus:ring-2 focus:ring-[var(--focus-ring)] ${
              config.canEditPort
                ? 'bg-[var(--bg-elevated)] text-[var(--text-primary)]'
                : 'bg-[rgba(127,138,154,0.12)] text-[var(--text-muted)]'
            }`}
          />
        </label>

        <label className="grid gap-2">
          <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Data directory</span>
          <input
            type="text"
            value={draft.dataDir}
            disabled={!config.canEditDataDir}
            onChange={(event) =>
              setServiceConfigDrafts((current) => ({
                ...current,
                [service.name]: {
                  port: current[service.name]?.port ?? draft.port,
                  dataDir: event.target.value,
                },
              }))
            }
            className={`rounded-[var(--radius-md)] border border-[var(--border-soft)] px-4 py-3 text-[13px] outline-none focus:ring-2 focus:ring-[var(--focus-ring)] ${
              config.canEditDataDir
                ? 'bg-[var(--bg-elevated)] font-mono text-[var(--text-primary)]'
                : 'bg-[rgba(127,138,154,0.12)] font-mono text-[var(--text-muted)]'
            }`}
          />
        </label>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          disabled={!canApplyConfig}
          className={!canApplyConfig ? buttonDisabledClass : buttonPrimaryClass}
          onClick={() =>
            onApplyServiceConfig(
              service.name,
              draft.port.trim().length ? Number(draft.port) : null,
              draft.dataDir.trim().length ? draft.dataDir.trim() : null,
            )
          }
        >
          Apply config
        </button>
        <button
          type="button"
          disabled={!canApplyAndReconcile}
          className={!canApplyAndReconcile ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() =>
            onApplyServiceConfigAndRestart(
              service.name,
              draft.port.trim().length ? Number(draft.port) : null,
              draft.dataDir.trim().length ? draft.dataDir.trim() : null,
            )
          }
        >
          {reconcileLabel}
        </button>
        <button
          type="button"
          className={buttonSecondaryClass}
          onClick={() =>
            setServiceConfigDrafts((current) => ({
              ...current,
              [service.name]: {
                port: config.port != null ? String(config.port) : '',
                dataDir: config.dataDir ?? '',
              },
            }))
          }
        >
          Reset fields
        </button>
      </div>

      <div className="space-y-2">
        {canApplyConfig ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            Use `Apply config` to stage changes only, or `{reconcileLabel}` to write the
            managed block and immediately {service.running ? 'restart the current process' : 'start the service with the new settings'}.
          </p>
        ) : null}
        {config.notes.map((note) => (
          <p key={note} className="text-[12px] leading-5 text-[var(--text-secondary)]">
            {note}
          </p>
        ))}
      </div>
    </div>
  );
}
