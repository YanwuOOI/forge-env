import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  startTransition,
  type Dispatch,
  type SetStateAction,
  useDeferredValue,
  useEffect,
  useEffectEvent,
  useState,
} from 'react';
import { api } from './lib/api';
import type {
  EnvPlan,
  ExportBundle,
  HostDetail,
  HostSummary,
  ImportAction,
  ImportResult,
  JobRecord,
  JobStatus,
  NavKey,
  ProjectProfile,
  ProjectRuntimePolicyOptions,
  ProxySettings,
  RuntimeFamilyState,
  ServiceArtifact,
  ServiceConfigState,
  ServiceState,
  SystemDependencyState,
} from './lib/types';

const navItems: { key: NavKey; label: string; eyebrow: string }[] = [
  { key: 'overview', label: 'Overview', eyebrow: 'Command center' },
  { key: 'hosts', label: 'Hosts', eyebrow: 'Machine topology' },
  { key: 'languages', label: 'Languages', eyebrow: 'Runtime layers' },
  { key: 'projects', label: 'Projects', eyebrow: 'Project detection' },
  { key: 'deps', label: 'System Deps', eyebrow: 'Base toolchain' },
  { key: 'settings', label: 'Settings', eyebrow: 'Mirrors and export' },
];

const baseDependencies = [
  'Git',
  'SSH',
  'OpenSSL',
  'curl',
  'wget',
  'CMake',
  'GCC',
  'Clang',
  'pkg-config',
  'FFmpeg',
];

const panelClass =
  'rounded-[var(--radius-lg)] bg-[var(--bg-panel)] shadow-[var(--shadow-raised-md)]';
const cardClass =
  'rounded-[var(--radius-md)] bg-[var(--bg-elevated)] shadow-[var(--shadow-raised-sm)]';
const insetClass =
  'rounded-[var(--radius-md)] bg-[var(--bg-inset)] shadow-[var(--shadow-inset)]';
const buttonPrimaryClass =
  'rounded-[var(--radius-md)] bg-[var(--accent-soft)] px-4 py-2 text-[13px] font-semibold text-[var(--accent-primary)] shadow-[var(--shadow-raised-sm)] transition-[transform,box-shadow,background-color] duration-[var(--motion-hover)] ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)] active:translate-y-0 active:shadow-[var(--shadow-inset)]';
const buttonSecondaryClass =
  'rounded-[var(--radius-md)] bg-[var(--bg-elevated)] px-4 py-2 text-[13px] font-semibold text-[var(--text-primary)] shadow-[var(--shadow-raised-sm)] transition-[transform,box-shadow] duration-[var(--motion-hover)] ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)] active:translate-y-0 active:shadow-[var(--shadow-inset)]';
const buttonDisabledClass =
  'cursor-not-allowed rounded-[var(--radius-md)] bg-[rgba(127,138,154,0.18)] px-4 py-2 text-[13px] font-semibold text-[var(--text-muted)] shadow-none opacity-70';

type JobFilterMode = 'relevant' | 'all';
type ProjectPolicyDraftMap = Record<string, ProjectRuntimePolicyOptions>;

function confirmMutation(message: string) {
  if (typeof window === 'undefined' || typeof window.confirm !== 'function') {
    return true;
  }

  return window.confirm(message);
}

function App() {
  const [activeView, setActiveView] = useState<NavKey>('overview');
  const [hosts, setHosts] = useState<HostSummary[]>([]);
  const [runtimes, setRuntimes] = useState<RuntimeFamilyState[]>([]);
  const [projects, setProjects] = useState<ProjectProfile[]>([]);
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [hostDetail, setHostDetail] = useState<HostDetail | null>(null);
  const [selectedHostId, setSelectedHostId] = useState('');
  const [dependencies, setDependencies] = useState<SystemDependencyState[]>([]);
  const [services, setServices] = useState<ServiceState[]>([]);
  const [serviceArtifacts, setServiceArtifacts] = useState<ServiceArtifact[]>([]);
  const [serviceConfigs, setServiceConfigs] = useState<ServiceConfigState[]>([]);
  const [serviceConfigDrafts, setServiceConfigDrafts] = useState<Record<string, { port: string; dataDir: string }>>({});
  const [serviceRestoreDrafts, setServiceRestoreDrafts] = useState<Record<string, string>>({});
  const [projectQuery, setProjectQuery] = useState('');
  const [mirrorPreset, setMirrorPreset] = useState('Tsinghua');
  const [proxySettings, setProxySettings] = useState<ProxySettings>({
    enabled: false,
    scheme: 'http',
    host: '',
    port: '',
    username: '',
    passwordSaved: false,
    secureStore: 'Keychain',
  });
  const [proxyPasswordDraft, setProxyPasswordDraft] = useState('');
  const [loading, setLoading] = useState(true);
  const [busyLabel, setBusyLabel] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [exportBundle, setExportBundle] = useState<ExportBundle | null>(null);
  const [importDraft, setImportDraft] = useState('');
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [selectedImportActionIds, setSelectedImportActionIds] = useState<string[]>([]);
  const [envPlan, setEnvPlan] = useState<EnvPlan | null>(null);
  const [envTargetProfile, setEnvTargetProfile] = useState('');
  const [jobFilterMode, setJobFilterMode] = useState<JobFilterMode>('relevant');

  const deferredProjectQuery = useDeferredValue(projectQuery);
  const filteredProjects = projects.filter((project) => {
    const token = deferredProjectQuery.trim().toLowerCase();
    if (!token) {
      return true;
    }

    return [project.name, project.path, ...project.markers].join(' ').toLowerCase().includes(token);
  });

  const refreshAll = useEffectEvent(async () => {
    setLoading(true);
    setError(null);

    try {
      const [
        nextHosts,
        nextProjects,
        nextJobs,
        nextPreferences,
        nextProxySettings,
      ] = await Promise.all([
        api.hostsList(),
        api.projectsInspect(),
        api.jobsSubscribe(),
        api.appPreferences(),
        api.proxySettingsLoad(),
      ]);
      const persistedHostId =
        nextPreferences.selectedHostId &&
        nextHosts.some((host) => host.id === nextPreferences.selectedHostId)
          ? nextPreferences.selectedHostId
          : '';
      const preferredHostId =
        (selectedHostId && nextHosts.some((host) => host.id === selectedHostId) && selectedHostId) ||
        persistedHostId ||
        nextHosts[0]?.id ||
        '';
      const [nextHostDetail, nextEnvPlan, nextRuntimes, nextDependencies, nextServices, nextServiceArtifacts, nextServiceConfigs] = await Promise.all([
        preferredHostId ? api.hostInspect(preferredHostId) : Promise.resolve(null),
        api.envPreview(preferredHostId || 'native'),
        api.runtimesList(preferredHostId || 'native'),
        api.depsList(preferredHostId || 'native'),
        api.servicesList(preferredHostId || 'native'),
        api.serviceArtifactsList(preferredHostId || 'native'),
        api.serviceConfigsList(preferredHostId || 'native'),
      ]);

      setHosts(nextHosts);
      setRuntimes(nextRuntimes);
      setProjects(nextProjects);
      setJobs(nextJobs);
      setHostDetail(nextHostDetail);
      setSelectedHostId(preferredHostId);
      setDependencies(nextDependencies);
      setServices(nextServices);
      setServiceArtifacts(nextServiceArtifacts);
      setServiceConfigs(nextServiceConfigs);
      setServiceConfigDrafts(draftsFromServiceConfigs(nextServiceConfigs));
      setServiceRestoreDrafts(draftsFromServiceArtifacts(nextServices, nextServiceArtifacts));
      setEnvPlan(nextEnvPlan);
      setEnvTargetProfile(nextEnvPlan.targetProfile);
      setMirrorPreset(nextPreferences.appliedMirrorPreset ?? 'Tsinghua');
      setProxySettings(nextProxySettings);
      setProxyPasswordDraft('');
    } catch (refreshError) {
      setError(refreshError instanceof Error ? refreshError.message : 'Failed to load workspace data.');
    } finally {
      setLoading(false);
    }
  });

  useEffect(() => {
    void refreshAll();
  }, [refreshAll]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let cleanup: (() => void) | undefined;

    void listen<JobRecord[]>('jobs://updated', (event) => {
      setJobs(event.payload);
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);

  const runAction = useEffectEvent(async (label: string, action: () => Promise<unknown>) => {
    setBusyLabel(label);
    setError(null);

    try {
      await action();
      await refreshAll();
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : `${label} failed.`);
    } finally {
      setBusyLabel(null);
    }
  });

  const activeHost = hosts.find((host) => host.id === selectedHostId) ?? hosts[0];
  const pendingJobs = jobs.filter((job) => job.status === 'queued' || job.status === 'running');
  const visibleJobs = filterJobsForView(jobs, activeView, jobFilterMode);
  const completedJobs = visibleJobs.filter((job) => job.status === 'completed').slice(0, 4);
  const totalInstalledRuntimes = runtimes.reduce((sum, runtime) => sum + runtime.installed.length, 0);
  const relevantJobCount = filterJobsForView(jobs, activeView, 'relevant').length;

  return (
    <div className="min-h-dvh bg-[var(--bg-canvas)] p-4 text-[var(--text-primary)] md:p-6">
      <div className="mx-auto grid min-h-[calc(100dvh-2rem)] max-w-[1600px] gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
        <aside className={`${panelClass} flex flex-col gap-4 p-4`}>
          <div className="rounded-[var(--radius-md)] bg-[linear-gradient(145deg,rgba(255,255,255,0.55),rgba(220,231,255,0.55))] p-4 shadow-[var(--shadow-raised-sm)]">
            <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--text-muted)]">
              Forge Env
            </p>
            <h1 className="mt-2 text-[28px] font-semibold leading-[1.05] text-[var(--text-primary)]">
              Soft industrial control room for runtimes.
            </h1>
            <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
              A Tauri shell for language versions, mirrors, PATH policy, and project detection.
            </p>
          </div>

          <nav className={`${insetClass} flex flex-col gap-2 p-2`}>
            {navItems.map((item) => {
              const active = activeView === item.key;
              return (
                <button
                  key={item.key}
                  type="button"
                  onClick={() => {
                    startTransition(() => setActiveView(item.key));
                  }}
                  aria-current={active ? 'page' : undefined}
                  aria-pressed={active}
                  className={`flex items-center gap-3 rounded-[var(--radius-md)] px-3 py-3 text-left transition-[transform,box-shadow,background-color] duration-[var(--motion-hover)] ease-[var(--ease-standard)] ${
                    active
                      ? 'bg-[var(--bg-elevated)] shadow-[var(--shadow-raised-sm)]'
                      : 'hover:bg-[rgba(255,255,255,0.35)]'
                  }`}
                >
                  <NavGlyph name={item.key} active={active} />
                  <span className="min-w-0">
                    <span className="block text-[11px] uppercase tracking-[0.18em] text-[var(--text-muted)]">
                      {item.eyebrow}
                    </span>
                    <span className="block truncate text-[15px] font-semibold text-[var(--text-primary)]">
                      {item.label}
                    </span>
                  </span>
                </button>
              );
            })}
          </nav>

          <div className={`${cardClass} mt-auto space-y-3 p-4`}>
            <div className="flex items-center justify-between">
              <span className="text-[12px] font-semibold uppercase tracking-[0.16em] text-[var(--text-muted)]">
                Live Status
              </span>
              <span className="rounded-[var(--radius-pill)] bg-[var(--accent-soft)] px-3 py-1 text-[11px] font-semibold text-[var(--accent-primary)]">
                {busyLabel ? 'Busy' : 'Ready'}
              </span>
            </div>
            <p className="text-[13px] leading-6 text-[var(--text-secondary)]">
              {busyLabel ??
                'No running task. Runtime, mirror, and dependency actions require confirmation before they mutate the host.'}
            </p>
            <div className="grid grid-cols-2 gap-3">
              <MetricTile label="Hosts" value={String(hosts.length)} />
              <MetricTile label="Runtimes" value={String(totalInstalledRuntimes)} />
            </div>
          </div>
        </aside>

        <main className={`${panelClass} flex min-w-0 flex-col p-4 md:p-5`}>
          <header className={`${cardClass} flex flex-col gap-4 p-4 md:flex-row md:items-center md:justify-between`}>
            <div>
              <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--text-muted)]">
                {navItems.find((item) => item.key === activeView)?.eyebrow}
              </p>
              <h2 className="mt-2 text-[22px] font-semibold text-[var(--text-primary)]">
                {navItems.find((item) => item.key === activeView)?.label}
              </h2>
              <p className="mt-2 max-w-3xl text-[13px] leading-6 text-[var(--text-secondary)]">
                Current build covers host discovery, runtime state, project inspection, mirror presets,
                system dependency checks, and host-aware export/import workflows. Runtime, mirror,
                dependency, and environment actions now call real local tools, with import plans able
                to reconstruct matching hosts from an exported bundle. .NET SDK install plus project pinning,
                and C/C++ toolchain template install plus preset hints, are now included in the runtime surface.
              </p>
            </div>

            <div className="grid gap-3 md:min-w-[360px] md:grid-cols-[minmax(0,1fr)_auto]">
              <div className={`${insetClass} flex items-center gap-3 px-4 py-3`}>
                <div className="size-2 rounded-full bg-[var(--accent-primary)]" />
                <div className="min-w-0">
                  <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Active host</p>
                  <p className="truncate text-[14px] font-semibold text-[var(--text-primary)]">
                    {activeHost?.label ?? 'No host loaded'}
                  </p>
                </div>
              </div>
              <button type="button" className={buttonSecondaryClass} onClick={() => void refreshAll()}>
                Refresh snapshot
              </button>
            </div>
          </header>

          {error ? (
            <div className="mt-4 rounded-[var(--radius-md)] bg-[rgba(209,75,90,0.12)] px-4 py-3 text-[13px] text-[var(--danger)]">
              {error}
            </div>
          ) : null}

          {loading ? (
            <div className="mt-4 grid flex-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
              {Array.from({ length: 6 }).map((_, index) => (
                <div key={index} className={`${cardClass} animate-pulse p-5`}>
                  <div className="h-4 w-24 rounded bg-[rgba(91,103,120,0.16)]" />
                  <div className="mt-4 h-8 w-40 rounded bg-[rgba(91,103,120,0.12)]" />
                  <div className="mt-6 h-24 rounded bg-[rgba(91,103,120,0.10)]" />
                </div>
              ))}
            </div>
          ) : (
            <section className="mt-4 flex-1 overflow-auto">
              {activeView === 'overview' ? (
                <OverviewSection
                  hosts={hosts}
                  runtimes={runtimes}
                  jobs={jobs}
                  onMirrorApply={() =>
                    confirmMutation(
                      `Apply the ${mirrorPreset} mirror preset to npm, pip, and Cargo config on this host?`,
                    ) &&
                    runAction(`Apply ${mirrorPreset} mirror preset`, () =>
                      api.mirrorsApply(activeHost?.id ?? 'native', mirrorPreset),
                    )
                  }
                  mirrorPreset={mirrorPreset}
                  setMirrorPreset={setMirrorPreset}
                />
              ) : null}

              {activeView === 'hosts' ? (
                <HostsSection
                  hosts={hosts}
                  detail={hostDetail}
                  selectedHostId={selectedHostId}
                  onSelectHost={(hostId) =>
                    runAction(`Inspect ${hostId}`, async () => {
                      setSelectedHostId(hostId);
                      const [detail, nextRuntimes, nextDependencies, nextServices, nextServiceArtifacts, nextServiceConfigs, plan] = await Promise.all([
                        api.hostInspect(hostId),
                        api.runtimesList(hostId),
                        api.depsList(hostId),
                        api.servicesList(hostId),
                        api.serviceArtifactsList(hostId),
                        api.serviceConfigsList(hostId),
                        api.envPreview(hostId),
                      ]);
                      setHostDetail(detail);
                      setRuntimes(nextRuntimes);
                      setDependencies(nextDependencies);
                      setServices(nextServices);
                      setServiceArtifacts(nextServiceArtifacts);
                      setServiceConfigs(nextServiceConfigs);
                      setServiceConfigDrafts(draftsFromServiceConfigs(nextServiceConfigs));
                      setServiceRestoreDrafts(draftsFromServiceArtifacts(nextServices, nextServiceArtifacts));
                      setEnvPlan(plan);
                      setEnvTargetProfile(plan.targetProfile);
                    })
                  }
                />
              ) : null}

              {activeView === 'languages' ? (
                <LanguagesSection
                  runtimes={runtimes}
                  onInstall={(family, version) =>
                    confirmMutation(`Install ${family} ${version} using its canonical provider on this host?`) &&
                    runAction(`Install ${family} ${version}`, () => api.runtimesInstall(activeHost?.id ?? 'native', family, version))
                  }
                  onSwitch={(family, version) =>
                    confirmMutation(`Activate ${family} ${version} as the default toolchain on this host?`) &&
                    runAction(`Activate ${family} ${version}`, () => api.runtimesSwitch(activeHost?.id ?? 'native', family, version))
                  }
                  onRemove={(family, version) =>
                    confirmMutation(`Remove ${family} ${version} from the canonical provider on this host?`) &&
                    runAction(`Remove ${family} ${version}`, () => api.runtimesRemove(activeHost?.id ?? 'native', family, version))
                  }
                />
              ) : null}

              {activeView === 'projects' ? (
                <ProjectsSection
                  runtimes={runtimes}
                  query={projectQuery}
                  setQuery={setProjectQuery}
                  projects={filteredProjects}
                  onAlign={(projectPath, family, version, options) =>
                    confirmMutation(projectAlignPrompt(family, version, options)) &&
                    runAction(projectAlignLabel(family, version), () =>
                      family === '.NET' || family === 'C/C++'
                        ? api.projectRuntimeApply(
                            activeHost?.id ?? 'native',
                            projectPath,
                            family,
                            version,
                            options,
                          )
                        : api.runtimesInstall(activeHost?.id ?? 'native', family, version)
                    )
                  }
                />
              ) : null}

              {activeView === 'deps' ? (
                <SystemDepsSection
                  dependencies={dependencies}
                  services={services}
                  serviceArtifacts={serviceArtifacts}
                  jobs={jobs}
                  serviceConfigs={serviceConfigs}
                  serviceConfigDrafts={serviceConfigDrafts}
                  setServiceConfigDrafts={setServiceConfigDrafts}
                  serviceRestoreDrafts={serviceRestoreDrafts}
                  setServiceRestoreDrafts={setServiceRestoreDrafts}
                  onInstallTemplate={() =>
                    confirmMutation(
                      'Install the base system dependency template through the detected package manager?',
                    ) && runAction('Install base dependency template', () => api.depsInstall(activeHost?.id ?? 'native', baseDependencies))
                  }
                  onServiceAction={(name, action) =>
                    confirmMutation(`${action} ${name} on this host now?`) &&
                    runAction(`${action} ${name}`, () =>
                      api.serviceAction(activeHost?.id ?? 'native', name, action),
                    )
                  }
                  onApplyServiceConfig={(name, port, dataDir) =>
                    confirmMutation(
                      `Write the managed ${name} service override block on this host now?`,
                    ) &&
                    runAction(`Apply ${name} config`, () =>
                      api.serviceConfigApply(activeHost?.id ?? 'native', name, port, dataDir),
                    )
                  }
                  onApplyServiceConfigAndRestart={(name, port, dataDir) =>
                    confirmMutation(
                      `Write the managed ${name} service override block and reconcile the service process on this host now?`,
                    ) &&
                    runAction(`Apply ${name} config and reconcile service`, () =>
                      api.serviceConfigApplyAndRestart(
                        activeHost?.id ?? 'native',
                        name,
                        port,
                        dataDir,
                      ),
                    )
                  }
                  onCreateServiceBackup={(name) =>
                    confirmMutation(
                      `Create a stopped-service snapshot backup for ${name} on this host now?`,
                    ) &&
                    runAction(`Create ${name} snapshot backup`, () =>
                      api.serviceBackupCreate(activeHost?.id ?? 'native', name),
                    )
                  }
                  onCreateServiceLogicalBackup={(name) =>
                    confirmMutation(
                      `Create a logical backup artifact for ${name} on this host now?`,
                    ) &&
                    runAction(`Create ${name} logical backup`, () =>
                      api.serviceLogicalBackupCreate(activeHost?.id ?? 'native', name),
                    )
                  }
                  onExportServiceData={(name) =>
                    confirmMutation(
                      `Export the current stopped-service data directory for ${name} on this host now?`,
                    ) &&
                    runAction(`Export ${name} data directory`, () =>
                      api.serviceDataExport(activeHost?.id ?? 'native', name),
                    )
                  }
                  onRestoreServiceBackup={(name, archivePath) =>
                    confirmMutation(
                      `Restore ${name} from ${archivePath?.trim() ? 'the selected archive path' : 'the latest managed backup'} on this host now?`,
                    ) &&
                    runAction(`Restore ${name} snapshot backup`, () =>
                      api.serviceBackupRestore(
                        activeHost?.id ?? 'native',
                        name,
                        archivePath?.trim() ? archivePath.trim() : null,
                      ),
                    )
                  }
                  onValidateServiceArtifact={(path) =>
                    confirmMutation(`Validate the selected service artifact now?\n\n${path}`) &&
                    runAction('Validate service artifact', () =>
                      api.serviceArtifactValidate(activeHost?.id ?? 'native', path),
                    )
                  }
                  onDeleteServiceArtifact={(path) =>
                    confirmMutation(`Delete the selected managed service artifact now?\n\n${path}`) &&
                    runAction('Delete service artifact', () =>
                      api.serviceArtifactDelete(activeHost?.id ?? 'native', path),
                    )
                  }
                />
              ) : null}

              {activeView === 'settings' ? (
                <SettingsSection
                  hostDetail={hostDetail}
                  envPlan={envPlan}
                  envTargetProfile={envTargetProfile}
                  exportBundle={exportBundle}
                  importDraft={importDraft}
                  importResult={importResult}
                  setEnvTargetProfile={setEnvTargetProfile}
                  setImportDraft={setImportDraft}
                  onRefreshEnvPlan={() =>
                    runAction('Refresh environment plan', async () => {
                      const plan = await api.envPreview(activeHost?.id ?? 'native');
                      setEnvPlan(plan);
                      setEnvTargetProfile(plan.targetProfile);
                    })
                  }
                  onApplyEnvPlan={() =>
                    confirmMutation(
                      `Write the managed Forge Env shell block to ${envTargetProfile || envPlan?.targetProfile || 'the selected profile'}?`,
                    ) &&
                    runAction('Apply environment shell block', () =>
                      api.envApply(activeHost?.id ?? 'native', envTargetProfile || envPlan?.targetProfile || '~/.profile'),
                    )
                  }
                  onExport={() =>
                    runAction('Generate export bundle', async () => {
                      const bundle = await api.envExport(activeHost?.id ?? 'native');
                      setExportBundle(bundle);
                    })
                  }
                  onImport={() =>
                    runAction('Validate import bundle', async () => {
                      const result = await api.envImport(importDraft);
                      setImportResult(result);
                      setSelectedImportActionIds(defaultSelectedImportActionIds(result));
                    })
                  }
                  onApplyImport={() =>
                    confirmMutation('Apply the planned import actions to the matching local hosts now?') &&
                    runAction('Apply import plan', async () => {
                      const result = await api.envImportApply(importDraft, selectedImportActionIds);
                      setImportResult(result);
                      setSelectedImportActionIds(defaultSelectedImportActionIds(result));
                    })
                  }
                  proxySettings={proxySettings}
                  setProxySettings={setProxySettings}
                  proxyPasswordDraft={proxyPasswordDraft}
                  setProxyPasswordDraft={setProxyPasswordDraft}
                  onSaveProxySettings={() =>
                    confirmMutation(
                      `Save the managed proxy profile and update the ${proxySettings.secureStore} credential entry now?`,
                    ) &&
                    runAction('Save proxy profile', async () => {
                      await api.proxySettingsSave(
                        proxySettings,
                        proxyPasswordDraft.trim() ? proxyPasswordDraft : null,
                      );
                      setProxyPasswordDraft('');
                    })
                  }
                  onClearProxySettings={() =>
                    confirmMutation(
                      `Clear the managed proxy profile and remove any saved ${proxySettings.secureStore} credential entry now?`,
                    ) &&
                    runAction('Clear proxy profile', async () => {
                      await api.proxySettingsClear();
                      setProxyPasswordDraft('');
                    })
                  }
                  onRepairImportAction={(action) =>
                    action.family &&
                    confirmMutation(
                      `Bootstrap the canonical ${action.family} provider on ${action.hostLabel} now, then automatically continue the related import actions?`,
                    ) &&
                    runAction(`Repair ${action.family} provider on ${action.hostLabel}`, async () => {
                      await api.providerBootstrap(action.hostId, action.family!);
                      const repairedPlan = await api.envImport(importDraft);
                      const followUpActionIds = repairedPlan.actions
                        .filter(
                          (nextAction) =>
                            nextAction.status === 'planned' &&
                            nextAction.hostId === action.hostId &&
                            nextAction.family === action.family,
                        )
                        .map((nextAction) => nextAction.id);

                      if (!followUpActionIds.length) {
                        setImportResult(repairedPlan);
                        setSelectedImportActionIds(defaultSelectedImportActionIds(repairedPlan));
                        return;
                      }

                      const replayedResult = await api.envImportApply(importDraft, followUpActionIds);
                      setImportResult(replayedResult);
                      setSelectedImportActionIds(defaultSelectedImportActionIds(replayedResult));
                    })
                  }
                  selectedImportActionIds={selectedImportActionIds}
                  setSelectedImportActionIds={setSelectedImportActionIds}
                />
              ) : null}
            </section>
          )}

          <footer className="mt-4 grid gap-4 xl:grid-cols-[1.2fr_1fr]">
            <div className={`${cardClass} p-4`}>
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Job Queue</p>
                  <h3 className="mt-2 text-[16px] font-semibold">Recent orchestration tasks</h3>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <span className="text-[12px] font-semibold text-[var(--text-secondary)]">
                    {pendingJobs.length} pending
                  </span>
                  <button
                    type="button"
                    className={jobFilterMode === 'relevant' ? buttonSecondaryClass : `${buttonSecondaryClass} opacity-75`}
                    onClick={() => setJobFilterMode('relevant')}
                  >
                    Relevant
                  </button>
                  <button
                    type="button"
                    className={jobFilterMode === 'all' ? buttonSecondaryClass : `${buttonSecondaryClass} opacity-75`}
                    onClick={() => setJobFilterMode('all')}
                  >
                    All jobs
                  </button>
                </div>
              </div>
              <p className="mt-3 text-[12px] leading-5 text-[var(--text-secondary)]">
                {jobFilterMode === 'relevant'
                  ? `Showing ${relevantJobCount} task(s) most relevant to ${navItems.find((item) => item.key === activeView)?.label ?? 'this view'}.`
                  : `Showing the latest ${jobs.length} task(s) across every workflow category.`}
              </p>
              <div className="mt-4 space-y-3">
                {(completedJobs.length ? completedJobs : visibleJobs).map((job) => (
                  <div
                    key={job.id}
                    className={`${insetClass} flex items-start justify-between gap-4 px-4 py-3`}
                  >
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <p className="text-[13px] font-semibold text-[var(--text-primary)]">
                          {job.outcomeTitle ?? job.label}
                        </p>
                        {job.category ? (
                          <span className="rounded-[var(--radius-pill)] border border-[var(--border-soft)] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--text-secondary)]">
                            {job.category}
                          </span>
                        ) : null}
                        {job.targetName ? (
                          <span className="rounded-[var(--radius-pill)] bg-[var(--bg-elevated)] px-2 py-1 text-[10px] font-semibold text-[var(--text-secondary)] shadow-[var(--shadow-raised-sm)]">
                            {job.targetName}
                          </span>
                        ) : null}
                      </div>
                      {job.outcomeDetail ? (
                        <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">
                          {job.outcomeDetail}
                        </p>
                      ) : (
                        <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">
                          {job.label}
                        </p>
                      )}
                      {job.nextStep ? (
                        <p className="mt-2 text-[12px] leading-5 text-[var(--text-muted)]">
                          Next: {job.nextStep}
                        </p>
                      ) : null}
                      <p className="mt-1 font-mono text-[11px] text-[var(--text-muted)]">{job.timestamp}</p>
                    </div>
                    <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] shadow-[var(--shadow-raised-sm)] ${jobStatusClass(job.status)}`}>
                      {job.status}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            <div className={`${cardClass} p-4`}>
              <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Runtime policy</p>
              <h3 className="mt-2 text-[16px] font-semibold">Canonical provider choices</h3>
              <ul className="mt-4 space-y-3 text-[13px] leading-6 text-[var(--text-secondary)]">
                <li>
                  <strong className="text-[var(--text-primary)]">Python</strong>: `pyenv` plus `pip`, `pipenv`,
                  and `poetry`.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">Node.js</strong>: `Volta` as cross-platform
                  runtime anchor, not `nvm`.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">Rust</strong>: `rustup` and `cargo`, first-class
                  across all three desktop OS targets.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">Java</strong>: `SDKMAN!` as the preferred
                  Unix-side manager, with Maven and Gradle detected as companion tooling.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">Go</strong>: official toolchain baseline first,
                  with `gvm` treated as optional rather than required.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">.NET</strong>: official user-space SDK install is
                  available, while `global.json` remains the project pinning surface for activation.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">PHP</strong>: `phpbrew` for managed versions, with
                  Composer treated as companion tooling.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">Ruby</strong>: `rbenv` plus `ruby-build`, with
                  gems and Bundler layered above the selected runtime.
                </li>
                <li>
                  <strong className="text-[var(--text-primary)]">C/C++</strong>: host compiler and build toolchain
                  inspection only, with `clang` / `gcc` / `MSVC` and `CMake` / `Ninja` surfaced without mutation.
                </li>
              </ul>
            </div>
          </footer>
        </main>
      </div>
    </div>
  );
}

function OverviewSection({
  hosts,
  runtimes,
  jobs,
  onMirrorApply,
  mirrorPreset,
  setMirrorPreset,
}: {
  hosts: HostSummary[];
  runtimes: RuntimeFamilyState[];
  jobs: JobRecord[];
  onMirrorApply: () => void;
  mirrorPreset: string;
  setMirrorPreset: (value: string) => void;
}) {
  return (
    <div className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
      <div className="space-y-4">
        <div className={`${cardClass} p-5`}>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Workspace pulse</p>
              <h3 className="mt-2 text-[18px] font-semibold">System landscape snapshot</h3>
            </div>
            <div className={`${insetClass} flex items-center gap-2 px-3 py-2`}>
              <span className="size-2 rounded-full bg-[var(--success)]" />
              <span className="text-[12px] font-semibold text-[var(--text-secondary)]">
                {hosts.length} host layer{hosts.length > 1 ? 's' : ''}
              </span>
            </div>
          </div>

          <div className="mt-5 grid gap-3 md:grid-cols-3">
            <MetricPanel label="Detected hosts" value={String(hosts.length)} hint="Native + WSL modeled separately" />
            <MetricPanel
              label="Managed runtimes"
              value={String(runtimes.reduce((sum, runtime) => sum + runtime.installed.length, 0))}
              hint="Installed versions across Python, Node.js, Rust, Java, Go, .NET, PHP, Ruby, and C/C++"
            />
            <MetricPanel
              label="Recent jobs"
              value={String(jobs.length)}
              hint="Queued, running, and completed orchestration tasks"
            />
          </div>
        </div>

        <div className={`${cardClass} p-5`}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Runtime focus</p>
              <h3 className="mt-2 text-[18px] font-semibold">Default versions per language family</h3>
            </div>
          </div>

          <div className="mt-5 grid gap-3 lg:grid-cols-3">
            {runtimes.map((runtime) => {
              const active = runtime.installed.find((entry) => entry.active);
              return (
                <div key={runtime.family} className={`${insetClass} p-4`}>
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
                      {runtime.family}
                    </p>
                    <RuntimeHealthBadge runtime={runtime} />
                  </div>
                  <p className="mt-3 text-[22px] font-semibold text-[var(--text-primary)]">
                    {active?.version ?? 'Not installed'}
                  </p>
                  <p className="mt-2 text-[13px] leading-6 text-[var(--text-secondary)]">{runtime.provider}</p>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      <div className="space-y-4">
        <div className={`${cardClass} p-5`}>
          <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Mirror preset</p>
          <h3 className="mt-2 text-[18px] font-semibold">Apply domestic mirrors safely</h3>
          <div className={`${insetClass} mt-5 p-4`}>
            <label className="block text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
              Preset
            </label>
            <select
              value={mirrorPreset}
              onChange={(event) => setMirrorPreset(event.target.value)}
              className="mt-3 w-full appearance-none rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-elevated)] px-4 py-3 text-[14px] text-[var(--text-primary)] outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
            >
              <option>Tsinghua</option>
              <option>Aliyun</option>
              <option>Huawei Cloud</option>
              <option>Company Proxy</option>
            </select>
            <button type="button" className={`${buttonPrimaryClass} mt-4 w-full`} onClick={onMirrorApply}>
              Apply preset
            </button>
          </div>
        </div>

        <div className={`${cardClass} p-5`}>
          <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Architecture note</p>
          <h3 className="mt-2 text-[18px] font-semibold">Why this UI stays compact</h3>
          <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
            Cards stay shallow, focus rings remain explicit, and the interface caps itself at three elevation
            levels so the developer tool reads like an instrument panel rather than a skeuomorphic toy.
          </p>
        </div>
      </div>
    </div>
  );
}

function HostsSection({
  hosts,
  detail,
  selectedHostId,
  onSelectHost,
}: {
  hosts: HostSummary[];
  detail: HostDetail | null;
  selectedHostId: string;
  onSelectHost: (hostId: string) => void;
}) {
  return (
    <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
      <div className="grid gap-4">
        {hosts.map((host) => (
          <button
            key={host.id}
            type="button"
            onClick={() => onSelectHost(host.id)}
            aria-pressed={selectedHostId === host.id}
            className={`${cardClass} p-5 text-left transition-[transform,box-shadow] duration-[var(--motion-hover)] ease-[var(--ease-standard)] hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)] ${
              selectedHostId === host.id ? 'ring-2 ring-[var(--focus-ring)]' : ''
            }`}
          >
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-[11px] uppercase tracking-[0.18em] text-[var(--text-muted)]">{host.kind}</p>
                <h3 className="mt-2 text-[18px] font-semibold">{host.label}</h3>
              </div>
              <span className="rounded-[var(--radius-pill)] bg-[var(--accent-soft)] px-3 py-1 text-[11px] font-semibold text-[var(--accent-primary)]">
                {host.status}
              </span>
            </div>

            <div className="mt-5 grid gap-3 md:grid-cols-3">
              <MetricPanel label="Arch" value={host.architecture} hint="Runtime binaries must match host arch." />
              <MetricPanel label="Shell" value={host.shell} hint="Used for PATH and profile updates." />
              <MetricPanel label="Pkg mgr" value={host.recommendedPackageManager} hint="System package abstraction anchor." />
            </div>

            <div className={`${insetClass} mt-5 p-4`}>
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">PATH preview</p>
              <div className="mt-3 flex flex-wrap gap-2">
                {host.pathPreview.length ? (
                  host.pathPreview.map((entry) => (
                    <span
                      key={entry}
                      className="rounded-[var(--radius-pill)] bg-[var(--bg-elevated)] px-3 py-1 font-mono text-[11px] text-[var(--text-secondary)] shadow-[var(--shadow-raised-sm)]"
                    >
                      {entry}
                    </span>
                  ))
                ) : (
                  <span className="text-[12px] text-[var(--text-secondary)]">
                    PATH preview resolves on detailed inspection for this host.
                  </span>
                )}
              </div>
            </div>
          </button>
        ))}
      </div>

      <div className={`${cardClass} p-5`}>
        <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Host detail</p>
        <h3 className="mt-2 text-[18px] font-semibold">Shell and machine policy surface</h3>
        {detail ? (
          <div className="mt-5 space-y-4">
            <div className={`${insetClass} grid gap-3 p-4 md:grid-cols-2`}>
              <MetricTile label="OS Version" value={detail.osVersion} />
              <MetricTile label="PATH Entries" value={String(detail.pathEntriesCount)} />
            </div>

            <div className={`${insetClass} p-4`}>
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Shell profiles</p>
              <div className="mt-3 flex flex-wrap gap-2">
                {detail.shellProfiles.map((profile) => (
                  <span key={profile} className="rounded-[var(--radius-pill)] bg-[var(--bg-elevated)] px-3 py-1 font-mono text-[11px] text-[var(--text-secondary)] shadow-[var(--shadow-raised-sm)]">
                    {profile}
                  </span>
                ))}
              </div>
            </div>

            <div className={`${insetClass} p-4`}>
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Package managers</p>
              <div className="mt-3 flex flex-wrap gap-2">
                {detail.packageManagers.map((manager) => (
                  <span key={manager} className="rounded-[var(--radius-pill)] border border-[var(--border-soft)] px-3 py-1 text-[12px] text-[var(--text-secondary)]">
                    {manager}
                  </span>
                ))}
              </div>
            </div>

            <div className={`${insetClass} p-4`}>
              <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Working context</p>
              <p className="mt-3 font-mono text-[11px] leading-5 text-[var(--text-secondary)]">{detail.cwd}</p>
              <p className="mt-2 font-mono text-[11px] leading-5 text-[var(--text-secondary)]">{detail.homeDir}</p>
              <div className="mt-4 space-y-2">
                {detail.notes.map((note) => (
                  <p key={note} className="text-[12px] leading-5 text-[var(--text-secondary)]">
                    {note}
                  </p>
                ))}
              </div>
            </div>
          </div>
        ) : (
          <p className="mt-4 text-[13px] leading-6 text-[var(--text-secondary)]">No host detail loaded.</p>
        )}
      </div>
    </div>
  );
}

function LanguagesSection({
  runtimes,
  onInstall,
  onSwitch,
  onRemove,
}: {
  runtimes: RuntimeFamilyState[];
  onInstall: (family: string, version: string) => void;
  onSwitch: (family: string, version: string) => void;
  onRemove: (family: string, version: string) => void;
}) {
  return (
    <div className="grid gap-4">
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
}

function ProjectsSection({
  runtimes,
  query,
  setQuery,
  projects,
  onAlign,
}: {
  runtimes: RuntimeFamilyState[];
  query: string;
  setQuery: (value: string) => void;
  projects: ProjectProfile[];
  onAlign: (
    projectPath: string,
    family: string,
    version: string,
    options?: ProjectRuntimePolicyOptions,
  ) => void;
}) {
  const [policyDrafts, setPolicyDrafts] = useState<ProjectPolicyDraftMap>({});
  const runtimeByFamily = new Map(runtimes.map((runtime) => [runtime.family, runtime]));

  return (
    <div className="grid gap-4">
      <div className={`${cardClass} p-5`}>
        <div className="grid gap-4 xl:grid-cols-[1fr_auto] xl:items-center">
          <div>
            <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">Project discovery</p>
            <h3 className="mt-2 text-[18px] font-semibold">Marker-driven runtime suggestions</h3>
          </div>
          <div className={`${insetClass} flex items-center gap-3 px-4 py-3`}>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Filter by path, marker, or project name"
              className="w-full min-w-[280px] bg-transparent text-[14px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]"
            />
          </div>
        </div>
      </div>

      {projects.length ? (
        projects.map((project) => (
          <div key={project.id} className={`${cardClass} p-5`}>
            <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
              <div>
                <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{project.health}</p>
                <h3 className="mt-2 text-[18px] font-semibold">{project.name}</h3>
                <p className="mt-2 font-mono text-[12px] text-[var(--text-muted)]">{project.path}</p>
              </div>
              <div className="flex flex-wrap gap-2">
                {project.markers.map((marker) => (
                  <span
                    key={marker}
                    className="rounded-[var(--radius-pill)] bg-[var(--bg-inset)] px-3 py-1 font-mono text-[11px] text-[var(--text-secondary)] shadow-[var(--shadow-inset)]"
                  >
                    {marker}
                  </span>
                ))}
              </div>
            </div>

            <div className="mt-5 grid gap-3 xl:grid-cols-[1.1fr_0.9fr]">
              <div className={`${insetClass} p-4`}>
                <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Suggested runtimes</p>
                <div className="mt-4 space-y-3">
                  {project.suggestedRuntimes.map((suggestion) => {
                    const runtime = runtimeByFamily.get(suggestion.family);
                    const manageable =
                      suggestion.family === '.NET' || suggestion.family === 'C/C++'
                        ? Boolean(runtime)
                        : Boolean(runtime?.capabilities.canInstall);
                    const policyKey = projectPolicyKey(project.id, suggestion.family);
                    const policyDraft =
                      policyDrafts[policyKey] ?? defaultProjectPolicyDraft(suggestion.family, suggestion.version);

                    return (
                      <div
                        key={`${project.id}-${suggestion.family}-${suggestion.version}`}
                        className="flex flex-col gap-3 rounded-[var(--radius-md)] bg-[var(--bg-elevated)] p-4 shadow-[var(--shadow-raised-sm)] md:flex-row md:items-center md:justify-between"
                      >
                        <div>
                          <p className="text-[14px] font-semibold text-[var(--text-primary)]">
                            {suggestion.family} · {suggestion.version}
                          </p>
                          <p className="mt-1 text-[12px] text-[var(--text-secondary)]">{suggestion.reason}</p>
                          {!manageable ? (
                            <p className="mt-2 text-[12px] leading-5 text-[var(--warning)]">
                              Forge Env can inspect this family, but its provider is not yet in a manageable state on the selected host.
                            </p>
                          ) : null}
                          {manageable && (suggestion.family === '.NET' || suggestion.family === 'C/C++') ? (
                            <div className="mt-3">
                              <ProjectPolicyEditor
                                family={suggestion.family}
                                options={policyDraft}
                                onChange={(nextOptions) =>
                                  setPolicyDrafts((current) => ({
                                    ...current,
                                    [policyKey]: nextOptions,
                                  }))
                                }
                              />
                            </div>
                          ) : null}
                        </div>
                        <button
                          type="button"
                          disabled={!manageable}
                          className={manageable ? buttonPrimaryClass : buttonDisabledClass}
                          onClick={() =>
                            onAlign(
                              project.path,
                              suggestion.family,
                              suggestion.version,
                              sanitizeProjectPolicyDraft(suggestion.family, policyDraft),
                            )
                          }
                        >
                          {projectActionLabel(suggestion.family)}
                        </button>
                      </div>
                    );
                  })}
                </div>
              </div>

              <div className={`${insetClass} p-4`}>
                <p className="text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">Risk flags</p>
                {project.riskFlags.length ? (
                  <div className="mt-4 space-y-2">
                    {project.riskFlags.map((flag) => (
                      <div
                        key={flag}
                        className="rounded-[var(--radius-md)] bg-[rgba(209,75,90,0.10)] px-3 py-3 text-[12px] text-[var(--danger)]"
                      >
                        {flag}
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="mt-4 text-[13px] leading-6 text-[var(--text-secondary)]">
                    No critical drift detected in this project profile.
                  </p>
                )}
              </div>
            </div>
          </div>
        ))
      ) : (
        <div className={`${cardClass} p-6 text-[13px] leading-6 text-[var(--text-secondary)]`}>
          No project markers matched the current filter. The Rust backend scans the workspace root and one level of
          children for `.python-version`, `pyproject.toml`, `requirements.txt`, `.nvmrc`, `package.json`,
          `Cargo.toml`, `pom.xml`, `build.gradle(.kts)`, `go.mod`, `global.json`, `.csproj`, `.fsproj`, `.sln`,
          `composer.json`, `Gemfile`, `.ruby-version`, `CMakeLists.txt`, `compile_commands.json`, and `Makefile`.
        </div>
      )}
    </div>
  );
}

function SystemDepsSection({
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
}: {
  dependencies: SystemDependencyState[];
  services: ServiceState[];
  serviceArtifacts: ServiceArtifact[];
  jobs: JobRecord[];
  serviceConfigs: ServiceConfigState[];
  serviceConfigDrafts: Record<string, { port: string; dataDir: string }>;
  setServiceConfigDrafts: Dispatch<
    SetStateAction<Record<string, { port: string; dataDir: string }>>
  >;
  serviceRestoreDrafts: Record<string, string>;
  setServiceRestoreDrafts: Dispatch<SetStateAction<Record<string, string>>>;
  onInstallTemplate: () => void;
  onServiceAction: (name: string, action: 'start' | 'stop' | 'restart') => void;
  onApplyServiceConfig: (name: string, port?: number | null, dataDir?: string | null) => void;
  onApplyServiceConfigAndRestart: (
    name: string,
    port?: number | null,
    dataDir?: string | null,
  ) => void;
  onCreateServiceBackup: (name: string) => void;
  onCreateServiceLogicalBackup: (name: string) => void;
  onExportServiceData: (name: string) => void;
  onRestoreServiceBackup: (name: string, archivePath?: string) => void;
  onValidateServiceArtifact: (path: string) => void;
  onDeleteServiceArtifact: (path: string) => void;
}) {
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
            const latestLogicalBackup = artifacts.find(
              (artifact) => artifact.kind === 'logical-backup',
            );
            const restoreDraft = serviceRestoreDrafts[service.name] ?? latestBackup?.path ?? '';
            const recentServiceJob = jobs.find((job) =>
              job.label.toLowerCase().includes(service.name.toLowerCase()),
            );
            const serviceTaskSummary = summarizeServiceTask(
              service,
              recentServiceJob,
              canApplyConfig,
              reconcileLabel,
            );
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
                      <p className="text-[11px] uppercase tracking-[0.12em] text-[var(--text-muted)]">
                        Managed config
                      </p>
                      <p className="mt-2 font-mono text-[11px] leading-5 text-[var(--text-secondary)]">
                        {config.configPath ?? 'No writable config path detected yet.'}
                      </p>
                    </div>

                    <div className="grid gap-3">
                      <label className="grid gap-2">
                        <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
                          Override port
                        </span>
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
                        <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
                          Data directory
                        </span>
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
                        <p className="text-[11px] uppercase tracking-[0.12em] text-[var(--text-muted)]">
                          Snapshot tools
                        </p>
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
                      <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
                        Restore archive path
                      </span>
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
                          Latest logical backup:{' '}
                          <span className="font-mono text-[11px]">{latestLogicalBackup.path}</span>
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
                            <button
                              type="button"
                              className={buttonSecondaryClass}
                              onClick={() => onValidateServiceArtifact(artifact.path)}
                            >
                              Validate
                            </button>
                            <button
                              type="button"
                              className={buttonSecondaryClass}
                              onClick={() => onDeleteServiceArtifact(artifact.path)}
                            >
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
                      <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
                        Service state
                      </p>
                      <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold ${serviceTaskSummary.toneClass}`}>
                        {serviceTaskSummary.title}
                      </span>
                    </div>
                    <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">
                      {serviceTaskSummary.detail}
                    </p>
                    {serviceTaskSummary.nextStep ? (
                      <p className="mt-2 text-[12px] leading-5 text-[var(--text-muted)]">
                        Next: {serviceTaskSummary.nextStep}
                      </p>
                    ) : null}
                  </div>
                  {recentServiceJob ? (
                    <div className={`${cardClass} px-3 py-3`}>
                      <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
                        Latest task
                      </p>
                      <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">
                        {recentServiceJob.label}
                      </p>
                      <p className="mt-1 font-mono text-[11px] text-[var(--text-muted)]">
                        {recentServiceJob.timestamp}
                      </p>
                    </div>
                  ) : null}
                  {service.notes.map((note) => (
                    <p key={note} className="text-[12px] leading-5 text-[var(--text-secondary)]">
                      {note}
                    </p>
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

function SettingsSection({
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
}: {
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
}) {
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
              <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
                Credential store
              </p>
              <p className="mt-1 text-[13px] font-semibold text-[var(--text-primary)]">
                {proxySettings.secureStore}
              </p>
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
                setProxySettings((current) => ({
                  ...current,
                  scheme: event.target.value,
                }))
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
                setProxySettings((current) => ({
                  ...current,
                  host: event.target.value,
                }))
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
                setProxySettings((current) => ({
                  ...current,
                  port: event.target.value,
                }))
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
                setProxySettings((current) => ({
                  ...current,
                  username: event.target.value,
                }))
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
          <p>
            Save writes the non-secret proxy profile into Forge Env settings and updates the system credential entry
            only when a new password is supplied.
          </p>
          <p>
            Clear removes the managed proxy profile and asks the secure store to delete the saved password for the
            current username.
          </p>
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
                  <div
                    key={variable.key}
                    className="rounded-[var(--radius-md)] bg-[var(--bg-elevated)] p-3 shadow-[var(--shadow-raised-sm)]"
                  >
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
                  <p key={note} className="text-[12px] leading-5 text-[var(--text-secondary)]">
                    {note}
                  </p>
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
          <p className="mt-4 text-[13px] leading-6 text-[var(--text-secondary)]">
            Environment plan unavailable.
          </p>
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
                                    if (!canSelect) {
                                      return;
                                    }
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
                              <p className="mt-2 text-[12px] leading-5 text-[var(--warning)]">
                                Fix suggestion: {action.remediation}
                              </p>
                            ) : null}
                            {action.nextStep ? (
                              <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">
                                Next step: {action.nextStep}
                              </p>
                            ) : null}
                            {action.status === 'blocked' && action.repairAvailable && action.family ? (
                              <div className="mt-3">
                                <button
                                  type="button"
                                  className={buttonSecondaryClass}
                                  onClick={() => onRepairImportAction(action)}
                                >
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

function RuntimeHealthBadge({ runtime }: { runtime: RuntimeFamilyState }) {
  const tone =
    runtime.health === 'good'
      ? 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]'
      : runtime.health === 'missing'
        ? 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]'
        : 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]';

  return (
    <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] ${tone}`}>
      {runtime.providerStatus}
    </span>
  );
}

function ProjectPolicyEditor({
  family,
  options,
  onChange,
}: {
  family: string;
  options: ProjectRuntimePolicyOptions;
  onChange: (options: ProjectRuntimePolicyOptions) => void;
}) {
  if (family === '.NET') {
    const dotnet = options.dotnet ?? {};
    return (
      <div className="grid gap-3 md:grid-cols-[minmax(0,220px)_auto] md:items-end">
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            rollForward
          </span>
          <select
            value={dotnet.rollForward ?? ''}
            onChange={(event) =>
              onChange({
                dotnet: {
                  ...dotnet,
                  rollForward: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
          >
            <option value="disable">disable</option>
            <option value="patch">patch</option>
            <option value="feature">feature</option>
            <option value="minor">minor</option>
            <option value="major">major</option>
            <option value="latestPatch">latestPatch</option>
            <option value="latestFeature">latestFeature</option>
            <option value="latestMinor">latestMinor</option>
            <option value="latestMajor">latestMajor</option>
          </select>
        </label>
        <label className="flex items-center gap-3 rounded-[var(--radius-md)] bg-[rgba(220,231,255,0.38)] px-4 py-3 text-[12px] text-[var(--text-secondary)]">
          <input
            type="checkbox"
            checked={Boolean(dotnet.allowPrerelease)}
            onChange={(event) =>
              onChange({
                dotnet: {
                  ...dotnet,
                  allowPrerelease: event.target.checked,
                },
              })
            }
            className="size-4 rounded border border-[var(--border-soft)] accent-[var(--accent-primary)]"
          />
          <span>Allow prerelease SDK resolution</span>
        </label>
      </div>
    );
  }

  if (family === 'C/C++') {
    const cpp = options.cpp ?? {};
    return (
      <div className="grid gap-3 md:grid-cols-2">
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Generator
          </span>
          <select
            value={cpp.generator ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  generator: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
          >
            <option value="">Auto detect</option>
            <option value="Ninja">Ninja</option>
            <option value="Unix Makefiles">Unix Makefiles</option>
            <option value="NMake Makefiles">NMake Makefiles</option>
          </select>
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Build type
          </span>
          <select
            value={cpp.buildType ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  buildType: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
          >
            <option value="">Unspecified</option>
            <option value="Debug">Debug</option>
            <option value="Release">Release</option>
            <option value="RelWithDebInfo">RelWithDebInfo</option>
            <option value="MinSizeRel">MinSizeRel</option>
          </select>
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Binary dir
          </span>
          <input
            value={cpp.binaryDir ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  binaryDir: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder="${sourceDir}/build/forge-env"
          />
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
            Toolchain file
          </span>
          <input
            value={cpp.toolchainFile ?? ''}
            onChange={(event) =>
              onChange({
                cpp: {
                  ...cpp,
                  toolchainFile: event.target.value,
                },
              })
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder="cmake/toolchains/dev.cmake"
          />
        </label>
      </div>
    );
  }

  return null;
}

function projectActionLabel(family: string) {
  if (family === '.NET') {
    return 'Write SDK pin';
  }
  if (family === 'C/C++') {
    return 'Write CMake preset';
  }
  return 'Align runtime';
}

function projectAlignPrompt(
  family: string,
  version: string,
  options?: ProjectRuntimePolicyOptions,
) {
  if (family === '.NET') {
    return `Write or update global.json to pin .NET ${version}${projectPolicyOptionSummary(family, options)} for this detected project?`;
  }
  if (family === 'C/C++') {
    return `Write or update CMakePresets.json with a ${version} compiler/build preset${projectPolicyOptionSummary(family, options)} for this detected project?`;
  }
  return `Install or align ${family} ${version} for the detected project profile?`;
}

function projectAlignLabel(family: string, version: string) {
  if (family === '.NET') {
    return `Pin .NET ${version}`;
  }
  if (family === 'C/C++') {
    return `Write ${version} CMake preset`;
  }
  return `Align ${family} ${version}`;
}

function projectPolicyKey(projectId: string, family: string) {
  return `${projectId}:${family}`;
}

function defaultProjectPolicyDraft(
  family: string,
  version: string,
): ProjectRuntimePolicyOptions {
  if (family === '.NET') {
    return {
      dotnet: {
        rollForward: defaultDotnetRollForward(version),
        allowPrerelease: version.includes('-'),
      },
    };
  }
  if (family === 'C/C++') {
    return {
      cpp: {
        generator: '',
        binaryDir: '${sourceDir}/build/forge-env',
        toolchainFile: '',
        buildType: '',
      },
    };
  }
  return {};
}

function defaultDotnetRollForward(version: string) {
  const normalized = version.trim().replace(/ LTS$/u, '');
  return normalized.split('.').length >= 3 ? 'disable' : 'latestFeature';
}

function sanitizeProjectPolicyDraft(
  family: string,
  options?: ProjectRuntimePolicyOptions,
): ProjectRuntimePolicyOptions | undefined {
  if (!options) {
    return undefined;
  }

  if (family === '.NET') {
    const dotnet = options.dotnet;
    if (!dotnet) {
      return undefined;
    }
    return {
      dotnet: {
        rollForward: normalizeOptionalText(dotnet.rollForward),
        allowPrerelease:
          typeof dotnet.allowPrerelease === 'boolean' ? dotnet.allowPrerelease : undefined,
      },
    };
  }

  if (family === 'C/C++') {
    const cpp = options.cpp;
    if (!cpp) {
      return undefined;
    }
    return {
      cpp: {
        generator: normalizeOptionalText(cpp.generator),
        binaryDir: normalizeOptionalText(cpp.binaryDir),
        toolchainFile: normalizeOptionalText(cpp.toolchainFile),
        buildType: normalizeOptionalText(cpp.buildType),
      },
    };
  }

  return undefined;
}

function normalizeOptionalText(value?: string | null) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function projectPolicyOptionSummary(
  family: string,
  options?: ProjectRuntimePolicyOptions,
) {
  if (family === '.NET') {
    const dotnet = options?.dotnet;
    if (!dotnet) {
      return '';
    }
    const parts = [
      dotnet.rollForward ? `rollForward=${dotnet.rollForward}` : null,
      typeof dotnet.allowPrerelease === 'boolean'
        ? `allowPrerelease=${String(dotnet.allowPrerelease)}`
        : null,
    ].filter(Boolean);
    return parts.length ? ` (${parts.join(', ')})` : '';
  }

  if (family === 'C/C++') {
    const cpp = options?.cpp;
    if (!cpp) {
      return '';
    }
    const parts = [
      cpp.generator ? `generator=${cpp.generator}` : null,
      cpp.binaryDir ? `binaryDir=${cpp.binaryDir}` : null,
      cpp.toolchainFile ? `toolchainFile=${cpp.toolchainFile}` : null,
      cpp.buildType ? `buildType=${cpp.buildType}` : null,
    ].filter(Boolean);
    return parts.length ? ` (${parts.join(', ')})` : '';
  }

  return '';
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

function categoriesForView(view: NavKey): string[] | null {
  switch (view) {
    case 'overview':
      return null;
    case 'hosts':
      return ['environment', 'import'];
    case 'languages':
      return ['runtime', 'provider'];
    case 'projects':
      return ['runtime', 'provider', 'import'];
    case 'deps':
      return ['dependency', 'service', 'mirror'];
    case 'settings':
      return ['environment', 'mirror', 'import', 'provider'];
    default:
      return null;
  }
}

function filterJobsForView(jobs: JobRecord[], view: NavKey, mode: JobFilterMode) {
  if (mode === 'all') {
    return jobs;
  }

  const categories = categoriesForView(view);
  if (!categories) {
    return jobs;
  }

  const filtered = jobs.filter((job) => job.category && categories.includes(job.category));
  return filtered.length ? filtered : jobs;
}

function draftsFromServiceConfigs(configs: ServiceConfigState[]) {
  return Object.fromEntries(
    configs.map((config) => [
      config.serviceName,
      {
        port: config.port != null ? String(config.port) : '',
        dataDir: config.dataDir ?? '',
      },
    ]),
  );
}

function draftsFromServiceArtifacts(
  services: ServiceState[],
  artifacts: ServiceArtifact[],
) {
  return Object.fromEntries(
    services.map((service) => {
      const latestBackup = artifacts.find(
        (artifact) => artifact.serviceName === service.name && artifact.kind === 'backup',
      );
      return [service.name, latestBackup?.path ?? ''];
    }),
  );
}

function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function jobStatusClass(status: JobStatus) {
  if (status === 'failed') {
    return 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]';
  }
  if (status === 'queued' || status === 'running') {
    return 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]';
  }
  return 'bg-[var(--bg-elevated)] text-[var(--text-secondary)]';
}

function groupImportActionsByHost(actions: ImportAction[]) {
  const grouped = new Map<string, ImportAction[]>();
  actions.forEach((action) => {
    const existing = grouped.get(action.hostLabel) ?? [];
    existing.push(action);
    grouped.set(action.hostLabel, existing);
  });
  return Array.from(grouped.entries());
}

function defaultSelectedImportActionIds(result: ImportResult) {
  return result.actions
    .filter((action) => action.status === 'planned' && action.selected)
    .map((action) => action.id);
}

function MetricTile({ label, value }: { label: string; value: string }) {
  return (
    <div className={`${insetClass} px-3 py-3`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{label}</p>
      <p className="mt-2 text-[18px] font-semibold text-[var(--text-primary)]">{value}</p>
    </div>
  );
}

function MetricPanel({ label, value, hint }: { label: string; value: string; hint: string }) {
  return (
    <div className={`${insetClass} p-4`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{label}</p>
      <p className="mt-3 text-[24px] font-semibold text-[var(--text-primary)]">{value}</p>
      <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">{hint}</p>
    </div>
  );
}

function NavGlyph({ name, active }: { name: NavKey; active: boolean }) {
  const stroke = active ? 'var(--accent-primary)' : 'var(--text-secondary)';

  return (
    <svg
      viewBox="0 0 24 24"
      className="size-5 shrink-0"
      fill="none"
      stroke={stroke}
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {name === 'overview' ? (
        <>
          <rect x="4" y="4" width="7" height="7" rx="2" />
          <rect x="13" y="4" width="7" height="4" rx="2" />
          <rect x="13" y="10" width="7" height="10" rx="2" />
          <rect x="4" y="13" width="7" height="7" rx="2" />
        </>
      ) : null}
      {name === 'hosts' ? (
        <>
          <rect x="4" y="5" width="16" height="6" rx="2" />
          <path d="M8 11v6" />
          <path d="M16 11v6" />
          <path d="M6 17h4" />
          <path d="M14 17h4" />
        </>
      ) : null}
      {name === 'languages' ? (
        <>
          <path d="M5 7h14" />
          <path d="M8 4v16" />
          <path d="M16 4v16" />
          <path d="M5 17h14" />
        </>
      ) : null}
      {name === 'projects' ? (
        <>
          <path d="M4 7h7l2 2h7v8a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3z" />
          <path d="M9 13h6" />
        </>
      ) : null}
      {name === 'deps' ? (
        <>
          <circle cx="8" cy="12" r="3" />
          <circle cx="16" cy="12" r="3" />
          <path d="M11 12h2" />
          <path d="M8 9V6" />
          <path d="M16 15v3" />
        </>
      ) : null}
      {name === 'settings' ? (
        <>
          <circle cx="12" cy="12" r="3" />
          <path d="M12 4v2" />
          <path d="M12 18v2" />
          <path d="M4 12h2" />
          <path d="M18 12h2" />
          <path d="m6.3 6.3 1.4 1.4" />
          <path d="m16.3 16.3 1.4 1.4" />
          <path d="m17.7 6.3-1.4 1.4" />
          <path d="m7.7 16.3-1.4 1.4" />
        </>
      ) : null}
    </svg>
  );
}

export default App;
