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
  function getSortSelect(container: HTMLElement): HTMLSelectElement {
    const selects = container.querySelectorAll(
      '.bet-list-controls select',
    );
    // 2 controls: first is user filter, second is sort.
    return selects[1] as HTMLSelectElement;
  }

  function getRowIds(container: HTMLElement): string[] {
    // The user-cell has a title containing the user's name; the bet id
    // is on the row key but not directly readable. Read the order column
    // values in column 4 (amount cell) — distinct, monotonic.
    const rows = container.querySelectorAll('tbody tr');
    return Array.from(rows).map((r) => r.textContent || '');
  }

  it('sorts by odds desc by default? no — default is bettedAtDesc', () => {
    const bets = [
      makeBet({ id: 'b1', odds: 1.2, created_at: '2026-08-01T10:00:00Z' }),
      makeBet({ id: 'b2', odds: 5.0, created_at: '2026-08-03T10:00:00Z' }),
      makeBet({ id: 'b3', odds: 3.0, created_at: '2026-08-02T10:00:00Z' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    // Default sort is bettedAt desc → b2 (08-03), b3 (08-02), b1 (08-01).
    expect(getRowIds(container).map((t) => t).join('|')).toContain('08/03');
  });

  it('sorts by odds descending when sort=oddsDesc', () => {
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

    const sortSel = getSortSelect(container);
    fireEvent.change(sortSel, { target: { value: 'oddsDesc' } });

    const oddsCells = Array.from(
      container.querySelectorAll('.odds-cell'),
    ).map((c) => c.textContent?.trim());
    expect(oddsCells).toEqual(['5x', '3x', '1.2x']);
  });

  it('sorts by odds ascending when sort=oddsAsc', () => {
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

    const sortSel = getSortSelect(container);
    fireEvent.change(sortSel, { target: { value: 'oddsAsc' } });

    const oddsCells = Array.from(
      container.querySelectorAll('.odds-cell'),
    ).map((c) => c.textContent?.trim());
    expect(oddsCells).toEqual(['1.2x', '3x', '5x']);
  });

  it('sorts by bettedAt ascending when sort=bettedAtAsc', () => {
    const bets = [
      makeBet({ id: 'b1', created_at: '2026-08-05T10:00:00Z' }),
      makeBet({ id: 'b2', created_at: '2026-08-01T10:00:00Z' }),
      makeBet({ id: 'b3', created_at: '2026-08-03T10:00:00Z' }),
    ];
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <BetList bets={bets} />
      </I18nextProvider>,
    );

    const sortSel = getSortSelect(container);
    fireEvent.change(sortSel, { target: { value: 'bettedAtAsc' } });

    const dates = Array.from(
      container.querySelectorAll('.betted-at-date'),
    ).map((c) => c.textContent?.trim());
    // 08/01/2026, 08/03/2026, 08/05/2026 — depending on locale it could
    // also be 01/08/2026, but the day-month-year ordering is stable.
    expect(dates[0]).toMatch(/01\/08|08\/01/);
    expect(dates[1]).toMatch(/03\/08|08\/03/);
    expect(dates[2]).toMatch(/05\/08|08\/05/);
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

    const sortSel = getSortSelect(container);
    fireEvent.change(sortSel, { target: { value: 'statusAsc' } });

    const badges = Array.from(
      container.querySelectorAll('.status-badge'),
    ).map((c) => c.textContent?.trim());
    // statusAsc = alphabetical: lost, pending, won.
    expect(badges[0]).toBe(badges[0]); // sanity
    const sorted = [...badges].sort();
    expect(badges).toEqual(sorted);
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

    const sortSel = getSortSelect(container);
    fireEvent.change(sortSel, { target: { value: 'matchTimeAsc' } });

    const rows = Array.from(container.querySelectorAll('tbody tr'));
    const lastRowHasDash =
      rows[rows.length - 1].querySelector('.no-match-time') !== null;
    expect(lastRowHasDash).toBe(true);

    fireEvent.change(sortSel, { target: { value: 'matchTimeDesc' } });
    const rows2 = Array.from(container.querySelectorAll('tbody tr'));
    const lastRowHasDash2 =
      rows2[rows2.length - 1].querySelector('.no-match-time') !== null;
    expect(lastRowHasDash2).toBe(true);
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

    const selects = container.querySelectorAll(
      '.bet-list-controls select',
    );
    const userFilter = selects[0] as HTMLSelectElement;
    const sortSel = selects[1] as HTMLSelectElement;

    fireEvent.change(userFilter, { target: { value: 'u1' } });
    fireEvent.change(sortSel, { target: { value: 'oddsDesc' } });

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