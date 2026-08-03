import { useCallback, useEffect, useState } from 'react';
import { GoogleOAuthProvider } from '@react-oauth/google';
import { fetchBets } from './api/client';
import { AuthProvider, useAuth } from './context/AuthContext';
import type { Bet } from './types';
import BetForm from './components/BetForm';
import BetList from './components/BetList';
import GoogleLoginButton from './components/GoogleLoginButton';
import DevLoginButton from './components/DevLoginButton';
import GroupSwitcher from './components/GroupSwitcher';
import Leaderboard from './components/Leaderboard';
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
  const { user, groups, loading, loginError, clearLoginError, logout } = useAuth();
  const [bets, setBets] = useState<Bet[]>([]);
  const [backendStatus, setBackendStatus] = useState<string>('checking...');
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(getGroupFromUrl);

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

  const loadBets = useCallback(async () => {
    if (!user) {
      setBets([]);
      return;
    }
    try {
      const data = (await fetchBets(selectedGroupId ?? undefined)) as Bet[];
      setBets(data);
    } catch {
      console.error('Failed to load bets');
    }
  }, [user, selectedGroupId]);

  useEffect(() => {
    fetch('/health')
      .then((r) => r.json())
      .then((d) => setBackendStatus(d.status))
      .catch(() => setBackendStatus('offline'));
  }, []);

  useEffect(() => {
    loadBets();
  }, [loadBets]);

  return (
    <div className="app">
      <header>
        <h1>🎲 Sobrou Nada Pro Bet</h1>
        <div className="header-right">
          <span
            className={`backend-status ${backendStatus === 'ok' ? 'online' : 'offline'}`}
          >
            Backend: {backendStatus}
          </span>
          {loading ? (
            <span className="auth-loading">Loading…</span>
          ) : user ? (
            <div className="user-info">
              {user.avatar_url && (
                <img src={user.avatar_url} alt="" className="avatar" />
              )}
              <span className="user-name">{user.name}</span>
              <button onClick={logout} className="btn-logout">
                Logout
              </button>
            </div>
          ) : (
            <>
              <GoogleLoginButton />
              {import.meta.env.DEV && <DevLoginButton />}
            </>
          )}
        </div>
      </header>

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
              onBetCreated={loadBets}
            />
            <Leaderboard groupId={selectedGroup.id} />
          </>
        ) : user ? (
          <p className="login-prompt">
            {groups.length === 0
              ? 'Create or join a group to start betting'
              : 'Select a group above to place bets'}
          </p>
        ) : (
          <p className="login-prompt">Sign in with Google to place bets</p>
        )}
        {user && <BetList bets={bets} onUpdate={loadBets} />}
      </main>
    </div>
  );
}

export default function App() {
  return (
    <GoogleOAuthProvider clientId={GOOGLE_CLIENT_ID}>
      <AuthProvider>
        <AppContent />
      </AuthProvider>
    </GoogleOAuthProvider>
  );
}
