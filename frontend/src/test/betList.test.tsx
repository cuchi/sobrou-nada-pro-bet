import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n, { DEFAULT_LANGUAGE } from '../i18n';
import BetList from '../components/BetList';
import type { Bet } from '../types';

function makeBet(overrides: Partial<Bet> = {}): Bet {
  return {
    id: 'bet-1',
    user_id: 'u1',
    group_id: 'g1',
    event_id: 'e1',
    prediction: 'home_win',
    amount: 100,
    odds: 1.5,
    status: 'pending',
    created_at: '2026-08-07T19:00:00Z',
    user_name: 'Kaká',
    user_email: 'kaka@example.com',
    user_avatar_url: null,
    home_team: 'Flamengo',
    away_team: 'Vasco',
    ...overrides,
  };
}

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe('BetList — dev resolve column', () => {
  it('renders three resolve buttons on each pending row and fires the endpoint on click', async () => {
    const bet = makeBet();
    const fetchSpy = vi.fn(async (url: string | URL, init?: RequestInit) => {
      if (String(url).endsWith('/api/dev/resolve-bet') && init?.method === 'POST') {
        return new Response(
          JSON.stringify({
            bet_id: 'bet-1',
            outcome: 'home_win',
            score: '1–0',
            resolved: 1,
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        );
      }
      return new Response('{}', { status: 404 });
    });
    globalThis.fetch = fetchSpy as unknown as typeof fetch;

    const onBetResolved = vi.fn();
    const { container, getAllByRole } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={[bet]} onBetResolved={onBetResolved} />
      </I18nextProvider>,
    );

    await waitFor(() => {
      // Three buttons per row: home, draw, away
      expect(getAllByRole('button').length).toBeGreaterThanOrEqual(3);
    });

    const homeBtn = container.querySelector('.bet-resolve-btn');
    expect(homeBtn).toBeTruthy();
    fireEvent.click(homeBtn!);

    await waitFor(() => {
      expect(fetchSpy).toHaveBeenCalled();
      const call = fetchSpy.mock.calls.find(
        ([url, init]) =>
          String(url).endsWith('/api/dev/resolve-bet') &&
          (init as RequestInit | undefined)?.method === 'POST',
      );
      expect(call).toBeTruthy();
      expect(onBetResolved).toHaveBeenCalled();
    });
  });

  it('shows a placeholder dash on resolved rows instead of buttons', async () => {
    const wonBet = makeBet({ id: 'bet-2', status: 'won' });
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={[wonBet]} />
      </I18nextProvider>,
    );

    expect(container.querySelectorAll('.bet-resolve-btn').length).toBe(0);
    expect(container.querySelector('.bet-resolve-done')).toBeTruthy();
  });
});

// Avoid an unused-default-language import warning when the helper isn't
// used by this file but the i18n module still re-exports it.
void DEFAULT_LANGUAGE;
