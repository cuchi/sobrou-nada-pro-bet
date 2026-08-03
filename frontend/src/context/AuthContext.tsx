import { createContext, useCallback, useContext, useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { googleLogin, fetchMe } from '../api/client';
import type { AuthResponse, GroupWithBalance, MeResponse, PublicUser } from '../types';

interface AuthState {
  user: PublicUser | null;
  token: string | null;
  groups: GroupWithBalance[];
  loading: boolean;
  loginError: string | null;
  login: (credential: string) => Promise<void>;
  logout: () => void;
  clearLoginError: () => void;
  addGroup: (g: GroupWithBalance) => void;
}

const AuthContext = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<PublicUser | null>(null);
  const [token, setToken] = useState<string | null>(() => localStorage.getItem('token'));
  const [groups, setGroups] = useState<GroupWithBalance[]>([]);
  const [loading, setLoading] = useState(true);
  const [loginError, setLoginError] = useState<string | null>(null);

  useEffect(() => {
    if (!token) {
      setLoading(false);
      return;
    }
    fetchMe()
      .then((data) => {
        const resp = data as MeResponse;
        setUser(resp.user);
        setGroups(resp.groups);
      })
      .catch(() => {
        localStorage.removeItem('token');
        setToken(null);
        setUser(null);
        setGroups([]);
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
      // Groups are empty on first login — user creates/joins them later
      setGroups([]);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Login failed';
      setLoginError(msg);
    }
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('token');
    setToken(null);
    setUser(null);
    setGroups([]);
    setLoginError(null);
  }, []);

  const clearLoginError = useCallback(() => setLoginError(null), []);
  const addGroup = useCallback(
    (g: GroupWithBalance) => setGroups((prev) => [...prev, g]),
    [],
  );

  return (
    <AuthContext.Provider
      value={{ user, token, groups, loading, loginError, login, logout, clearLoginError, addGroup }}
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
