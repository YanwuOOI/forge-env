import { invoke, isTauri } from '@tauri-apps/api/core';
import type {
  ProjectRuntimePolicyOptions,
  ProxySettings,
  AppPreferences,
  EnvPlan,
  ExportBundle,
  HostDetail,
  HostSummary,
  ImportResult,
  JobRecord,
  ProjectProfile,
  RuntimeFamilyState,
  ServiceArtifact,
  ServiceConfigState,
  ServiceState,
  SystemDependencyState,
} from './types';
import { browserInvoke } from './mock';

async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return browserInvoke<T>(name, args);
  }

  return invoke<T>(name, args);
}

export const api = {
  hostsList: () => command<HostSummary[]>('hosts_list'),
  hostInspect: (hostId: string) => command<HostDetail>('hosts_inspect', { hostId }),
  projectsInspect: (path?: string) => command<ProjectProfile[]>('projects_inspect', { path }),
  projectRuntimeApply: (
    hostId: string,
    projectPath: string,
    family: string,
    version: string,
    options?: ProjectRuntimePolicyOptions,
  ) => command<JobRecord>('project_runtime_apply', { hostId, projectPath, family, version, options }),
  runtimesList: (hostId: string) => command<RuntimeFamilyState[]>('runtimes_list', { hostId }),
  depsList: (hostId: string) => command<SystemDependencyState[]>('deps_list', { hostId }),
  servicesList: (hostId: string) => command<ServiceState[]>('services_list', { hostId }),
  serviceArtifactsList: (hostId: string) =>
    command<ServiceArtifact[]>('service_artifacts_list', { hostId }),
  serviceConfigsList: (hostId: string) =>
    command<ServiceConfigState[]>('services_config_list', { hostId }),
  appPreferences: () => command<AppPreferences>('app_preferences'),
  proxySettingsLoad: () => command<ProxySettings>('proxy_settings_load'),
  runtimesInstall: (hostId: string, family: string, version: string, scope = 'global') =>
    command<JobRecord>('runtimes_install', { hostId, family, version, scope }),
  runtimesSwitch: (hostId: string, family: string, version: string, scope = 'global') =>
    command<JobRecord>('runtimes_switch', { hostId, family, version, scope }),
  runtimesRemove: (hostId: string, family: string, version: string) =>
    command<JobRecord>('runtimes_remove', { hostId, family, version }),
  depsInstall: (hostId: string, dependencies: string[]) => command<JobRecord>('deps_install', { hostId, dependencies }),
  serviceAction: (hostId: string, name: string, action: 'start' | 'stop' | 'restart') =>
    command<JobRecord>('service_action', { hostId, name, action }),
  serviceConfigApply: (
    hostId: string,
    name: string,
    port?: number | null,
    dataDir?: string | null,
  ) => command<JobRecord>('service_config_apply', { hostId, name, port, dataDir }),
  serviceConfigApplyAndRestart: (
    hostId: string,
    name: string,
    port?: number | null,
    dataDir?: string | null,
  ) => command<JobRecord>('service_config_apply_and_restart', { hostId, name, port, dataDir }),
  serviceBackupCreate: (hostId: string, name: string) =>
    command<JobRecord>('service_backup_create', { hostId, name }),
  serviceLogicalBackupCreate: (hostId: string, name: string) =>
    command<JobRecord>('service_logical_backup_create', { hostId, name }),
  serviceDataExport: (hostId: string, name: string) =>
    command<JobRecord>('service_data_export', { hostId, name }),
  serviceBackupRestore: (hostId: string, name: string, archivePath?: string | null) =>
    command<JobRecord>('service_backup_restore', { hostId, name, archivePath }),
  serviceArtifactValidate: (hostId: string, path: string) =>
    command<JobRecord>('service_artifact_validate', { hostId, path }),
  serviceArtifactDelete: (hostId: string, path: string) =>
    command<JobRecord>('service_artifact_delete', { hostId, path }),
  proxySettingsSave: (settings: ProxySettings, password?: string | null) =>
    command<JobRecord>('proxy_settings_save', { settings, password }),
  proxySettingsClear: () => command<JobRecord>('proxy_settings_clear'),
  mirrorsApply: (hostId: string, mirrorSet: string) =>
    command<JobRecord>('mirrors_apply', { hostId, mirrorSet }),
  providerBootstrap: (hostId: string, family: string) =>
    command<JobRecord>('provider_bootstrap', { hostId, family }),
  envExport: (hostId?: string) => command<ExportBundle>('env_export', hostId ? { hostId } : undefined),
  envImport: (payload: string) => command<ImportResult>('env_import', { payload }),
  envImportApply: (payload: string, selectedActionIds?: string[]) =>
    command<ImportResult>('env_import_apply', { payload, selectedActionIds }),
  envPreview: (hostId: string) => command<EnvPlan>('env_preview', { hostId }),
  envApply: (hostId: string, profilePath: string) => command<JobRecord>('env_apply', { hostId, profilePath }),
  jobsSubscribe: () => command<JobRecord[]>('jobs_subscribe'),
};
