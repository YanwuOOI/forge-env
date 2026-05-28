import type { HostSummary, JobRecord, RuntimeFamilyState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass } from '../../lib/constants';
import { MetricPanel } from '../shared/MetricPanel';
import { RuntimeHealthBadge } from '../shared/RuntimeHealthBadge';

interface OverviewSectionProps {
  hosts: HostSummary[];
  runtimes: RuntimeFamilyState[];
  jobs: JobRecord[];
  onMirrorApply: () => void;
  mirrorPreset: string;
  setMirrorPreset: (value: string) => void;
}

export function OverviewSection({
  hosts,
  runtimes,
  jobs,
  onMirrorApply,
  mirrorPreset,
  setMirrorPreset,
}: OverviewSectionProps) {
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
