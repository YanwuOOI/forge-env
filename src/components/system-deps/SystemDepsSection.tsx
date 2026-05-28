import type { Dispatch, SetStateAction } from 'react';
import type {
  JobRecord,
  ServiceArtifact,
  ServiceConfigState,
  ServiceState,
  SystemDependencyState,
} from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass, buttonDisabledClass } from '../../lib/constants';
import { formatBytes } from '../../lib/utils';

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

export function SystemDepsSection({
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
            const snapshotCapable = ['Redis', 'PostgreSQL', 'MySQL'].includes(service.name);
            const logicalBackupCapable = ['PostgreSQL', 'MySQL'].includes(service.name);
            const artifacts = artifactsByService.get(service.name) ?? [];
            const latestBackup = artifacts.find((artifact) => artifact.kind === 'backup');
            const latestExport = artifacts.find((artifact) => artifact.kind === 'export');
            const latestLogicalBackup = artifacts.find((artifact) => artifact.kind === 'logical-backup');
            const restoreDraft = serviceRestoreDrafts[service.name] ?? latestBackup?.path ?? '';
            const recentServiceJob = jobs.find((job) =>
              job.label.toLowerCase().includes(service.name.toLowerCase()),
            );
            const serviceTaskSummary = summarizeServiceTask(service, recentServiceJob, canApplyConfig, reconcileLabel);

            return (
              <div key={service.name} className={`${insetClass} p-4`}>
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
                ) : null}

                {snapshotCapable ? (
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
          })}
        </div>
      </div>
    </div>
  );
}

function summarizeServiceTask(
  service: ServiceState,
  recentJob: JobRecord | undefined,
  canApplyConfig: boolean,
  reconcileLabel: string,
) {
  const successTone = 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]';
  const warningTone = 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]';
  const dangerTone = 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]';

  if (!service.installed) {
    return {
      title: 'Not installed',
      detail: `${service.name} is not installed on the current host, so control and config reconciliation remain unavailable.`,
      nextStep: `Install ${service.name} through ${service.manager} before applying overrides.`,
      toneClass: dangerTone,
    };
  }

  if (recentJob) {
    if (recentJob.category === 'service' && recentJob.targetName === service.name) {
      const metadataTone =
        recentJob.status === 'failed'
          ? dangerTone
          : recentJob.outcomeTitle?.includes('Applied') ||
        recentJob.outcomeTitle?.includes('Started') ||
        recentJob.outcomeTitle?.includes('Restarted') ||
        recentJob.outcomeTitle?.includes('Healthy')
          ? successTone
          : recentJob.outcomeTitle?.includes('Not installed')
            ? dangerTone
            : warningTone;
      return {
        title: recentJob.outcomeTitle ?? 'Recent task',
        detail: recentJob.outcomeDetail ?? recentJob.label,
        nextStep: recentJob.nextStep ?? null,
        toneClass: metadataTone,
      };
    }

    const label = recentJob.label.toLowerCase();
    if (
      label.includes('config') &&
      (label.includes('restart') || label.includes('restarted') || label.includes('start service'))
    ) {
      return {
        title: 'Applied + reconciled',
        detail: 'The latest change wrote the managed config override and then reapplied the service process.',
        nextStep: `Verify the service is listening on ${service.port ?? 'the expected port'} and that client connections still succeed.`,
        toneClass: successTone,
      };
    }

    if (label.includes('config')) {
      return {
        title: 'Config staged',
        detail: 'The latest change updated the managed override block without reconciling the service process.',
        nextStep: canApplyConfig ? `Use ${reconcileLabel} when you want the new settings to take effect immediately.` : 'Reconcile the service manually if the new config should be applied now.',
        toneClass: warningTone,
      };
    }

    if (label.includes('stop')) {
      return {
        title: 'Stopped',
        detail: 'The latest service task stopped the process cleanly on this host.',
        nextStep: canApplyConfig ? `Use ${reconcileLabel.replace('restart', 'start')} to bring it back with the managed settings.` : 'Use Start when you want the service online again.',
        toneClass: warningTone,
      };
    }

    if (label.includes('restart') || label.includes('restarted')) {
      return {
        title: 'Restarted',
        detail: 'The latest service task recycled the running process on this host.',
        nextStep: `Verify connectivity on port ${service.port ?? 'the expected port'} and watch for any migration or bind errors in the service logs.`,
        toneClass: successTone,
      };
    }

    if (label.includes('start') || label.includes('started')) {
      return {
        title: 'Started',
        detail: 'The latest service task brought the service online on this host.',
        nextStep: `Check that the service is reachable on port ${service.port ?? 'the expected port'} from your local toolchain.`,
        toneClass: successTone,
      };
    }
  }

  if (service.running) {
    return {
      title: 'Healthy',
      detail: `${service.name} is running and detected through its expected local control path.`,
      nextStep: canApplyConfig ? `Use ${reconcileLabel} after changing port or path settings.` : 'Use Restart if you need to reload the current host state.',
      toneClass: successTone,
    };
  }

  return {
    title: 'Idle',
    detail: `${service.name} is installed but not currently running on this host.`,
    nextStep: canApplyConfig ? `Use ${reconcileLabel} to apply config changes and bring it online in one step.` : 'Use Start when you want the service online again.',
    toneClass: warningTone,
  };
}
