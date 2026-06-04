import { isTauri } from '@tauri-apps/api/core';

interface ConnectionStatusProps {
  onReconnect?: () => void; // Reserved for future use
}

export function ConnectionStatus({ onReconnect: _onReconnect }: ConnectionStatusProps) {
  if (isTauri()) return null;

  return (
    <div className="rounded-[var(--radius-md)] bg-[rgba(209,138,29,0.12)] px-4 py-3 text-[13px]" role="status">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <span className="size-2 rounded-full bg-[var(--warning)]" />
          <p className="font-semibold text-[var(--warning)]">Browser preview mode</p>
        </div>
        <p className="text-[12px] text-[var(--text-secondary)]">
          Running with mock data. Some features require the Tauri desktop app.
        </p>
      </div>
    </div>
  );
}
