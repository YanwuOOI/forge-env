import { cardClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';

export function RuntimePolicyCard() {
  const { t } = useI18n();
  const policies: { key: string; label: string }[] = [
    { key: 'python', label: 'Python' },
    { key: 'node', label: 'Node.js' },
    { key: 'rust', label: 'Rust' },
    { key: 'java', label: 'Java' },
    { key: 'go', label: 'Go' },
    { key: 'dotnet', label: '.NET' },
    { key: 'php', label: 'PHP' },
    { key: 'ruby', label: 'Ruby' },
    { key: 'cpp', label: 'C/C++' },
  ];

  return (
    <div className={`${cardClass} p-4`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('runtimePolicy.runtimePolicy')}</p>
      <h3 className="mt-2 text-[16px] font-semibold">{t('runtimePolicy.canonicalChoices')}</h3>
      <ul className="mt-4 space-y-3 text-[13px] leading-6 text-[var(--text-secondary)]">
        {policies.map(({ key, label }) => (
          <li key={key}>
            <strong className="text-[var(--text-primary)]">{label}</strong>: {t(`runtimePolicy.${key}`)}
          </li>
        ))}
      </ul>
    </div>
  );
}
