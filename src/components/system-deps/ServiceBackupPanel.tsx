import type { Dispatch, SetStateAction } from 'react';
import type { ServiceArtifact, ServiceState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';
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
  const { t } = useI18n();
  const latestBackup = artifacts.find((artifact) => artifact.kind === 'backup');
  const latestExport = artifacts.find((artifact) => artifact.kind === 'export');
  const latestLogicalBackup = artifacts.find((artifact) => artifact.kind === 'logical-backup');

  return (
    <div className={`${cardClass} mt-4 space-y-4 p-4`}>
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="text-[11px] uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('systemDeps.snapshotTools')}</p>
          <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">
            {t('systemDeps.snapshotDescription')}
          </p>
        </div>
        <span className="rounded-[var(--radius-pill)] bg-[rgba(220,231,255,0.42)] px-3 py-1 text-[11px] font-semibold text-[var(--text-secondary)]">
          {t('systemDeps.nArtifacts').replace('{count}', String(artifacts.length))}
        </span>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          disabled={!service.installed || !service.dataDir || service.running}
          className={!service.installed || !service.dataDir || service.running ? buttonDisabledClass : buttonPrimaryClass}
          onClick={() => onCreateServiceBackup(service.name)}
        >
          {t('systemDeps.createBackup')}
        </button>
        <button
          type="button"
          disabled={!service.installed || !service.dataDir || service.running}
          className={!service.installed || !service.dataDir || service.running ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onExportServiceData(service.name)}
        >
          {t('systemDeps.exportDataDir')}
        </button>
        <button
          type="button"
          disabled={!service.installed || !logicalBackupCapable}
          className={!service.installed || !logicalBackupCapable ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onCreateServiceLogicalBackup(service.name)}
        >
          {t('systemDeps.logicalBackup')}
        </button>
        <button
          type="button"
          disabled={!service.installed || service.running || !latestBackup}
          className={!service.installed || service.running || !latestBackup ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onRestoreServiceBackup(service.name)}
        >
          {t('systemDeps.restoreLatest')}
        </button>
      </div>

      <label className="grid gap-2">
        <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('systemDeps.restoreArchivePath')}</span>
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
          placeholder={latestBackup?.path ?? t('systemDeps.restorePathPlaceholder')}
        />
      </label>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          disabled={!service.installed || service.running || !restoreDraft.trim()}
          className={!service.installed || service.running || !restoreDraft.trim() ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onRestoreServiceBackup(service.name, restoreDraft)}
        >
          {t('systemDeps.restoreFromPath')}
        </button>
      </div>

      <div className="space-y-2">
        {latestBackup ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            {t('systemDeps.latestBackup')} <span className="font-mono text-[11px]">{latestBackup.path}</span>
          </p>
        ) : null}
        {latestExport ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            {t('systemDeps.latestExport')} <span className="font-mono text-[11px]">{latestExport.path}</span>
          </p>
        ) : null}
        {latestLogicalBackup ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            {t('systemDeps.latestLogicalBackup')} <span className="font-mono text-[11px]">{latestLogicalBackup.path}</span>
          </p>
        ) : null}
        {!artifacts.length ? (
          <p className="text-[12px] leading-5 text-[var(--text-secondary)]">
            {t('systemDeps.noArchives')}
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
              {artifact.createdAt ? t('systemDeps.createdAt').replace('{date}', artifact.createdAt) : t('systemDeps.timestampUnavailable')}
              {artifact.sizeBytes != null ? ` · ${formatBytes(artifact.sizeBytes)}` : ''}
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              <button type="button" className={buttonSecondaryClass} onClick={() => onValidateServiceArtifact(artifact.path)}>
                {t('systemDeps.validate')}
              </button>
              <button type="button" className={buttonSecondaryClass} onClick={() => onDeleteServiceArtifact(artifact.path)}>
                {t('systemDeps.delete')}
              </button>
            </div>
          </div>
        ))}
        <p className="text-[12px] leading-5 text-[var(--warning)]">
          {t('systemDeps.snapshotWarning')}
        </p>
      </div>
    </div>
  );
}
