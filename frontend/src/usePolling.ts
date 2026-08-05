import { useEffect, useRef, useState, useCallback } from 'react';

/**
 * Polls a fetcher every `intervalMs`. On subsequent polls, if the data is
 * JSON-identical to the previous value, the state is **not** replaced — so
 * React won't re-render and the UI stays perfectly still.
 *
 * Returns [data, isLoading, setData] where:
 * - `isLoading` is true only on the very first fetch
 * - `setData` allows optimistic updates between polls
 */
export function usePolling<T>(
  fetcher: () => Promise<T>,
  intervalMs: number,
): [T | null, boolean, (value: T | ((prev: T | null) => T)) => void] {
  const [data, setData] = useState<T | null>(null);
  const [initialLoading, setInitialLoading] = useState(true);
  const latestRef = useRef<string | null>(null);
  const mountedRef = useRef(true);

  const tick = useCallback(async () => {
    try {
      const fresh = await fetcher();
      if (!mountedRef.current) return;
      const serialized = JSON.stringify(fresh);
      if (serialized !== latestRef.current) {
        latestRef.current = serialized;
        setData(fresh);
      }
    } catch {
      // Keep stale data on transient failures
    } finally {
      if (mountedRef.current) setInitialLoading(false);
    }
  }, [fetcher]);

  useEffect(() => {
    mountedRef.current = true;
    tick(); // immediate first fetch

    const id = setInterval(tick, intervalMs);
    return () => {
      mountedRef.current = false;
      clearInterval(id);
    };
  }, [tick, intervalMs]);

  // When caller does an optimistic update, sync the ref so polling
  // won't overwrite with identical data that was already optimistically applied.
  const setDataWrapped = useCallback(
    (value: T | ((prev: T | null) => T)) => {
      setData((prev) => {
        const next = typeof value === 'function' ? (value as (prev: T | null) => T)(prev) : value;
        latestRef.current = JSON.stringify(next);
        return next;
      });
    },
    [],
  );

  return [data, initialLoading, setDataWrapped];
}
