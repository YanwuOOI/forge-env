import { insetClass } from '../../lib/constants';

export function MetricTile({ label, value }: { label: string; value: string }) {
  return (
    <div className={`${insetClass} px-3 py-3`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{label}</p>
      <p className="mt-2 text-[18px] font-semibold text-[var(--text-primary)]">{value}</p>
    </div>
  );
}
