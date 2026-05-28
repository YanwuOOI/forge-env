import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  useDeferredValue,
  useEffect,
  useEffectEvent,
  useState,
} from 'react';
import { api } from './lib/api';
import { baseDependencies } from './lib/constants';
import { filterJobsForView, defaultSelectedImportActionIds, draftsFromServiceConfigs, draftsFromServiceArtifacts } from './lib/utils';
import { useConfirm } from './lib/hooks/useConfirm';
import { useTheme } from './lib/hooks/useTheme';
import { useUpdater } from './lib/hooks/useUpdater';
import type {
  EnvPlan,
  ExportBundle,
  HostDetail,
  HostSummary,
  ImportResult,
  JobRecord,
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

import { Sidebar } from './components/layout/Sidebar';
import { TopBar } from './components/layout/TopBar';
import { JobQueue } from './components/layout/JobQueue';
import { RuntimePolicyCard } from './components/layout/RuntimePolicyCard';
import { OverviewSection } from './components/overview/OverviewSection';
import { HostsSection } from './components/hosts/HostsSection';
import { LanguagesSection } from './components/languages/LanguagesSection';
import { ProjectsSection } from './components/projects/ProjectsSection';
import { SystemDepsSection } from './components/system-deps/SystemDepsSection';
import { SettingsSection } from './components/settings/SettingsSection';
import { UpdateBanner } from './components/shared/UpdateBanner';

import { panelClass, cardClass } from './lib/constants';

type JobFilterMode = 'relevant' | 'all';

function projectAlignPrompt(family: string, version: string, options?: ProjectRuntimePolicyOptions) {
  if (family === '.NET') {
    const parts: string[] = [];
    if (options?.dotnet?.rollForward) parts.push(`rollForward=${options.dotnet.rollForward}`);
    if (typeof options?.dotnet?.allowPrerelease === 'boolean') parts.push(`allowPrerelease=${String(options.dotnet.allowPrerelease)}`);
    const suffix = parts.length ? ` (${parts.join(', ')})` : '';
    return `Write or update global.json to pin .NET ${version}${suffix} for this detected project?`;
  }
  if (family === 'C/C++') {
    const parts: string[] = [];
    if (options?.cpp?.generator) parts.push(`generator=${options.cpp.generator}`);
    if (options?.cpp?.binaryDir) parts.push(`binaryDir=${options.cpp.binaryDir}`);
    if (options?.cpp?.toolchainFile) parts.push(`toolchainFile=${options.cpp.toolchainFile}`);
    if (options?.cpp?.buildType) parts.push(`buildType=${options.cpp.buildType}`);
    const suffix = parts.length ? ` (${parts.join(', ')})` : '';
    return `Write or update CMakePresets.json with a ${version} compiler/build preset${suffix} for this detected project?`;
  }
  return `Install or align ${family} ${version} for the detected project profile?`;
}

function projectAlignLabel(family: string, version: string) {
  if (family === '.NET') return `Pin .NET ${version}`;
  if (family === 'C/C++') return `Write ${version} CMake preset`;
  return `Align ${family} ${version}`;
}

function App() {
  const confirm = useConfirm();
  const { theme, toggleTheme } = useTheme();
  const { updateInfo, downloading, installUpdate, dismissUpdate } = useUpdater();
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
    if (!token) return true;
    return [project.name, project.path, ...project.markers].join(' ').toLowerCase().includes(token);
  });

  const refreshAll = useEffectEvent(async () => {
    setLoading(true);
    setError(null);

    try {
      const [nextHosts, nextProjects, nextJobs, nextPreferences, nextProxySettings] = await Promise.all([
        api.hostsList(),
        api.projectsInspect(),
        api.jobsSubscribe(),
        api.appPreferences(),
        api.proxySettingsLoad(),
      ]);
      const persistedHostId =
        nextPreferences.selectedHostId && nextHosts.some((host) => host.id === nextPreferences.selectedHostId)
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
    if (!isTauri()) return;
    let cleanup: (() => void) | undefined;
    void listen<JobRecord[]>('jobs://updated', (event) => {
      setJobs(event.payload);
    }).then((unlisten) => {
      cleanup = unlisten;
    });
    return () => { cleanup?.(); };
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
  const totalInstalledRuntimes = runtimes.reduce((sum, runtime) => sum + runtime.installed.length, 0);

  return (
    <div className="min-h-dvh bg-[var(--bg-canvas)] p-4 text-[var(--text-primary)] md:p-6">
      <div className="mx-auto grid min-h-[calc(100dvh-2rem)] max-w-[1600px] gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
        <Sidebar
          activeView={activeView}
          setActiveView={setActiveView}
          hostsCount={hosts.length}
          totalInstalledRuntimes={totalInstalledRuntimes}
          busyLabel={busyLabel}
          theme={theme}
          onToggleTheme={toggleTheme}
        />

        <main className={`${panelClass} flex min-w-0 flex-col p-4 md:p-5`}>
          <TopBar
            activeView={activeView}
            activeHost={activeHost}
            onRefresh={() => void refreshAll()}
          />

          {error ? (
            <div className="mt-4 rounded-[var(--radius-md)] bg-[rgba(209,75,90,0.12)] px-4 py-3 text-[13px] text-[var(--danger)]">
              {error}
            </div>
          ) : null}

          {updateInfo ? (
            <div className="mt-4">
              <UpdateBanner
                version={updateInfo.version}
                body={updateInfo.body}
                downloading={downloading}
                onInstall={installUpdate}
                onDismiss={dismissUpdate}
              />
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
                  onMirrorApply={async () => {
                    if (await confirm(`Apply the ${mirrorPreset} mirror preset to npm, pip, and Cargo config on this host?`)) {
                      await runAction(`Apply ${mirrorPreset} mirror preset`, () =>
                        api.mirrorsApply(activeHost?.id ?? 'native', mirrorPreset),
                      );
                    }
                  }}
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
                  onInstall={async (family, version) => {
                    if (await confirm(`Install ${family} ${version} using its canonical provider on this host?`)) {
                      await runAction(`Install ${family} ${version}`, () => api.runtimesInstall(activeHost?.id ?? 'native', family, version));
                    }
                  }}
                  onSwitch={async (family, version) => {
                    if (await confirm(`Activate ${family} ${version} as the default toolchain on this host?`)) {
                      await runAction(`Activate ${family} ${version}`, () => api.runtimesSwitch(activeHost?.id ?? 'native', family, version));
                    }
                  }}
                  onRemove={async (family, version) => {
                    if (await confirm(`Remove ${family} ${version} from the canonical provider on this host?`)) {
                      await runAction(`Remove ${family} ${version}`, () => api.runtimesRemove(activeHost?.id ?? 'native', family, version));
                    }
                  }}
                />
              ) : null}

              {activeView === 'projects' ? (
                <ProjectsSection
                  runtimes={runtimes}
                  query={projectQuery}
                  setQuery={setProjectQuery}
                  projects={filteredProjects}
                  onAlign={async (projectPath, family, version, options) => {
                    if (await confirm(projectAlignPrompt(family, version, options))) {
                      await runAction(projectAlignLabel(family, version), () =>
                        family === '.NET' || family === 'C/C++'
                          ? api.projectRuntimeApply(activeHost?.id ?? 'native', projectPath, family, version, options)
                          : api.runtimesInstall(activeHost?.id ?? 'native', family, version)
                      );
                    }
                  }}
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
                  onInstallTemplate={async () => {
                    if (await confirm('Install the base system dependency template through the detected package manager?')) {
                      await runAction('Install base dependency template', () => api.depsInstall(activeHost?.id ?? 'native', baseDependencies));
                    }
                  }}
                  onServiceAction={async (name, action) => {
                    if (await confirm(`${action} ${name} on this host now?`)) {
                      await runAction(`${action} ${name}`, () => api.serviceAction(activeHost?.id ?? 'native', name, action));
                    }
                  }}
                  onApplyServiceConfig={async (name, port, dataDir) => {
                    if (await confirm(`Write the managed ${name} service override block on this host now?`)) {
                      await runAction(`Apply ${name} config`, () => api.serviceConfigApply(activeHost?.id ?? 'native', name, port, dataDir));
                    }
                  }}
                  onApplyServiceConfigAndRestart={async (name, port, dataDir) => {
                    if (await confirm(`Write the managed ${name} service override block and reconcile the service process on this host now?`)) {
                      await runAction(`Apply ${name} config and reconcile service`, () => api.serviceConfigApplyAndRestart(activeHost?.id ?? 'native', name, port, dataDir));
                    }
                  }}
                  onCreateServiceBackup={async (name) => {
                    if (await confirm(`Create a stopped-service snapshot backup for ${name} on this host now?`)) {
                      await runAction(`Create ${name} snapshot backup`, () => api.serviceBackupCreate(activeHost?.id ?? 'native', name));
                    }
                  }}
                  onCreateServiceLogicalBackup={async (name) => {
                    if (await confirm(`Create a logical backup artifact for ${name} on this host now?`)) {
                      await runAction(`Create ${name} logical backup`, () => api.serviceLogicalBackupCreate(activeHost?.id ?? 'native', name));
                    }
                  }}
                  onExportServiceData={async (name) => {
                    if (await confirm(`Export the current stopped-service data directory for ${name} on this host now?`)) {
                      await runAction(`Export ${name} data directory`, () => api.serviceDataExport(activeHost?.id ?? 'native', name));
                    }
                  }}
                  onRestoreServiceBackup={async (name, archivePath) => {
                    if (await confirm(`Restore ${name} from ${archivePath?.trim() ? 'the selected archive path' : 'the latest managed backup'} on this host now?`)) {
                      await runAction(`Restore ${name} snapshot backup`, () => api.serviceBackupRestore(activeHost?.id ?? 'native', name, archivePath?.trim() ? archivePath.trim() : null));
                    }
                  }}
                  onValidateServiceArtifact={async (path) => {
                    if (await confirm(`Validate the selected service artifact now?\n\n${path}`)) {
                      await runAction('Validate service artifact', () => api.serviceArtifactValidate(activeHost?.id ?? 'native', path));
                    }
                  }}
                  onDeleteServiceArtifact={async (path) => {
                    if (await confirm(`Delete the selected managed service artifact now?\n\n${path}`, { danger: true })) {
                      await runAction('Delete service artifact', () => api.serviceArtifactDelete(activeHost?.id ?? 'native', path));
                    }
                  }}
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
                  onApplyEnvPlan={async () => {
                    if (await confirm(`Write the managed Forge Env shell block to ${envTargetProfile || envPlan?.targetProfile || 'the selected profile'}?`)) {
                      await runAction('Apply environment shell block', () =>
                        api.envApply(activeHost?.id ?? 'native', envTargetProfile || envPlan?.targetProfile || '~/.profile'),
                      );
                    }
                  }}
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
                  onApplyImport={async () => {
                    if (await confirm('Apply the planned import actions to the matching local hosts now?')) {
                      await runAction('Apply import plan', async () => {
                        const result = await api.envImportApply(importDraft, selectedImportActionIds);
                        setImportResult(result);
                        setSelectedImportActionIds(defaultSelectedImportActionIds(result));
                      });
                    }
                  }}
                  proxySettings={proxySettings}
                  setProxySettings={setProxySettings}
                  proxyPasswordDraft={proxyPasswordDraft}
                  setProxyPasswordDraft={setProxyPasswordDraft}
                  onSaveProxySettings={async () => {
                    if (await confirm(`Save the managed proxy profile and update the ${proxySettings.secureStore} credential entry now?`)) {
                      await runAction('Save proxy profile', async () => {
                        await api.proxySettingsSave(proxySettings, proxyPasswordDraft.trim() ? proxyPasswordDraft : null);
                        setProxyPasswordDraft('');
                      });
                    }
                  }}
                  onClearProxySettings={async () => {
                    if (await confirm(`Clear the managed proxy profile and remove any saved ${proxySettings.secureStore} credential entry now?`)) {
                      await runAction('Clear proxy profile', async () => {
                        await api.proxySettingsClear();
                        setProxyPasswordDraft('');
                      });
                    }
                  }}
                  onRepairImportAction={async (action) => {
                    if (!action.family) return;
                    if (await confirm(`Bootstrap the canonical ${action.family} provider on ${action.hostLabel} now, then automatically continue the related import actions?`)) {
                      await runAction(`Repair ${action.family} provider on ${action.hostLabel}`, async () => {
                        await api.providerBootstrap(action.hostId, action.family!);
                        const repairedPlan = await api.envImport(importDraft);
                        const followUpActionIds = repairedPlan.actions
                          .filter((nextAction) => nextAction.status === 'planned' && nextAction.hostId === action.hostId && nextAction.family === action.family)
                          .map((nextAction) => nextAction.id);
                        if (!followUpActionIds.length) {
                          setImportResult(repairedPlan);
                          setSelectedImportActionIds(defaultSelectedImportActionIds(repairedPlan));
                          return;
                        }
                        const replayedResult = await api.envImportApply(importDraft, followUpActionIds);
                        setImportResult(replayedResult);
                        setSelectedImportActionIds(defaultSelectedImportActionIds(replayedResult));
                      });
                    }
                  }}
                  selectedImportActionIds={selectedImportActionIds}
                  setSelectedImportActionIds={setSelectedImportActionIds}
                />
              ) : null}
            </section>
          )}

          <footer className="mt-4 grid gap-4 xl:grid-cols-[1.2fr_1fr]">
            <JobQueue
              jobs={jobs}
              activeView={activeView}
              jobFilterMode={jobFilterMode}
              setJobFilterMode={setJobFilterMode}
              pendingJobs={pendingJobs}
            />
            <RuntimePolicyCard />
          </footer>
        </main>
      </div>
    </div>
  );
}

export default App;
