import type { RuntimeFamilyState } from '../../lib/types';

export function RuntimeHealthBadge({ runtime }: { runtime: RuntimeFamilyState }) {
  const tone =
    runtime.health === 'good'
      ? 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]'
      : runtime.health === 'missing'
        ? 'bg-[rgba(209,75,90,0.10)] text-[var(--danger)]'
        : 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]';

  return (
    <span className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] ${tone}`}>
      {runtime.providerStatus}
    </span>
  );
}
