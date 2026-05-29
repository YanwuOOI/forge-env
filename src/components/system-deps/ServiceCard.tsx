import type { Dispatch, SetStateAction } from 'react';
import type {
  JobRecord,
  ServiceArtifact,
  ServiceConfigState,
  ServiceState,
} from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';
import type { ServiceTaskSummary } from './serviceTaskSummary';
import { ServiceConfigEditor } from './ServiceConfigEditor';
import { ServiceBackupPanel } from './ServiceBackupPanel';

interface ServiceCardProps {
  service: ServiceState;
  config: ServiceConfigState | undefined;
  draft: { port: string; dataDir: string };
  setServiceConfigDrafts: Dispatch<SetStateAction<Record<string, { port: string; dataDir: string }>>>;
  canApplyConfig: boolean;
  canApplyAndReconcile: boolean;
  reconcileLabel: string;
  logicalBackupCapable: boolean;
  artifacts: ServiceArtifact[];
  restoreDraft: string;
  setServiceRestoreDrafts: Dispatch<SetStateAction<Record<string, string>>>;
  recentServiceJob: JobRecord | undefined;
  serviceTaskSummary: ServiceTaskSummary;
  onServiceAction: (name: string, action: 'start' | 'stop' | 'restart') => void;
  onApplyServiceConfig: (name: string, port?: number | null, dataDir?: string | null) => void;
  onApplyServiceConfigAndRestart: (name: string, port?: number | null, dataDir?: string | null) => void;
  onCreateServiceBackup: (name: string) => void;
  onCreateServiceLogicalBackup: (name: string) => void;
  onExportServiceData: (name: string) => void;
  onRestoreServiceBackup: (name: string, archivePath?: string) => void;
  onValidateServiceArtifact: (path: string) => void;
  onDeleteServiceArtifact: (path: string) => void;
}

export function ServiceCard({
  service,
  config,
  draft,
  setServiceConfigDrafts,
  canApplyConfig,
  canApplyAndReconcile,
  reconcileLabel,
  logicalBackupCapable,
  artifacts,
  restoreDraft,
  setServiceRestoreDrafts,
  recentServiceJob,
  serviceTaskSummary,
  onServiceAction,
  onApplyServiceConfig,
  onApplyServiceConfigAndRestart,
  onCreateServiceBackup,
  onCreateServiceLogicalBackup,
  onExportServiceData,
  onRestoreServiceBackup,
  onValidateServiceArtifact,
  onDeleteServiceArtifact,
}: ServiceCardProps) {
  return (
    <div className={`${insetClass} p-4`}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{service.kind}</p>
          <h4 className="mt-2 text-[17px] font-semibold text-[var(--text-primary)]">{service.name}</h4>
        </div>
        <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] ${service.health === 'good' ? 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]' : service.health === 'missing' ? 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]' : 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]'}`}>
          {service.running ? 'running' : service.installed ? 'stopped' : 'missing'}
        </span>
      </div>

      <div className="mt-4 space-y-2">
        <p className="text-[12px] text-[var(--text-secondary)]">
          {service.version ?? `${service.name} binaries not detected yet.`}
        </p>
        <p className="text-[12px] text-[var(--text-secondary)]">Manager: {service.manager}</p>
        <p className="text-[12px] text-[var(--text-secondary)]">Live port: {service.port ?? 'unknown'}</p>
        <p className="font-mono text-[11px] leading-5 text-[var(--text-muted)]">
          {service.dataDir ?? 'Data directory not available yet.'}
        </p>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        <button
          type="button"
          disabled={!service.installed || service.running}
          className={!service.installed || service.running ? buttonDisabledClass : buttonPrimaryClass}
          onClick={() => onServiceAction(service.name, 'start')}
        >
          Start
        </button>
        <button
          type="button"
          disabled={!service.installed || !service.running}
          className={!service.installed || !service.running ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onServiceAction(service.name, 'stop')}
        >
          Stop
        </button>
        <button
          type="button"
          disabled={!service.installed}
          className={!service.installed ? buttonDisabledClass : buttonSecondaryClass}
          onClick={() => onServiceAction(service.name, 'restart')}
        >
          Restart
        </button>
      </div>

      {config ? (
        <ServiceConfigEditor
          service={service}
          config={config}
          draft={draft}
          setServiceConfigDrafts={setServiceConfigDrafts}
          canApplyConfig={canApplyConfig}
          canApplyAndReconcile={canApplyAndReconcile}
          reconcileLabel={reconcileLabel}
          onApplyServiceConfig={onApplyServiceConfig}
          onApplyServiceConfigAndRestart={onApplyServiceConfigAndRestart}
        />
      ) : null}

      {['Redis', 'PostgreSQL', 'MySQL'].includes(service.name) ? (
        <ServiceBackupPanel
          service={service}
          artifacts={artifacts}
          logicalBackupCapable={logicalBackupCapable}
          restoreDraft={restoreDraft}
          setServiceRestoreDrafts={setServiceRestoreDrafts}
          onCreateServiceBackup={onCreateServiceBackup}
          onCreateServiceLogicalBackup={onCreateServiceLogicalBackup}
          onExportServiceData={onExportServiceData}
          onRestoreServiceBackup={onRestoreServiceBackup}
          onValidateServiceArtifact={onValidateServiceArtifact}
          onDeleteServiceArtifact={onDeleteServiceArtifact}
        />
      ) : null}

      <div className="mt-4 space-y-2">
        <div className={`${cardClass} px-3 py-3`}>
          <div className="flex items-center justify-between gap-3">
            <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Service state</p>
            <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold ${serviceTaskSummary.toneClass}`}>
              {serviceTaskSummary.title}
            </span>
          </div>
          <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">{serviceTaskSummary.detail}</p>
          {serviceTaskSummary.nextStep ? (
            <p className="mt-2 text-[12px] leading-5 text-[var(--text-muted)]">Next: {serviceTaskSummary.nextStep}</p>
          ) : null}
        </div>
        {recentServiceJob ? (
          <div className={`${cardClass} px-3 py-3`}>
            <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Latest task</p>
            <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">{recentServiceJob.label}</p>
            <p className="mt-1 font-mono text-[11px] text-[var(--text-muted)]">{recentServiceJob.timestamp}</p>
          </div>
        ) : null}
        {service.notes.map((note) => (
          <p key={note} className="text-[12px] leading-5 text-[var(--text-secondary)]">{note}</p>
        ))}
      </div>
    </div>
  );
}
