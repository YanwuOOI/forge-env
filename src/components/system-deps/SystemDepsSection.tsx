import { memo } from 'react';
import type { Dispatch, SetStateAction } from 'react';
import type {
  JobRecord,
  ServiceArtifact,
  ServiceConfigState,
  ServiceState,
  SystemDependencyState,
} from '../../lib/types';
import { cardClass, insetClass } from '../../lib/constants';
import { DependencyGrid } from './DependencyGrid';
import { ServiceCard } from './ServiceCard';
import { summarizeServiceTask } from './serviceTaskSummary';

interface SystemDepsSectionProps {
  dependencies: SystemDependencyState[];
  services: ServiceState[];
  serviceArtifacts: ServiceArtifact[];
  jobs: JobRecord[];
  serviceConfigs: ServiceConfigState[];
  serviceConfigDrafts: Record<string, { port: string; dataDir: string }>;
  setServiceConfigDrafts: Dispatch<SetStateAction<Record<string, { port: string; dataDir: string }>>>;
  serviceRestoreDrafts: Record<string, string>;
  setServiceRestoreDrafts: Dispatch<SetStateAction<Record<string, string>>>;
  onInstallTemplate: () => void;
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

export const SystemDepsSection = memo(function SystemDepsSection({
  dependencies,
  services,
  serviceArtifacts,
  jobs,
  serviceConfigs,
  serviceConfigDrafts,
  setServiceConfigDrafts,
  serviceRestoreDrafts,
  setServiceRestoreDrafts,
  onInstallTemplate,
  onServiceAction,
  onApplyServiceConfig,
  onApplyServiceConfigAndRestart,
  onCreateServiceBackup,
  onCreateServiceLogicalBackup,
  onExportServiceData,
  onRestoreServiceBackup,
  onValidateServiceArtifact,
  onDeleteServiceArtifact,
}: SystemDepsSectionProps) {
  const configByService = new Map(serviceConfigs.map((config) => [config.serviceName, config]));
  const artifactsByService = new Map(
    services.map((service) => [
      service.name,
      serviceArtifacts.filter((artifact) => artifact.serviceName === service.name),
    ]),
  );

  return (
    <div className="grid gap-4">
      <DependencyGrid dependencies={dependencies} onInstallTemplate={onInstallTemplate} />

      <div className={`${cardClass} p-5`}>
        <div className="flex flex-col gap-3 xl:flex-row xl:items-start xl:justify-between">
          <div>
            <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Service provider MVP</p>
            <h3 className="mt-2 text-[18px] font-semibold">Data stores, queues, and edge proxy</h3>
            <p className="mt-3 max-w-3xl text-[13px] leading-6 text-[var(--text-secondary)]">
              Forge Env now tracks Redis, PostgreSQL, MySQL, MongoDB, RabbitMQ, and Nginx per host. It can
              start, stop, or restart them through the host service manager, while Redis and PostgreSQL still
              keep direct local fallbacks when no manager is available.
            </p>
          </div>
          <div className={`${insetClass} flex items-center gap-3 px-4 py-3`}>
            <span className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
              {services.filter((service) => service.running).length} running
            </span>
            <span className="rounded-[var(--radius-pill)] bg-[var(--bg-elevated)] px-3 py-1 text-[12px] font-semibold text-[var(--text-secondary)] shadow-[var(--shadow-raised-sm)]">
              {services.length} tracked
            </span>
          </div>
        </div>

        <div className="mt-5 grid gap-3 xl:grid-cols-3">
          {services.map((service) => {
            const config = configByService.get(service.name);
            const draft = serviceConfigDrafts[service.name] ?? {
              port: config?.port != null ? String(config.port) : '',
              dataDir: config?.dataDir ?? '',
            };
            const canApplyConfig =
              !!config &&
              (service.name === 'Redis' || Boolean(config.configPath)) &&
              (config.canEditPort || config.canEditDataDir);
            const canApplyAndReconcile = canApplyConfig && service.installed;
            const reconcileLabel = service.running ? 'Apply + restart' : 'Apply + start';
            const logicalBackupCapable = ['PostgreSQL', 'MySQL'].includes(service.name);
            const artifacts = artifactsByService.get(service.name) ?? [];
            const latestBackup = artifacts.find((artifact) => artifact.kind === 'backup');
            const restoreDraft = serviceRestoreDrafts[service.name] ?? latestBackup?.path ?? '';
            const recentServiceJob = jobs.find((job) =>
              job.label.toLowerCase().includes(service.name.toLowerCase()),
            );
            const serviceTaskSummary = summarizeServiceTask(service, recentServiceJob, canApplyConfig, reconcileLabel);

            return (
              <ServiceCard
                key={service.name}
                service={service}
                config={config}
                draft={draft}
                setServiceConfigDrafts={setServiceConfigDrafts}
                canApplyConfig={canApplyConfig}
                canApplyAndReconcile={canApplyAndReconcile}
                reconcileLabel={reconcileLabel}
                logicalBackupCapable={logicalBackupCapable}
                artifacts={artifacts}
                restoreDraft={restoreDraft}
                setServiceRestoreDrafts={setServiceRestoreDrafts}
                recentServiceJob={recentServiceJob}
                serviceTaskSummary={serviceTaskSummary}
                onServiceAction={onServiceAction}
                onApplyServiceConfig={onApplyServiceConfig}
                onApplyServiceConfigAndRestart={onApplyServiceConfigAndRestart}
                onCreateServiceBackup={onCreateServiceBackup}
                onCreateServiceLogicalBackup={onCreateServiceLogicalBackup}
                onExportServiceData={onExportServiceData}
                onRestoreServiceBackup={onRestoreServiceBackup}
                onValidateServiceArtifact={onValidateServiceArtifact}
                onDeleteServiceArtifact={onDeleteServiceArtifact}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
});
