import { buttonPrimaryClass, buttonSecondaryClass } from '../../lib/constants';

interface UpdateBannerProps {
  version: string;
  body?: string;
  downloading: boolean;
  onInstall: () => void;
  onDismiss: () => void;
}

export function UpdateBanner({ version, body, downloading, onInstall, onDismiss }: UpdateBannerProps) {
  return (
    <div className="rounded-[var(--radius-md)] bg-[rgba(47,107,255,0.10)] px-4 py-3 text-[13px]">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="font-semibold text-[var(--accent-primary)]">
            Update available: v{version}
          </p>
          {body ? (
            <p className="mt-1 text-[12px] leading-5 text-[var(--text-secondary)]">{body}</p>
          ) : null}
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            className={buttonPrimaryClass}
            onClick={onInstall}
            disabled={downloading}
          >
            {downloading ? 'Downloading...' : 'Install & restart'}
          </button>
          <button type="button" className={buttonSecondaryClass} onClick={onDismiss}>
            Dismiss
          </button>
        </div>
      </div>
    </div>
  );
}
