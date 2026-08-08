import { useTranslation } from 'react-i18next';
import { useOnlineStatus } from '../useOnlineStatus';

/**
 * Persistent top-of-page banner shown when the browser reports it has no
 * network connectivity. Hidden when online. No close button — the banner
 * appears/disappears automatically as the OS flips the link state.
 */
export function OfflineBanner() {
  const online = useOnlineStatus();
  const { t } = useTranslation();
  if (online) return null;
  return (
    <div className="offline-banner" role="status" aria-live="polite">
      <span className="offline-banner-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
          <path d="M 2 8.5 Q 12 3 22 8.5" />
          <path d="M 5 12 Q 12 8.5 19 12" />
          <path d="M 8.5 15.5 Q 12 13.5 15.5 15.5" />
          <line x1="3" y1="3" x2="21" y2="21" />
        </svg>
      </span>
      <span className="offline-banner-text">{t('offline.message')}</span>
    </div>
  );
}