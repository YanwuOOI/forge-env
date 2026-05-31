import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { api } from '../api';
import { draftsFromServiceConfigs, draftsFromServiceArtifacts } from '../utils';
import type {
  HostDetail,
  HostSummary,
  JobRecord,
  RuntimeFamilyState,
  ServiceArtifact,
  ServiceConfigState,
  ServiceState,
  SystemDependencyState,
  EnvPlan,
  ProjectProfile,
} from '../types';

export interface WorkspaceState {
  hosts: HostSummary[];
  runtimes: RuntimeFamilyState[];
  projects: ProjectProfile[];
  jobs: JobRecord[];
  hostDetail: HostDetail | null;
  selectedHostId: string;
  dependencies: SystemDependencyState[];
  services: ServiceState[];
  serviceArtifacts: ServiceArtifact[];
  serviceConfigs: ServiceConfigState[];
  serviceConfigDrafts: Record<string, { port: string; dataDir: string }>;
  serviceRestoreDrafts: Record<string, string>;
  envPlan: EnvPlan | null;
  envTargetProfile: string;
  mirrorPreset: string;
  loading: boolean;
  error: string | null;
}

export function useWorkspace() {
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
  const [envPlan, setEnvPlan] = useState<EnvPlan | null>(null);
  const [envTargetProfile, setEnvTargetProfile] = useState('');
  const [mirrorPreset, setMirrorPreset] = useState('Tsinghua');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refreshAll = useCallback(async (currentSelectedHostId?: string) => {
    setLoading(true);
    setError(null);

    try {
      const [nextHosts, nextProjects, nextJobs, nextPreferences] = await Promise.all([
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
        (currentSelectedHostId && nextHosts.some((host) => host.id === currentSelectedHostId) && currentSelectedHostId) ||
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
    } catch (refreshError) {
      setError(refreshError instanceof Error ? refreshError.message : 'Failed to load workspace data.');
    } finally {
      setLoading(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    void refreshAll(selectedHostId);
  }, [refreshAll]);

  // Subscribe to job updates
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

  const selectHost = useCallback(async (hostId: string) => {
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
  }, []);

  const updateEnvPlan = useCallback(async () => {
    const plan = await api.envPreview(selectedHostId || 'native');
    setEnvPlan(plan);
    setEnvTargetProfile(plan.targetProfile);
  }, [selectedHostId]);

  return useMemo(() => ({
    hosts,
    runtimes,
    projects,
    jobs,
    hostDetail,
    selectedHostId,
    dependencies,
    services,
    serviceArtifacts,
    serviceConfigs,
    serviceConfigDrafts,
    setServiceConfigDrafts,
    serviceRestoreDrafts,
    setServiceRestoreDrafts,
    envPlan,
    envTargetProfile,
    setEnvTargetProfile,
    mirrorPreset,
    setMirrorPreset,
    loading,
    error,
    refreshAll,
    selectHost,
    updateEnvPlan,
  }), [
    hosts, runtimes, projects, jobs, hostDetail, selectedHostId,
    dependencies, services, serviceArtifacts, serviceConfigs,
    serviceConfigDrafts, serviceRestoreDrafts, envPlan, envTargetProfile,
    mirrorPreset, loading, error, refreshAll, selectHost, updateEnvPlan,
    setServiceConfigDrafts, setServiceRestoreDrafts, setEnvTargetProfile, setMirrorPreset,
  ]);
}
