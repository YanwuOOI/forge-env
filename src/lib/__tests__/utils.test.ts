import { describe, it, expect, vi } from 'vitest';
import {
  formatBytes,
  jobStatusClass,
  filterJobsForView,
  groupImportActionsByHost,
  defaultSelectedImportActionIds,
  normalizeOptionalText,
  draftsFromServiceConfigs,
  draftsFromServiceArtifacts,
  confirmMutation,
} from '../utils';
import type { JobRecord, ImportAction, ImportResult, ServiceConfigState, ServiceState, ServiceArtifact } from '../types';

describe('formatBytes', () => {
  it('formats bytes', () => {
    expect(formatBytes(500)).toBe('500 B');
  });

  it('formats kilobytes', () => {
    expect(formatBytes(1536)).toBe('1.5 KB');
  });

  it('formats megabytes', () => {
    expect(formatBytes(2621440)).toBe('2.5 MB');
  });

  it('formats gigabytes', () => {
    expect(formatBytes(2684354560)).toBe('2.5 GB');
  });
});

describe('jobStatusClass', () => {
  it('returns danger class for failed', () => {
    expect(jobStatusClass('failed')).toContain('danger');
  });

  it('returns warning class for queued', () => {
    expect(jobStatusClass('queued')).toContain('warning');
  });

  it('returns warning class for running', () => {
    expect(jobStatusClass('running')).toContain('warning');
  });

  it('returns secondary class for completed', () => {
    expect(jobStatusClass('completed')).toContain('text-secondary');
  });
});

describe('filterJobsForView', () => {
  const jobs = [
    { id: '1', label: 'Install Python', category: 'runtime', status: 'completed' },
    { id: '2', label: 'Apply mirror', category: 'mirror', status: 'completed' },
    { id: '3', label: 'Install deps', category: 'dependency', status: 'completed' },
    { id: '4', label: 'Start Redis', category: 'service', status: 'completed' },
  ] as JobRecord[];

  it('returns all jobs in "all" mode', () => {
    expect(filterJobsForView(jobs, 'overview', 'all')).toHaveLength(4);
  });

  it('filters by view-relevant categories', () => {
    const filtered = filterJobsForView(jobs, 'languages', 'relevant');
    expect(filtered).toHaveLength(1);
    expect(filtered[0].category).toBe('runtime');
  });

  it('returns all jobs when no categories match (overview)', () => {
    const filtered = filterJobsForView(jobs, 'overview', 'relevant');
    expect(filtered).toHaveLength(4);
  });
});

describe('groupImportActionsByHost', () => {
  it('groups actions by hostLabel', () => {
    const actions = [
      { id: '1', hostLabel: 'macOS', status: 'planned' },
      { id: '2', hostLabel: 'Ubuntu', status: 'planned' },
      { id: '3', hostLabel: 'macOS', status: 'applied' },
    ] as ImportAction[];

    const grouped = groupImportActionsByHost(actions);
    expect(grouped).toHaveLength(2);
    expect(grouped[0][0]).toBe('macOS');
    expect(grouped[0][1]).toHaveLength(2);
    expect(grouped[1][0]).toBe('Ubuntu');
    expect(grouped[1][1]).toHaveLength(1);
  });
});

describe('defaultSelectedImportActionIds', () => {
  it('selects planned actions marked as selected', () => {
    const result = {
      actions: [
        { id: 'a', status: 'planned', selected: true },
        { id: 'b', status: 'planned', selected: false },
        { id: 'c', status: 'applied', selected: true },
      ],
    } as ImportResult;

    expect(defaultSelectedImportActionIds(result)).toEqual(['a']);
  });
});

describe('normalizeOptionalText', () => {
  it('returns undefined for empty string', () => {
    expect(normalizeOptionalText('')).toBeUndefined();
  });

  it('returns undefined for whitespace', () => {
    expect(normalizeOptionalText('   ')).toBeUndefined();
  });

  it('returns undefined for null/undefined', () => {
    expect(normalizeOptionalText(null)).toBeUndefined();
    expect(normalizeOptionalText(undefined)).toBeUndefined();
  });

  it('trims and returns non-empty value', () => {
    expect(normalizeOptionalText('  hello  ')).toBe('hello');
  });
});

describe('draftsFromServiceConfigs', () => {
  it('creates draft map from configs', () => {
    const configs = [
      { serviceName: 'Redis', port: 6379, dataDir: '/var/lib/redis' },
      { serviceName: 'PostgreSQL', port: null, dataDir: null },
    ] as ServiceConfigState[];

    const drafts = draftsFromServiceConfigs(configs);
    expect(drafts['Redis']).toEqual({ port: '6379', dataDir: '/var/lib/redis' });
    expect(drafts['PostgreSQL']).toEqual({ port: '', dataDir: '' });
  });
});

describe('draftsFromServiceArtifacts', () => {
  it('maps latest backup path per service', () => {
    const services = [
      { name: 'Redis' },
      { name: 'PostgreSQL' },
    ] as ServiceState[];
    const artifacts = [
      { serviceName: 'Redis', kind: 'backup', path: '/backups/redis-1.tar.gz' },
      { serviceName: 'Redis', kind: 'backup', path: '/backups/redis-2.tar.gz' },
      { serviceName: 'PostgreSQL', kind: 'export', path: '/exports/pg.tar.gz' },
    ] as ServiceArtifact[];

    const drafts = draftsFromServiceArtifacts(services, artifacts);
    expect(drafts['Redis']).toBe('/backups/redis-1.tar.gz');
    expect(drafts['PostgreSQL']).toBe('');
  });
});

describe('confirmMutation', () => {
  it('returns true when window.confirm returns true', () => {
    vi.stubGlobal('confirm', vi.fn(() => true));
    expect(confirmMutation('test')).toBe(true);
  });

  it('returns false when window.confirm returns false', () => {
    vi.stubGlobal('confirm', vi.fn(() => false));
    expect(confirmMutation('test')).toBe(false);
  });

  it('returns true when window.confirm is unavailable', () => {
    vi.stubGlobal('confirm', undefined);
    expect(confirmMutation('test')).toBe(true);
  });
});
