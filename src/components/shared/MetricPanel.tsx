import { insetClass } from '../../lib/constants';

export function MetricPanel({ label, value, hint }: { label: string; value: string; hint: string }) {
  return (
    <div className={`${insetClass} p-4`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{label}</p>
      <p className="mt-3 text-[24px] font-semibold text-[var(--text-primary)]">{value}</p>
      <p className="mt-2 text-[12px] leading-5 text-[var(--text-secondary)]">{hint}</p>
    </div>
  );
}
