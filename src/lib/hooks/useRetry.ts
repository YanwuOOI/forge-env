import { useCallback, useState } from 'react';

interface UseRetryOptions {
  maxRetries?: number;
  delayMs?: number;
}

interface UseRetryResult {
  retrying: boolean;
  retryCount: number;
  execute: (fn: () => Promise<unknown>) => Promise<boolean>;
  reset: () => void;
}

export function useRetry(options: UseRetryOptions = {}): UseRetryResult {
  const { maxRetries = 2, delayMs = 1000 } = options;
  const [retrying, setRetrying] = useState(false);
  const [retryCount, setRetryCount] = useState(0);

  const execute = useCallback(async (fn: () => Promise<unknown>): Promise<boolean> => {
    setRetrying(true);
    setRetryCount(0);

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        await fn();
        setRetrying(false);
        return true;
      } catch {
        if (attempt < maxRetries) {
          setRetryCount(attempt + 1);
          await new Promise((resolve) => setTimeout(resolve, delayMs * (attempt + 1)));
        }
      }
    }

    setRetrying(false);
    return false;
  }, [maxRetries, delayMs]);

  const reset = useCallback(() => {
    setRetrying(false);
    setRetryCount(0);
  }, []);

  return { retrying, retryCount, execute, reset };
}
