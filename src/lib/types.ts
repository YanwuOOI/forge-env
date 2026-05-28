export type NavKey =
  | 'overview'
  | 'hosts'
  | 'languages'
  | 'projects'
  | 'deps'
  | 'settings';

export type HostKind = 'macos' | 'linux' | 'windows' | 'wsl';
export type JobStatus = 'queued' | 'running' | 'completed' | 'failed';

export interface HostSummary {
  id: string;
  label: string;
  kind: HostKind;
  architecture: string;
  shell: string;
  status: string;
  recommendedPackageManager: string;
  pathPreview: string[];
}

export interface HostDetail {
  summary: HostSummary;
  osVersion: string;
  cwd: string;
  homeDir: string;
  pathEntriesCount: number;
  shellProfiles: string[];
  packageManagers: string[];
  mirrorsSupported: string[];
  notes: string[];
}

export interface RuntimeInstallation {
  version: string;
  channel: string;
  active: boolean;
  source: string;
  tools: string[];
}

export interface RuntimeCapabilities {
  canInstall: boolean;
  canActivate: boolean;
  canRemove: boolean;
}

export interface RuntimeFamilyState {
  family: string;
  provider: string;
  providerStatus: string;
  detectedBinary?: string | null;
  health: string;
  packageTools: string[];
  mirrors: string[];
  recommendedVersions: string[];
  capabilities: RuntimeCapabilities;
  notes: string[];
  installed: RuntimeInstallation[];
}

export interface SuggestedRuntime {
  family: string;
  version: string;
  reason: string;
}

export interface ProjectProfile {
  id: string;
  name: string;
  path: string;
  markers: string[];
  suggestedRuntimes: SuggestedRuntime[];
  riskFlags: string[];
  health: 'good' | 'attention';
}

export interface DotnetProjectPolicyOptions {
  rollForward?: string | null;
  allowPrerelease?: boolean | null;
}

export interface CppProjectPolicyOptions {
  generator?: string | null;
  binaryDir?: string | null;
  toolchainFile?: string | null;
  buildType?: string | null;
}

export interface ProjectRuntimePolicyOptions {
  dotnet?: DotnetProjectPolicyOptions | null;
  cpp?: CppProjectPolicyOptions | null;
}

export interface JobRecord {
  id: string;
  label: string;
  status: JobStatus;
  family?: string | null;
  version?: string | null;
  category?: string | null;
  targetName?: string | null;
  outcomeTitle?: string | null;
  outcomeDetail?: string | null;
  nextStep?: string | null;
  timestamp: string;
  progress?: number | null;
  progressLabel?: string | null;
}

export interface ExportBundle {
  fileName: string;
  payload: string;
}

export interface ImportResult {
  accepted: boolean;
  readyToApply: boolean;
  runtimeCount: number;
  hostCount: number;
  plannedCount: number;
  appliedCount: number;
  issues: string[];
  actions: ImportAction[];
}

export interface ImportAction {
  id: string;
  hostId: string;
  hostLabel: string;
  kind: string;
  family?: string | null;
  status: 'planned' | 'skipped' | 'blocked' | 'applied' | 'failed';
  selected: boolean;
  label: string;
  reason?: string | null;
  outcomeTitle?: string | null;
  nextStep?: string | null;
  remediation?: string | null;
  repairAvailable: boolean;
}

export interface AppPreferences {
  appliedMirrorPreset?: string | null;
  selectedHostId?: string | null;
  lastEnvTargetProfile?: string | null;
}

export interface SystemDependencyState {
  name: string;
  command: string;
  installed: boolean;
  version?: string | null;
  sourceHint: string;
}

export interface ServiceState {
  name: string;
  kind: string;
  installed: boolean;
  running: boolean;
  health: string;
  version?: string | null;
  manager: string;
  port?: number | null;
  dataDir?: string | null;
  notes: string[];
}

export interface ServiceConfigState {
  serviceName: string;
  configPath?: string | null;
  port?: number | null;
  dataDir?: string | null;
  canEditPort: boolean;
  canEditDataDir: boolean;
  notes: string[];
}

export interface ServiceArtifact {
  serviceName: string;
  kind: 'backup' | 'export' | 'logical-backup';
  path: string;
  sizeBytes?: number | null;
  createdAt?: string | null;
  managed: boolean;
}

export interface ProxySettings {
  enabled: boolean;
  scheme: string;
  host: string;
  port: string;
  username: string;
  passwordSaved: boolean;
  secureStore: string;
}

export interface EnvVariableSuggestion {
  key: string;
  value: string;
  reason: string;
}

export interface EnvProfileTarget {
  path: string;
  exists: boolean;
  managedByForgeEnv: boolean;
}

export interface EnvPlan {
  targetProfile: string;
  availableProfiles: EnvProfileTarget[];
  pathEntries: string[];
  variables: EnvVariableSuggestion[];
  managedBlock: string;
  notes: string[];
}
