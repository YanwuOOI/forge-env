import { memo } from 'react';
import type { HostSummary, JobRecord, RuntimeFamilyState } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';
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

const mirrorOptions = ['Tsinghua', 'Aliyun', 'Huawei Cloud', 'Company Proxy'];

export const OverviewSection = memo(function OverviewSection({
  hosts,
  runtimes,
  jobs,
  onMirrorApply,
  mirrorPreset,
  setMirrorPreset,
}: OverviewSectionProps) {
  const { t } = useI18n();

  return (
    <div className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
      <div className="space-y-4">
        <div className={`${cardClass} p-5`}>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('overview.workspacePulse')}</p>
              <h3 className="mt-2 text-[18px] font-semibold">{t('overview.systemLandscape')}</h3>
            </div>
            <div className={`${insetClass} flex items-center gap-2 px-3 py-2`}>
              <span className="size-2 rounded-full bg-[var(--success)]" />
              <span className="text-[12px] font-semibold text-[var(--text-secondary)]">
                {hosts.length} {t('overview.hostLayers')}
              </span>
            </div>
          </div>

          <div className="mt-5 grid gap-3 md:grid-cols-3">
            <MetricPanel label={t('overview.detectedHosts')} value={String(hosts.length)} hint={t('overview.hostsHint')} />
            <MetricPanel
              label={t('overview.managedRuntimes')}
              value={String(runtimes.reduce((sum, runtime) => sum + runtime.installed.length, 0))}
              hint={t('overview.runtimesHint')}
            />
            <MetricPanel
              label={t('overview.recentJobs')}
              value={String(jobs.length)}
              hint={t('overview.jobsHint')}
            />
          </div>
        </div>

        <div className={`${cardClass} p-5`}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('overview.runtimeFocus')}</p>
              <h3 className="mt-2 text-[18px] font-semibold">{t('overview.defaultVersions')}</h3>
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
                    {active?.version ?? t('overview.notInstalled')}
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
          <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('overview.mirrorPreset')}</p>
          <h3 className="mt-2 text-[18px] font-semibold">{t('overview.applyDomesticMirrors')}</h3>
          <div className={`${insetClass} mt-5 p-4`}>
            <label className="block text-[12px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">
              {t('overview.preset')}
            </label>
            <select
              value={mirrorPreset}
              onChange={(event) => setMirrorPreset(event.target.value)}
              className="mt-3 w-full appearance-none rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-elevated)] px-4 py-3 text-[14px] text-[var(--text-primary)] outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
            >
              {mirrorOptions.map((opt) => (
                <option key={opt}>{opt}</option>
              ))}
            </select>
            <button type="button" className={`${buttonPrimaryClass} mt-4 w-full`} onClick={onMirrorApply}>
              {t('overview.applyPreset')}
            </button>
          </div>
        </div>

        <div className={`${cardClass} p-5`}>
          <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('overview.architectureNote')}</p>
          <h3 className="mt-2 text-[18px] font-semibold">{t('overview.whyCompact')}</h3>
          <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
            {t('overview.architectureDesc')}
          </p>
        </div>
      </div>
    </div>
  );
});
