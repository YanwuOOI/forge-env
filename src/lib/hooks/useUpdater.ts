import { useCallback, useEffect, useRef, useState } from 'react';
import { isTauri } from '@tauri-apps/api/core';

interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

export function useUpdater() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const updateRef = useRef<{ downloadAndInstall: () => Promise<void> } | null>(null);

  const checkForUpdates = useCallback(async () => {
    if (!isTauri()) return;

    setChecking(true);
    setError(null);

    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        updateRef.current = update;
        setUpdateInfo({
          version: update.version,
          date: update.date,
          body: update.body,
        });
      } else {
        updateRef.current = null;
        setUpdateInfo(null);
      }
    } catch {
      // Updater not configured or network error — silently ignore
    } finally {
      setChecking(false);
    }
  }, []);

  const installUpdate = useCallback(async () => {
    if (!isTauri() || !updateRef.current) return;

    setDownloading(true);
    try {
      await updateRef.current.downloadAndInstall();
      // App will restart after installation
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Update installation failed');
      setDownloading(false);
    }
  }, []);

  const dismissUpdate = useCallback(() => {
    setUpdateInfo(null);
    updateRef.current = null;
  }, []);

  useEffect(() => {
    void checkForUpdates();
  }, [checkForUpdates]);

  return {
    updateInfo,
    checking,
    downloading,
    error,
    checkForUpdates,
    installUpdate,
    dismissUpdate,
  };
}
