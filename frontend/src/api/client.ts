import i18n from '../i18n';

const BASE = '/api';

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
    const msg = (data as { error?: string }).error || i18n.t('errors.googleLogin');
    throw new Error(msg);
  }
  return data;
}

export async function devLogin(): Promise<unknown> {
  const res = await fetch(`${BASE}/dev/login`, { method: 'POST' });
  const data = await res.json();
  if (!res.ok) {
    const msg = (data as { error?: string }).error || i18n.t('errors.devLogin');
    throw new Error(msg);
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
    const msg = (data as { error?: string }).error || i18n.t('errors.createBet');
    throw new Error(msg);
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
    const msg = (data as { error?: string }).error || i18n.t('errors.joinGroup');
    throw new Error(msg);
  }
  return res.json();
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
