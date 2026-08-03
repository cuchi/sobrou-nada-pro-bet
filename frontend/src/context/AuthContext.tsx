import { createContext, useCallback, useContext, useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { googleLogin, fetchMe } from '../api/client';
import type { AuthResponse, PublicUser } from '../types';

interface AuthState {
  user: PublicUser | null;
  token: string | null;
  loading: boolean;
  loginError: string | null;
  login: (credential: string) => Promise<void>;
  logout: () => void;
  clearLoginError: () => void;
}

const AuthContext = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<PublicUser | null>(null);
  const [token, setToken] = useState<string | null>(() => localStorage.getItem('token'));
  const [loading, setLoading] = useState(true);
  const [loginError, setLoginError] = useState<string | null>(null);

  // On mount (or token change), validate the stored token
  useEffect(() => {
    if (!token) {
      setLoading(false);
      return;
    }
    fetchMe()
      .then((data) => {
        const u = (data as { user: PublicUser }).user ?? (data as PublicUser);
        setUser(u);
      })
      .catch(() => {
        localStorage.removeItem('token');
        setToken(null);
        setUser(null);
      })
      .finally(() => setLoading(false));
  }, [token]);

  const login = useCallback(async (credential: string) => {
    setLoginError(null);
    try {
      const data = (await googleLogin(credential)) as AuthResponse;
      localStorage.setItem('token', data.token);
      setToken(data.token);
      setUser(data.user);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Login failed';
      setLoginError(msg);
    }
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('token');
    setToken(null);
    setUser(null);
    setLoginError(null);
  }, []);

  const clearLoginError = useCallback(() => setLoginError(null), []);

  return (
    <AuthContext.Provider
      value={{ user, token, loading, loginError, login, logout, clearLoginError }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
