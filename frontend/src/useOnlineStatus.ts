import { useEffect, useState } from 'react';

/**
 * Subscribe to the browser's online/offline events and return whether the
 * user currently has network connectivity. Initial value is
 * `navigator.onLine` (which most browsers set conservatively — false if
 * unknown). Re-renders on every change.
 *
 * Note: `navigator.onLine` reflects *link-layer* connectivity, not
 * reachability of our backend. A toast from a failed fetch is still the
 * right place for a per-request error; this hook is for the persistent
 * "you're offline" banner.
 */
export function useOnlineStatus(): boolean {
  const [online, setOnline] = useState<boolean>(() =>
    typeof navigator === 'undefined' ? true : navigator.onLine,
  );

  useEffect(() => {
    const goOnline = () => setOnline(true);
    const goOffline = () => setOnline(false);
    window.addEventListener('online', goOnline);
    window.addEventListener('offline', goOffline);
    return () => {
      window.removeEventListener('online', goOnline);
      window.removeEventListener('offline', goOffline);
    };
  }, []);

  return online;
}