import type { NavKey, JobRecord } from '../../lib/types';
import { navItems, cardClass, insetClass, buttonSecondaryClass } from '../../lib/constants';
import { filterJobsForView, jobStatusClass } from '../../lib/utils';

type JobFilterMode = 'relevant' | 'all';

interface JobQueueProps {
  jobs: JobRecord[];
  activeView: NavKey;
  jobFilterMode: JobFilterMode;
  setJobFilterMode: (mode: JobFilterMode) => void;
  pendingJobs: JobRecord[];
}

export function JobQueue({ jobs, activeView, jobFilterMode, setJobFilterMode, pendingJobs }: JobQueueProps) {
  const visibleJobs = filterJobsForView(jobs, activeView, jobFilterMode);
  const completedJobs = visibleJobs.filter((job) => job.status === 'completed').slice(0, 4);
  const relevantJobCount = filterJobsForView(jobs, activeView, 'relevant').length;

  return (
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
                <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">{job.outcomeDetail}</p>
              ) : (
                <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">{job.label}</p>
              )}
              {job.nextStep ? (
                <p className="mt-2 text-[12px] leading-5 text-[var(--text-muted)]">Next: {job.nextStep}</p>
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
  );
}
