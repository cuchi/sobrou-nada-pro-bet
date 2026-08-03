const BASE = '/api';

function authHeaders(): Record<string, string> {
  const token = localStorage.getItem('token');
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return headers;
}

// ── Bets ──────────────────────────────────────────────

export async function fetchBets(): Promise<unknown> {
  const res = await fetch(`${BASE}/bets`, { headers: authHeaders() });
  if (!res.ok) throw new Error('Failed to fetch bets');
  return res.json();
}

export async function createBet(req: { amount: number; odds: number }): Promise<unknown> {
  const res = await fetch(`${BASE}/bets`, {
    method: 'POST',
    headers: authHeaders(),
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error('Failed to create bet');
  return res.json();
}

export async function resolveBet(id: string, status: 'won' | 'lost'): Promise<unknown> {
  const res = await fetch(`${BASE}/bets/${id}/resolve`, {
    method: 'PATCH',
    headers: authHeaders(),
    body: JSON.stringify({ status }),
  });
  if (!res.ok) throw new Error('Failed to resolve bet');
  return res.json();
}

// ── Auth ──────────────────────────────────────────────

export async function googleLogin(credential: string): Promise<unknown> {
  const res = await fetch(`${BASE}/auth/google`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ credential }),
  });
  if (!res.ok) throw new Error('Google login failed');
  return res.json();
}

export async function fetchMe(): Promise<unknown> {
  const res = await fetch(`${BASE}/auth/me`, { headers: authHeaders() });
  if (!res.ok) throw new Error('Failed to fetch user');
  return res.json();
}
