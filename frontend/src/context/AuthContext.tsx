import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { googleLogin, fetchMe, patchMe, translateApiError } from '../api/client';
import type { AuthResponse, GroupWithBalance, MeResponse, PatchMeRequest, PublicUser } from '../types';

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
  /// Optimistically apply `patch` to the local user, fire PATCH /api/me,
  /// and roll back if the call fails. Returns the server-confirmed user
  /// (which may differ from the optimistic value if the server normalised
  /// something) or null if there's no signed-in user.
  updateUser: (patch: PatchMeRequest) => Promise<PublicUser | null>;
}

const AuthContext = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
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
      setLoginError(translateApiError(err, t, 'errors.googleLogin'));
    }
  }, [t]);

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

  // Locale syncing: when the user picks a new language in the menu,
  // push it to /api/me so the next email/digest picks the right
  // template. The ref guards against redundant syncs (e.g. on initial
  // mount when i18next fires languageChanged to the already-stored
  // locale).
  const lastSyncedLocale = useRef<string | null>(null);
  const { i18n } = useTranslation();
  useEffect(() => {
    const onChange = (lng: string) => {
      if (!token) return;
      if (lastSyncedLocale.current === lng) return;
      lastSyncedLocale.current = lng;
      patchMe({ locale: lng }).catch(() => {
        // Roll back the synced marker so the next change retries.
        lastSyncedLocale.current = null;
        // eslint-disable-next-line no-console
        console.error('Failed to sync locale to backend');
      });
    };
    i18n.on('languageChanged', onChange);
    return () => {
      i18n.off('languageChanged', onChange);
    };
  }, [i18n, token]);

  const updateUser = useCallback(
    async (patch: PatchMeRequest): Promise<PublicUser | null> => {
      if (!user) return null;
      const previous = user;
      // Optimistic local update.
      const optimistic: PublicUser = { ...user, ...patch };
      setUser(optimistic);
      try {
        const confirmed = await patchMe(patch);
        setUser(confirmed);
        if (patch.locale !== undefined) {
          lastSyncedLocale.current = confirmed.locale;
        }
        return confirmed;
      } catch (e) {
        // Roll back on failure.
        setUser(previous);
        throw e;
      }
    },
    [user],
  );

  return (
    <AuthContext.Provider
      value={{
        user,
        token,
        groups,
        loading,
        loginError,
        login,
        logout,
        clearLoginError,
        addGroup,
        updateUser,
      }}
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
