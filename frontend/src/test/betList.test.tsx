import { describe, it, expect, afterEach, beforeAll, afterAll, vi } from 'vitest';
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
    start_time: '2026-08-10T20:00:00Z',
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

describe('BetList — hooks ordering', () => {
  // Regression guard: hooks must run in the same order whether the initial
  // render hits the empty branch or the populated branch. Otherwise React
  // warns about "change in the order of Hooks" and the component may crash
  // when the parent transitions from empty → populated.
  let errorSpy: ReturnType<typeof vi.spyOn>;

  beforeAll(() => {
    errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterAll(() => {
    errorSpy.mockRestore();
  });

  it('does not warn about hook order on first render with bets', () => {
    const bet = makeBet();
    render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={[bet]} />
      </I18nextProvider>,
    );
    const hookWarnings = errorSpy.mock.calls.filter((args: unknown[]) =>
      args.some(
        (a: unknown) =>
          typeof a === 'string' &&
          a.includes('order of Hooks'),
      ),
    );
    expect(hookWarnings).toEqual([]);
  });

  it('does not warn when transitioning from empty → populated', () => {
    errorSpy.mockClear();
    const { rerender } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={[]} />
      </I18nextProvider>,
    );
    rerender(
      <I18nextProvider i18n={i18n}>
        <BetList bets={[makeBet({ id: 'b-late' })]} />
      </I18nextProvider>,
    );
    const hookWarnings = errorSpy.mock.calls.filter((args: unknown[]) =>
      args.some(
        (a: unknown) =>
          typeof a === 'string' &&
          a.includes('order of Hooks'),
      ),
    );
    expect(hookWarnings).toEqual([]);
  });
});

describe('BetList — match time column', () => {
  it('renders the match time cell with stacked date and time when start_time is set', () => {
    const bet = makeBet({ id: 'bet-mt-1' });
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={[bet]} />
      </I18nextProvider>,
    );

    const matchCell = container.querySelector('.match-cell');
    expect(matchCell).toBeTruthy();
    expect(matchCell!.querySelector('.match-time-date')).toBeTruthy();
    expect(matchCell!.querySelector('.match-time-time')).toBeTruthy();
    expect(container.querySelectorAll('.no-match-time').length).toBe(0);
  });

  it('renders a placeholder dash when start_time is null', () => {
    const bet = makeBet({ id: 'bet-mt-2', start_time: null });
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={[bet]} />
      </I18nextProvider>,
    );

    expect(container.querySelector('.no-match-time')).toBeTruthy();
    expect(container.querySelector('.match-time-date')).toBeNull();
  });
});

describe('BetList — filter by user', () => {
  it('shows all rows by default', () => {
    const bets = [
      makeBet({ id: 'b1', user_id: 'u1', user_name: 'Kaká' }),
      makeBet({ id: 'b2', user_id: 'u2', user_name: 'Ronaldinho' }),
      makeBet({ id: 'b3', user_id: 'u1', user_name: 'Kaká' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    const userFilter = container.querySelector(
      '.bet-list-controls select',
    ) as HTMLSelectElement;
    expect(userFilter).toBeTruthy();
    expect(userFilter.value).toBe('');

    const rows = container.querySelectorAll('tbody tr');
    expect(rows.length).toBe(3);
  });

  it('narrows rows to only the selected user', () => {
    const bets = [
      makeBet({ id: 'b1', user_id: 'u1', user_name: 'Kaká' }),
      makeBet({ id: 'b2', user_id: 'u2', user_name: 'Ronaldinho' }),
      makeBet({ id: 'b3', user_id: 'u1', user_name: 'Kaká' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    const userFilter = container.querySelector(
      '.bet-list-controls select',
    ) as HTMLSelectElement;
    fireEvent.change(userFilter, { target: { value: 'u2' } });

    const rows = container.querySelectorAll('tbody tr');
    expect(rows.length).toBe(1);
  });

  it('lists distinct users as options, sorted by name', () => {
    const bets = [
      makeBet({ id: 'b1', user_id: 'u1', user_name: 'Kaká' }),
      makeBet({ id: 'b2', user_id: 'u2', user_name: 'Ronaldinho' }),
      makeBet({ id: 'b3', user_id: 'u1', user_name: 'Kaká' }), // dup
      makeBet({ id: 'b4', user_id: 'u3', user_name: 'Adriano' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    const userFilter = container.querySelector(
      '.bet-list-controls select',
    ) as HTMLSelectElement;
    const optionValues = Array.from(userFilter.options).map((o) => o.value);
    // First option is the "All users" sentinel, then distinct users in alphabetical order.
    expect(optionValues).toEqual(['', 'u3', 'u1', 'u2']);
  });
});

describe('BetList — sort', () => {
  // Find the clickable header button for a given column by walking the
  // <th> children. Each sortable header renders a <button> with the column
  // label as its first text node.
  function clickHeader(container: HTMLElement, label: string) {
    const headers = Array.from(
      container.querySelectorAll<HTMLButtonElement>('.sort-header-btn'),
    );
    const btn = headers.find((b) =>
      b.textContent?.toLowerCase().includes(label.toLowerCase()),
    );
    if (!btn) throw new Error(`No sortable header matching "${label}"`);
    fireEvent.click(btn);
  }

  function getActiveArrow(container: HTMLElement): string {
    const active = container.querySelector('th.sortable.active .sort-arrow');
    return (active?.textContent || '').trim();
  }

  it('defaults to bettedAt desc (no user sort)', () => {
    // The default-sort short-circuit trusts that input is already in
    // `created_at DESC` order (which is what the backend returns), so the
    // test data must mirror that.
    const bets = [
      makeBet({ id: 'b2', odds: 5.0, created_at: '2026-08-03T10:00:00Z' }),
      makeBet({ id: 'b3', odds: 3.0, created_at: '2026-08-02T10:00:00Z' }),
      makeBet({ id: 'b1', odds: 1.2, created_at: '2026-08-01T10:00:00Z' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    // No header is active — the Betted at column shows the inactive arrow.
    expect(container.querySelectorAll('th.sortable.active').length).toBe(0);
    // Newest bet first (08-03).
    const dates = Array.from(
      container.querySelectorAll('.betted-at-date'),
    ).map((c) => c.textContent?.trim());
    expect(dates[0]).toMatch(/03\/08|08\/03/);
  });

  it('sorts by odds descending on first click, ascending on second, clears on third', () => {
    const bets = [
      makeBet({ id: 'b1', odds: 1.2 }),
      makeBet({ id: 'b2', odds: 5.0 }),
      makeBet({ id: 'b3', odds: 3.0 }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    clickHeader(container, 'Odds');
    expect(
      Array.from(container.querySelectorAll('.odds-cell')).map((c) =>
        c.textContent?.trim(),
      ),
    ).toEqual(['5x', '3x', '1.2x']);
    expect(getActiveArrow(container)).toBe('\u25BC');

    clickHeader(container, 'Odds');
    expect(
      Array.from(container.querySelectorAll('.odds-cell')).map((c) =>
        c.textContent?.trim(),
      ),
    ).toEqual(['1.2x', '3x', '5x']);
    expect(getActiveArrow(container)).toBe('\u25B2');

    clickHeader(container, 'Odds');
    // Cleared → no active header → odds default order (insertion).
    expect(container.querySelectorAll('th.sortable.active').length).toBe(0);
  });

  it('clicking a different sortable header switches the active column', () => {
    const bets = [
      makeBet({ id: 'b1', odds: 1.2 }),
      makeBet({ id: 'b2', odds: 5.0 }),
      makeBet({ id: 'b3', odds: 3.0 }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    clickHeader(container, 'Odds');
    expect(container.querySelector('th.sortable.active')?.textContent).toMatch(
      /Odds/,
    );

    clickHeader(container, 'Status');
    const active = container.querySelector('th.sortable.active');
    expect(active?.textContent).toMatch(/Status/);
    expect(active?.textContent).not.toMatch(/Odds/);
  });

  it('sorts by status', () => {
    const bets = [
      makeBet({ id: 'b1', status: 'won' }),
      makeBet({ id: 'b2', status: 'pending' }),
      makeBet({ id: 'b3', status: 'lost' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    clickHeader(container, 'Status');
    const badges = Array.from(
      container.querySelectorAll('.status-badge'),
    ).map((c) => c.textContent?.trim());
    // First click sets desc — alphabetical desc: won, pending, lost.
    expect(badges).toEqual(['Won', 'Pending', 'Lost']);

    clickHeader(container, 'Status');
    const badges2 = Array.from(
      container.querySelectorAll('.status-badge'),
    ).map((c) => c.textContent?.trim());
    // Second click sets asc — alphabetical asc: lost, pending, won.
    expect(badges2).toEqual(['Lost', 'Pending', 'Won']);
  });

  it('sorts by matchTime and puts nulls last regardless of direction', () => {
    const bets = [
      makeBet({ id: 'b1', start_time: '2026-08-15T20:00:00Z' }),
      makeBet({ id: 'b2', start_time: null }),
      makeBet({ id: 'b3', start_time: '2026-08-10T20:00:00Z' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    clickHeader(container, 'Match');
    let rows = Array.from(container.querySelectorAll('tbody tr'));
    expect(
      rows[rows.length - 1].querySelector('.no-match-time') !== null,
    ).toBe(true);

    clickHeader(container, 'Match');
    rows = Array.from(container.querySelectorAll('tbody tr'));
    expect(
      rows[rows.length - 1].querySelector('.no-match-time') !== null,
    ).toBe(true);
  });

  it('filter + sort compose (filter then sort)', () => {
    const bets = [
      makeBet({ id: 'b1', user_id: 'u1', user_name: 'Kaká', odds: 1.2 }),
      makeBet({ id: 'b2', user_id: 'u2', user_name: 'Ronaldinho', odds: 5.0 }),
      makeBet({ id: 'b3', user_id: 'u1', user_name: 'Kaká', odds: 3.0 }),
      makeBet({ id: 'b4', user_id: 'u2', user_name: 'Ronaldinho', odds: 2.0 }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    const userFilter = container.querySelector(
      '.bet-list-controls select',
    ) as HTMLSelectElement;
    fireEvent.change(userFilter, { target: { value: 'u1' } });

    clickHeader(container, 'Odds');

    const oddsCells = Array.from(
      container.querySelectorAll('.odds-cell'),
    ).map((c) => c.textContent?.trim());
    // u1's bets: b3 (3.0), b1 (1.2). Descending: 3x, 1.2x.
    expect(oddsCells).toEqual(['3x', '1.2x']);
  });
});

// Avoid an unused-default-language import warning when the helper isn't
// used by this file but the i18n module still re-exports it.
void DEFAULT_LANGUAGE;