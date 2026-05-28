import type { ImportAction, ImportResult, JobRecord, JobStatus, NavKey, ServiceArtifact, ServiceConfigState, ServiceState } from './types';

type JobFilterMode = 'relevant' | 'all';

export function confirmMutation(message: string) {
  if (typeof window === 'undefined' || typeof window.confirm !== 'function') {
    return true;
  }
  return window.confirm(message);
}

export function formatBytes(bytes: number) {
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

export function jobStatusClass(status: JobStatus) {
  if (status === 'failed') {
    return 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]';
  }
  if (status === 'queued' || status === 'running') {
    return 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]';
  }
  return 'bg-[var(--bg-elevated)] text-[var(--text-secondary)]';
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

export function filterJobsForView(jobs: JobRecord[], view: NavKey, mode: JobFilterMode) {
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

export function groupImportActionsByHost(actions: ImportAction[]) {
  const grouped = new Map<string, ImportAction[]>();
  actions.forEach((action) => {
    const existing = grouped.get(action.hostLabel) ?? [];
    existing.push(action);
    grouped.set(action.hostLabel, existing);
  });
  return Array.from(grouped.entries());
}

export function defaultSelectedImportActionIds(result: ImportResult) {
  return result.actions
    .filter((action) => action.status === 'planned' && action.selected)
    .map((action) => action.id);
}

export function normalizeOptionalText(value?: string | null) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

export function draftsFromServiceConfigs(configs: ServiceConfigState[]) {
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

export function draftsFromServiceArtifacts(services: ServiceState[], artifacts: ServiceArtifact[]) {
  return Object.fromEntries(
    services.map((service) => {
      const latestBackup = artifacts.find(
        (artifact) => artifact.serviceName === service.name && artifact.kind === 'backup',
      );
      return [service.name, latestBackup?.path ?? ''];
    }),
  );
}
