import type { Dispatch, SetStateAction } from 'react';
import type { ProxySettings } from '../../lib/types';
import { cardClass, insetClass, buttonPrimaryClass, buttonSecondaryClass } from '../../lib/constants';
import { useI18n } from '../../lib/hooks/useI18n';

interface ProxyPanelProps {
  proxySettings: ProxySettings;
  setProxySettings: Dispatch<SetStateAction<ProxySettings>>;
  proxyPasswordDraft: string;
  setProxyPasswordDraft: Dispatch<SetStateAction<string>>;
  onSaveProxySettings: () => void;
  onClearProxySettings: () => void;
}

export function ProxyPanel({
  proxySettings,
  setProxySettings,
  proxyPasswordDraft,
  setProxyPasswordDraft,
  onSaveProxySettings,
  onClearProxySettings,
}: ProxyPanelProps) {
  const { t } = useI18n();

  return (
    <div className={`${cardClass} p-5`}>
      <p className="text-[11px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{t('settings.proxyProfile')}</p>
      <h3 className="mt-2 text-[18px] font-semibold">{t('settings.storeCredentials')}</h3>
      <p className="mt-3 text-[13px] leading-6 text-[var(--text-secondary)]">
        Forge Env stores host, port, and username in its managed settings file, while the password is delegated to{' '}
        {proxySettings.secureStore}.
      </p>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <label className="flex items-center gap-3 rounded-[var(--radius-md)] bg-[rgba(220,231,255,0.38)] px-4 py-3 text-[12px] text-[var(--text-secondary)]">
          <input
            type="checkbox"
            checked={proxySettings.enabled}
            onChange={(event) =>
              setProxySettings((current) => ({
                ...current,
                enabled: event.target.checked,
              }))
            }
            className="size-4 rounded border border-[var(--border-soft)] accent-[var(--accent-primary)]"
          />
          <span>{t('settings.enableProxy')}</span>
        </label>
        <div className={`${insetClass} flex items-center justify-between gap-3 px-4 py-3`}>
          <div>
            <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('settings.credentialStore')}</p>
            <p className="mt-1 text-[13px] font-semibold text-[var(--text-primary)]">{proxySettings.secureStore}</p>
          </div>
          <span
            className={`rounded-[var(--radius-pill)] px-3 py-1 text-[11px] font-semibold ${
              proxySettings.passwordSaved
                ? 'bg-[rgba(31,157,104,0.12)] text-[var(--success)]'
                : 'bg-[rgba(209,138,29,0.14)] text-[var(--warning)]'
            }`}
          >
            {proxySettings.passwordSaved ? t('settings.passwordSaved') : t('settings.noPassword')}
          </span>
        </div>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('settings.proxyScheme')}</span>
          <select
            value={proxySettings.scheme}
            onChange={(event) =>
              setProxySettings((current) => ({ ...current, scheme: event.target.value }))
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none`}
          >
            <option value="http">http</option>
            <option value="https">https</option>
            <option value="socks5">socks5</option>
          </select>
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('settings.proxyHost')}</span>
          <input
            value={proxySettings.host}
            onChange={(event) =>
              setProxySettings((current) => ({ ...current, host: event.target.value }))
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder="127.0.0.1"
          />
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('settings.proxyPort')}</span>
          <input
            value={proxySettings.port}
            onChange={(event) =>
              setProxySettings((current) => ({ ...current, port: event.target.value }))
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder="7890"
          />
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('settings.proxyUsername')}</span>
          <input
            value={proxySettings.username}
            onChange={(event) =>
              setProxySettings((current) => ({ ...current, username: event.target.value }))
            }
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder="Optional proxy account"
          />
        </label>
        <label className="grid gap-2 text-[12px] text-[var(--text-secondary)]">
          <span className="font-semibold uppercase tracking-[0.12em] text-[var(--text-muted)]">{t('settings.proxyPassword')}</span>
          <input
            type="password"
            value={proxyPasswordDraft}
            onChange={(event) => setProxyPasswordDraft(event.target.value)}
            className={`${insetClass} px-3 py-3 text-[13px] text-[var(--text-primary)] outline-none placeholder:text-[var(--text-muted)]`}
            placeholder={
              proxySettings.passwordSaved ? 'Leave blank to keep saved password' : 'Stored in system vault'
            }
          />
        </label>
      </div>
      <div className="mt-4 flex flex-wrap gap-3">
        <button type="button" className={buttonPrimaryClass} onClick={onSaveProxySettings}>
          {t('settings.saveProxy')}
        </button>
        <button type="button" className={buttonSecondaryClass} onClick={onClearProxySettings}>
          {t('settings.clearProxy')}
        </button>
      </div>
    </div>
  );
}
