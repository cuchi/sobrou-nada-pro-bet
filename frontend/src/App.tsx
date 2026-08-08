import { useCallback, useEffect, useState } from 'react';
import { GoogleOAuthProvider } from '@react-oauth/google';
import { useTranslation } from 'react-i18next';
import { fetchBets } from './api/client';
import { AuthProvider, useAuth } from './context/AuthContext';
import { ToastProvider } from './components/Toast';
import type { Bet } from './types';
import BetForm from './components/BetForm';
import BetList from './components/BetList';
import GoogleLoginButton from './components/GoogleLoginButton';
import DevLoginButton from './components/DevLoginButton';
import GroupSwitcher from './components/GroupSwitcher';
import Leaderboard from './components/Leaderboard';
import { LanguageSwitcher } from './components/LanguageSwitcher';
import { UserMenu } from './components/UserMenu';
import EmptyState from './components/EmptyState';
import { OfflineBanner } from './components/OfflineBanner';
import { usePolling } from './usePolling';
import './i18n';
import './App.css';

const GOOGLE_CLIENT_ID = import.meta.env.VITE_GOOGLE_CLIENT_ID || '';

function getGroupFromUrl(): string | null {
  return new URLSearchParams(window.location.search).get('group');
}

function setGroupInUrl(groupId: string | null) {
  const url = new URL(window.location.href);
  if (groupId) {
    url.searchParams.set('group', groupId);
  } else {
    url.searchParams.delete('group');
  }
  window.history.replaceState(null, '', url.toString());
}

function AppContent() {
  const { t, i18n } = useTranslation();
  const { user, groups, loading, loginError, clearLoginError } = useAuth();
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(getGroupFromUrl);
  const [betsRefreshKey, setBetsRefreshKey] = useState(0);

  const [bets, _betsLoading, setBets] = usePolling<Bet[]>(
    useCallback(async () => {
      if (!user || !selectedGroupId) return [];
      return (await fetchBets(selectedGroupId)) as Bet[];
    }, [user, selectedGroupId, betsRefreshKey]),
    60_000,
  );
  const [backendStatus, setBackendStatus] = useState<string>('checking');
  const [tick, setTick] = useState(0);

  const selectedGroup = groups.find((g) => g.id === selectedGroupId);

  // If the URL group is stale (e.g. removed membership), clear it
  useEffect(() => {
    if (selectedGroupId && groups.length > 0 && !selectedGroup) {
      setSelectedGroupId(null);
    }
  }, [selectedGroupId, groups, selectedGroup]);

  const handleGroupSelect = useCallback((id: string | null) => {
    setSelectedGroupId(id);
    setGroupInUrl(id);
  }, []);

  useEffect(() => {
    fetch('/health')
      .then((r) => r.json())
      .then((d) => setBackendStatus(d.status))
      .catch(() => setBackendStatus('offline'));
  }, []);

  const statusLabel = t(
    `footer.backendStatus.${backendStatus === 'ok' ? 'ok' : backendStatus === 'offline' ? 'offline' : 'checking'}`,
  );

  return (
    <div className="app">
      <header>
        <h1>
                  <img src="/brand/logo.svg" alt="" className="app-logo" />
                  <span>{t('app.title')}</span>
                </h1>
        <div className="header-right">
          {user || loading ? (
            // UserMenu owns its own loading state: it renders a compact
            // disabled trigger with a spinner while AuthProvider resolves
            // /api/auth/me, and the real chip+trigger once the user is in.
            // When loading is done and there's still no user, it bails out
            // to null and the logged-out branch below takes over.
            <UserMenu />
          ) : (
            <>
              <GoogleLoginButton />
              {import.meta.env.DEV && <DevLoginButton />}
              <LanguageSwitcher />
            </>
          )}
        </div>
      </header>

      <OfflineBanner />

      {user && (
        <GroupSwitcher selectedGroupId={selectedGroupId} onSelect={handleGroupSelect} />
      )}

      {loginError && (
        <div className="login-error-banner">
          <span>{loginError}</span>
          <button onClick={clearLoginError} className="banner-dismiss">
            ×
          </button>
        </div>
      )}

      <main>
        {user && selectedGroup ? (
          <>
            <BetForm
              groupId={selectedGroup.id}
              groupName={selectedGroup.name}
              balance={selectedGroup.balance}
              bets={bets ?? []}
              onBetCreated={(optimistic) => {
                setBets(prev => [optimistic, ...(prev ?? [])]);
              }}
              onBetSettled={() => {
                setBetsRefreshKey(k => k + 1);
                setTick(t => t + 1);
              }}
              onBetFailed={() => {
                setBets(prev => prev ? prev.filter(b => !b.id.startsWith('optimistic-')) : []);
                setBetsRefreshKey(k => k + 1);
                setTick(t => t + 1);
              }}
            />
            <Leaderboard groupId={selectedGroup.id} refreshKey={tick} />
          </>
        ) : user ? (
          groups.length === 0 ? (
            <EmptyState
              icon="groups"
              title={t('app.noGroupsPrompt')}
              hint={t('app.noGroupsHint')}
            />
          ) : (
            <EmptyState
              icon="ball"
              title={t('app.selectGroupPrompt')}
            />
          )
        ) : (
          <p className="login-prompt">{t('app.loginPrompt')}</p>
        )}
        {user && selectedGroup && <BetList bets={bets ?? []} />}
      </main>

      <footer className="app-footer">
        <a
          href="https://github.com/cuchi/sobrou-nada-pro-bet"
          target="_blank"
          rel="noopener noreferrer"
          className="footer-link"
        >
          GitHub
        </a>
        <span className="footer-sep">·</span>
        <span className="footer-text">{t('footer.license')}</span>
        <span className="footer-sep">·</span>
        <span
          className={`backend-status ${backendStatus === 'ok' ? 'online' : backendStatus === 'offline' ? 'offline' : 'checking'}`}
          aria-live="polite"
        >
          {t('footer.backendStatus.label')} {statusLabel}
        </span>
        {/* i18n.language is referenced so unused-variable lint stays quiet on i18n;
            useful in devtools for confirming the active locale. */}
        <span data-active-lng={i18n.language} hidden />
      </footer>
    </div>
  );
}

function GoogleProviderShell({ children }: { children: React.ReactNode }) {
  // Pass our active locale through to Google so the iframe-rendered sign-in
  // button matches the UI. Google accepts BCP-47 codes (e.g. `pt-BR`,
  // `en`); these align with our `SUPPORTED_LANGUAGES` exactly.
  //
  // @react-oauth/google's useLoadGsiScript effect captures `locale` at mount
  // and only re-runs when `nonce` changes — so the GSI script's `?hl=` param
  // is frozen at first paint. We force a remount of the provider by keying it
  // on the locale, which re-injects the script with the new `hl`.
  const { i18n } = useTranslation();
  const locale = i18n.resolvedLanguage ?? 'en';
  return (
    <GoogleOAuthProvider key={locale} clientId={GOOGLE_CLIENT_ID} locale={locale}>
      {children}
    </GoogleOAuthProvider>
  );
}

export default function App() {
  return (
    <GoogleProviderShell>
      <AuthProvider>
        <ToastProvider>
          <AppContent />
        </ToastProvider>
      </AuthProvider>
    </GoogleProviderShell>
  );
}
