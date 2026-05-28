import type { HostDetail, HostSummary } from '../../lib/types';
import { cardClass, insetClass } from '../../lib/constants';
import { MetricPanel } from '../shared/MetricPanel';
import { MetricTile } from '../shared/MetricTile';

interface HostsSectionProps {
  hosts: HostSummary[];
  detail: HostDetail | null;
  selectedHostId: string;
  onSelectHost: (hostId: string) => void;
}

export function HostsSection({ hosts, detail, selectedHostId, onSelectHost }: HostsSectionProps) {
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
