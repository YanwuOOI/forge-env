import { useI18n } from '../../lib/hooks/useI18n';
import { cardClass, insetClass } from '../../lib/constants';
import type { Locale } from '../../lib/i18n';

const LANGUAGES: { value: Locale; label: string; native: string }[] = [
  { value: 'en', label: 'English', native: 'English' },
  { value: 'zh-CN', label: 'Chinese (Simplified)', native: '简体中文' },
];

export function LanguageSwitcher() {
  const { t, locale, setLocale } = useI18n();

  return (
    <div className={`${cardClass} p-5`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">
        {t('settings.envPolicy') !== 'settings.envPolicy' ? 'Language / 语言' : 'Language'}
      </p>
      <h3 className="mt-2 text-[18px] font-semibold text-[var(--text-primary)]">
        {locale === 'zh-CN' ? '选择界面语言' : 'Choose interface language'}
      </h3>
      <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
        {locale === 'zh-CN'
          ? '更改界面语言后立即生效，偏好设置保存在本地存储中。'
          : 'Language changes take effect immediately. Preference is stored in localStorage.'}
      </p>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        {LANGUAGES.map((lang) => (
          <button
            key={lang.value}
            type="button"
            onClick={() => setLocale(lang.value)}
            className={`${insetClass} flex items-center justify-between px-4 py-3 text-left transition-[transform,box-shadow] duration-[var(--motion-hover)] ease-[var(--ease-standard)] ${
              locale === lang.value
                ? 'ring-2 ring-[var(--focus-ring)]'
                : 'hover:-translate-y-px hover:shadow-[var(--shadow-raised-md)]'
            }`}
          >
            <div>
              <p className="text-[14px] font-semibold text-[var(--text-primary)]">{lang.native}</p>
              <p className="mt-1 text-[12px] text-[var(--text-secondary)]">{lang.label}</p>
            </div>
            {locale === lang.value ? (
              <span className="rounded-[var(--radius-pill)] bg-[var(--accent-soft)] px-3 py-1 text-[11px] font-semibold text-[var(--accent-primary)]">
                Active
              </span>
            ) : null}
          </button>
        ))}
      </div>
    </div>
  );
}
