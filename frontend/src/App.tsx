import { useCallback, useEffect, useState } from 'react';
import { GoogleOAuthProvider } from '@react-oauth/google';
import { fetchBets } from './api/client';
import { AuthProvider, useAuth } from './context/AuthContext';
import type { Bet } from './types';
import BetForm from './components/BetForm';
import BetList from './components/BetList';
import GoogleLoginButton from './components/GoogleLoginButton';
import './App.css';

const GOOGLE_CLIENT_ID = import.meta.env.VITE_GOOGLE_CLIENT_ID || '';

function AppContent() {
  const { user, loading, logout } = useAuth();
  const [bets, setBets] = useState<Bet[]>([]);
  const [backendStatus, setBackendStatus] = useState<string>('checking...');

  const loadBets = useCallback(async () => {
    try {
      const data = (await fetchBets()) as Bet[];
      setBets(data);
    } catch {
      console.error('Failed to load bets');
    }
  }, []);

  useEffect(() => {
    fetch('/health')
      .then((r) => r.json())
      .then((d) => setBackendStatus(d.status))
      .catch(() => setBackendStatus('offline'));
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
            <GoogleLoginButton />
          )}
        </div>
      </header>
      <main>
        {user ? (
          <BetForm onBetCreated={loadBets} />
        ) : (
          <p className="login-prompt">Sign in with Google to place bets</p>
        )}
        <BetList bets={bets} onUpdate={loadBets} />
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
