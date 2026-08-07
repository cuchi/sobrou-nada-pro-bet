import i18n from '../i18n';

const BASE = '/api';

export class ApiError extends Error {
  constructor(
    public code: string,
    public params: Record<string, unknown> | null,
    public message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * Translate a backend snake_case error code into the camelCase locale key
 * defined in the Phase D contract. The wire codes are stable snake_case
 * identifiers (e.g. `insufficient_balance`); the locale files group them
 * under `errors.<CamelCase>` for consistency with the rest of the keys.
 */
export function codeToLocaleKey(code: string): string {
  return code.replace(/_([a-z0-9])/g, (_, ch: string) => ch.toUpperCase());
}

/**
 * Convert any thrown value into a user-facing, locale-aware error string.
 *
 * Use this at every render site that catches errors from one of the four
 * in-scope API callers (`googleLogin`, `devLogin`, `createBet`,
 * `joinGroup`). The flow:
 *   1. If `err` is an `ApiError`, look up `t('errors.<key>', params)` via
 *      `codeToLocaleKey`. i18next returns the key path itself when a
 *      translation is missing — fall back to `err.message` (English) so
 *      the UI is never blank.
 *   2. If `err` is a plain `Error`, surface `err.message`.
 *   3. Otherwise use `fallbackKey` (defaults to `'errors.internal'`).
 *
 * Centralising this here keeps render sites trivial and means future API
 * callers automatically get the right behaviour.
 */
export function translateApiError(
  err: unknown,
  t: (key: string, params?: Record<string, unknown>) => string,
  fallbackKey: string = 'errors.internal',
): string {
  if (err instanceof ApiError) {
    const key = `errors.${codeToLocaleKey(err.code)}`;
    const translated = t(key, err.params ?? {});
    return translated === key ? err.message : translated;
  }
  if (err instanceof Error) return err.message;
  return t(fallbackKey);
}

function authHeaders(): Record<string, string> {
  const token = localStorage.getItem('token');
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return headers;
}

// ── Auth ──────────────────────────────────────────────

export async function googleLogin(credential: string): Promise<unknown> {
  const res = await fetch(`${BASE}/auth/google`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ credential }),
  });
  const data = await res.json();
  if (!res.ok) {
    const err = data as {
      code?: string;
      params?: Record<string, unknown> | null;
      message?: string;
    };
    throw new ApiError(
      err.code ?? 'auth_google_failed',
      err.params ?? null,
      err.message ?? i18n.t('errors.authGoogleFailed'),
    );
  }
  return data;
}

export async function devLogin(): Promise<unknown> {
  const res = await fetch(`${BASE}/dev/login`, { method: 'POST' });
  const data = await res.json();
  if (!res.ok) {
    const err = data as {
      code?: string;
      params?: Record<string, unknown> | null;
      message?: string;
    };
    throw new ApiError(
      err.code ?? 'internal',
      err.params ?? null,
      err.message ?? i18n.t('errors.internal'),
    );
  }
  return data;
}

export async function fetchMe(): Promise<unknown> {
  const res = await fetch(`${BASE}/auth/me`, { headers: authHeaders() });
  if (!res.ok) throw new Error(i18n.t('errors.fetchUser'));
  return res.json();
}

// ── Bets ──────────────────────────────────────────────

export async function fetchBets(groupId: string): Promise<unknown> {
  const res = await fetch(`${BASE}/bets?group_id=${groupId}`, { headers: authHeaders() });
  if (!res.ok) throw new Error(i18n.t('errors.fetchBets'));
  return res.json();
}

export async function createBet(req: {
  group_id: string;
  event_id: string;
  prediction: string;
  amount: number;
  odds: number;
}): Promise<unknown> {
  const res = await fetch(`${BASE}/bets`, {
    method: 'POST',
    headers: authHeaders(),
    body: JSON.stringify(req),
  });
  const data = await res.json();
  if (!res.ok) {
    const err = data as {
      code?: string;
      params?: Record<string, unknown> | null;
      message?: string;
    };
    throw new ApiError(
      err.code ?? 'internal',
      err.params ?? null,
      err.message ?? i18n.t('errors.internal'),
    );
  }
  return data;
}

// ── Groups ────────────────────────────────────────────

export async function createGroup(name: string): Promise<unknown> {
  const res = await fetch(`${BASE}/groups`, {
    method: 'POST',
    headers: authHeaders(),
    body: JSON.stringify({ name }),
  });
  if (!res.ok) throw new Error(i18n.t('errors.createGroup'));
  return res.json();
}

export async function joinGroup(inviteCode: string): Promise<unknown> {
  const res = await fetch(`${BASE}/groups/join/${inviteCode}`, {
    method: 'POST',
    headers: authHeaders(),
  });
  const data = await res.json();
  if (!res.ok) {
    const err = data as {
      code?: string;
      params?: Record<string, unknown> | null;
      message?: string;
    };
    throw new ApiError(
      err.code ?? 'internal',
      err.params ?? null,
      err.message ?? i18n.t('errors.internal'),
    );
  }
  return data;
}

export async function getInviteCode(groupId: string): Promise<unknown> {
  const res = await fetch(`${BASE}/groups/${groupId}/invite`, { headers: authHeaders() });
  if (!res.ok) throw new Error(i18n.t('errors.getInvite'));
  return res.json();
}

export async function fetchLeaderboard(groupId: string): Promise<unknown> {
  const res = await fetch(`${BASE}/groups/${groupId}/leaderboard`, { headers: authHeaders() });
  if (!res.ok) throw new Error(i18n.t('errors.fetchLeaderboard'));
  return res.json();
}

// ── Events ────────────────────────────────────────────

export async function fetchEvents(status?: string): Promise<unknown> {
  const url = status ? `${BASE}/events?status=${status}` : `${BASE}/events`;
  const res = await fetch(url, { headers: authHeaders() });
  if (!res.ok) throw new Error(i18n.t('errors.fetchEvents'));
  return res.json();
}
