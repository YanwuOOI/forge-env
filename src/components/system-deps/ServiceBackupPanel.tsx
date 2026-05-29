import type { Dispatch, SetStateAction } from 'react';
import type { ServiceArtifact, ServiceState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';
import { formatBytes } from '../../lib/utils';

interface ServiceBackupPanelProps {
  service: ServiceState;
  artifacts: ServiceArtifact[];
  logicalBackupCapable: boolean;
  restoreDraft: string;
  setServiceRestoreDrafts: Dispatch<SetStateAction<Record<string, string>>>;
  onCreateServiceBackup: (name: string) => void;
  onCreateServiceLogicalBackup: (name: string) => void;
  onExportServiceData: (name: string) => void;
  onRestoreServiceBackup: (name: string, archivePath?: string) => void;
  onValidateServiceArtifact: (path: string) => void;
  onDeleteServiceArtifact: (path: string) => void;
}

export function ServiceBackupPanel({
  service,
  artifacts,
  logicalBackupCapable,
  restoreDraft,
  setServiceRestoreDrafts,
  onCreateServiceBackup,
  onCreateServiceLogicalBackup,
  onExportServiceData,
  onRestoreServiceBackup,
  onValidateServiceArtifact,
  onDeleteServiceArtifact,
}: ServiceBackupPanelProps) {
  const latestBackup = artifacts.find((artifact) => artifact.kind === 'backup');
  const latestExport = artifacts.find((artifact) => artifact.kind === 'export');
  const latestLogicalBackup = artifacts.find((artifact) => artifact.kind === 'logical-backup');

  return (
    <div className={`${cardClass} mt-4 space-y-4 p-4`}>
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="text-[11px] uppercase tracking-[0.12em] text-[var(--text-muted)]">Snapshot tools</p>
          <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">
            Forge Env snapshots the raw data directory into managed `tar.gz` archives. Redis,
            PostgreSQL, and MySQL must be stopped before backup, export, or restore.
          </p>
        </div>
        <span className="rounded-[var(--radius-pill)] bg-[rgba(220,231,255,0.42)] px-3 py-1 text-[11px] font-semibold text-[var(--text-secondary)]">
          {artifacts.length} artifact{artifacts.length === 1 ? '' : 's'}
        </span>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          disabled={!service.installed || !service.dataDir || service.running}
          className={!service.installed || !service.dataDir || service.running ? buttonDisabledClass : buttonPrimaryClass}
          onClick={() => onCreateServiceBackup(service.name)}
        >
          Create backup
        </button>
        <button
          type="button"
          disabled={!service.installed || !service.dataDir || service.running}
          className={!service.installed || !service.dataDir || service.running ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onExportServiceData(service.name)}
        >
          Export data dir
        </button>
        <button
          type="button"
          disabled={!service.installed || !logicalBackupCapable}
          className={!service.installed || !logicalBackupCapable ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onCreateServiceLogicalBackup(service.name)}
        >
          Logical backup
        </button>
        <button
          type="button"
          disabled={!service.installed || service.running || !latestBackup}
          className={!service.installed || service.running || !latestBackup ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onRestoreServiceBackup(service.name)}
        >
          Restore latest
        </button>
      </div>

      <label className="grid gap-2">
        <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Restore archive path</span>
        <input
          type="text"
          value={restoreDraft}
          onChange={(event) =>
            setServiceRestoreDrafts((current) => ({
              ...current,
              [service.name]: event.target.value,
            }))
          }
          className={`${insetClass} px-3 py-3 font-mono text-[12px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
          placeholder={latestBackup?.path ?? 'Paste a managed .tar.gz archive path'}
        />
      </label>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          disabled={!service.installed || service.running || !restoreDraft.trim()}
          className={!service.installed || service.running || !restoreDraft.trim() ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onRestoreServiceBackup(service.name, restoreDraft)}
        >
          Restore from path
        </button>
      </div>

      <div className="space-y-2">
        {latestBackup ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            Latest backup: <span className="font-mono text-[11px]">{latestBackup.path}</span>
          </p>
        ) : null}
        {latestExport ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            Latest export: <span className="font-mono text-[11px]">{latestExport.path}</span>
          </p>
        ) : null}
        {latestLogicalBackup ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            Latest logical backup: <span className="font-mono text-[11px]">{latestLogicalBackup.path}</span>
          </p>
        ) : null}
        {!artifacts.length ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            No managed snapshot archives exist for this service on the selected host yet.
          </p>
        ) : null}
        {artifacts.slice(0, 3).map((artifact) => (
          <div key={`${artifact.kind}-${artifact.path}`} className="rounded-[var(--radius-md)] bg-[rgba(220,231,255,0.28)] px-3 py-3">
            <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
              {artifact.kind}
            </p>
            <p className="mt-2 font-mono text-[11px] leading-5 text-[var(--text-secondary)]">
              {artifact.path}
            </p>
            <p className="mt-2 text-[11px] text-[var(--text-muted)]">
              {artifact.createdAt ? `created ${artifact.createdAt}` : 'timestamp unavailable'}
              {artifact.sizeBytes != null ? ` · ${formatBytes(artifact.sizeBytes)}` : ''}
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              <button type="button" className={buttonSecondaryClass} onClick={() => onValidateServiceArtifact(artifact.path)}>
                Validate
              </button>
              <button type="button" className={buttonSecondaryClass} onClick={() => onDeleteServiceArtifact(artifact.path)}>
                Delete
              </button>
            </div>
          </div>
        ))}
        <p className="text-[12px] leading-5 text-[var(--warning)]">
          Snapshot archives capture raw on-disk state. Stop the service first, and only restore onto a
          host where you intend to overwrite the current local data directory.
        </p>
      </div>
    </div>
  );
}
