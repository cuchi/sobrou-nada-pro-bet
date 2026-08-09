import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { formatPoints, useActiveLocale } from './Points';
import EmptyState from './EmptyState';
import { ErrorBoundary } from './ErrorBoundary';
import { devResolveBet } from '../api/client';
import type { Bet, DevResolveOutcome } from '../types';

const PAGE_SIZE = 10;

type SortKey =
  | 'oddsDesc'
  | 'oddsAsc'
  | 'statusAsc'
  | 'statusDesc'
  | 'bettedAtDesc'
  | 'bettedAtAsc'
  | 'matchTimeDesc'
  | 'matchTimeAsc';

export default function BetList({
  bets,
  onBetResolved,
}: {
  bets: Bet[];
  onBetResolved?: () => void;
}) {
  return (
    <ErrorBoundary scope="BetList">
      <BetListInner bets={bets} onBetResolved={onBetResolved} />
    </ErrorBoundary>
  );
}

function BetListInner({
  bets,
  onBetResolved,
}: {
  bets: Bet[];
  onBetResolved?: () => void;
}) {
  const { t } = useTranslation();
  const locale = useActiveLocale();
  const [page, setPage] = useState(0);
  const [resolving, setResolving] = useState<string | null>(null);
  const [resolveError, setResolveError] = useState<string | null>(null);
  const [userFilter, setUserFilter] = useState<string | null>(null);
  const [sortKey, setSortKey] = useState<SortKey>('bettedAtDesc');

  const users = useMemo(() => {
    const seen = new Map<string, string>();
    for (const b of bets) {
      if (!seen.has(b.user_id)) {
        seen.set(b.user_id, b.user_name || b.user_email);
      }
    }
    return Array.from(seen.entries())
      .map(([id, name]) => ({ id, name }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [bets]);

  const filtered =
    userFilter === null ? bets : bets.filter((b) => b.user_id === userFilter);

  const sorted = useMemo(() => sortBets(filtered, sortKey), [filtered, sortKey]);

  if (bets.length === 0) {
    return (
      <div className="bet-list-empty">
        <EmptyState
          icon="ticket"
          title={t('betList.empty')}
          hint={t('betList.emptyHint')}
        />
      </div>
    );
  }

  const pageCount = Math.max(1, Math.ceil(sorted.length / PAGE_SIZE));
  const current = Math.min(page, pageCount - 1);
  const rows = sorted.slice(current * PAGE_SIZE, (current + 1) * PAGE_SIZE);

  function predictionText(bet: Bet): string {
    switch (bet.prediction) {
      case 'home_win':
        return bet.home_team || t('betList.prediction.homeWinFallback');
      case 'away_win':
        return bet.away_team || t('betList.prediction.awayWinFallback');
      case 'draw':
        return t('betList.prediction.draw');
      default:
        return t('betList.prediction.noPrediction');
    }
  }

  function statusLabel(status: Bet['status']): string {
    return t(`betList.statuses.${status}`);
  }

  function bettedAt(iso: string): { date: string; time: string } {
    return formatLocalDateTime(iso, locale);
  }

  function matchTime(iso: string | null): { date: string; time: string } | null {
    if (iso === null) return null;
    return formatLocalDateTime(iso, locale);
  }

  async function resolveBet(bet: Bet, outcome: DevResolveOutcome) {
    if (resolving) return;
    setResolving(bet.id);
    setResolveError(null);
    try {
      await devResolveBet({ bet_id: bet.id, outcome });
      onBetResolved?.();
    } catch (e) {
      setResolveError(e instanceof Error ? e.message : String(e));
    } finally {
      setResolving(null);
    }
  }

  return (
    <div className="bet-list">
      <h2>{t('betList.heading', { count: bets.length })}</h2>

      <div className="bet-list-controls">
        <label>
          <span>{t('betList.filter.user.label')}</span>
          <select
            className="group-select"
            value={userFilter ?? ''}
            onChange={(e) => {
              setPage(0);
              setUserFilter(e.target.value === '' ? null : e.target.value);
            }}
          >
            <option value="">{t('betList.filter.user.all')}</option>
            {users.map((u) => (
              <option key={u.id} value={u.id}>
                {u.name}
              </option>
            ))}
          </select>
        </label>

        <label>
          <span>{t('betList.sort.label')}</span>
          <select
            className="group-select"
            value={sortKey}
            onChange={(e) => {
              setPage(0);
              setSortKey(e.target.value as SortKey);
            }}
          >
            {(
              [
                'oddsDesc',
                'oddsAsc',
                'statusAsc',
                'statusDesc',
                'bettedAtDesc',
                'bettedAtAsc',
                'matchTimeDesc',
                'matchTimeAsc',
              ] as SortKey[]
            ).map((k) => (
              <option key={k} value={k}>
                {t(`betList.sort.options.${k}`)}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="bet-list-table-wrap">
      <table>
        <thead>
          <tr>
            <th>{t('betList.columns.user')}</th>
            <th>{t('betList.columns.event')}</th>
            <th>{t('betList.columns.match')}</th>
            <th>{t('betList.columns.pick')}</th>
            <th>{t('betList.columns.amount')}</th>
            <th>{t('betList.columns.odds')}</th>
            <th>{t('betList.columns.status')}</th>
            <th>{t('betList.columns.bettedAt')}</th>
            {import.meta.env.DEV && (
                          <th className="bet-resolve-header">{t('betList.columns.resolve')}</th>
                        )}
          </tr>
        </thead>
        <tbody>
          {rows.map((bet) => {
            const mt = matchTime(bet.start_time);
            return (
              <tr key={bet.id} className={`status-${bet.status}`}>
                <td className="user-cell">
                  {bet.user_avatar_url ? (
                    <img
                      src={bet.user_avatar_url}
                      alt=""
                      className="bet-avatar"
                      title={bet.user_name || bet.user_email}
                    />
                  ) : (
                    <span
                      className="bet-avatar bet-avatar-initial"
                      title={bet.user_name || bet.user_email}
                    >
                      {(bet.user_name || bet.user_email).slice(0, 2).toUpperCase()}
                    </span>
                  )}
                </td>
                <td className="event-cell">
                  {bet.home_team && bet.away_team ? (
                    <span className="event-teams-small">
                      {bet.home_team} vs {bet.away_team}
                    </span>
                  ) : (
                    <span className="no-event">{t('betList.noEvent')}</span>
                  )}
                </td>
                <td className="match-cell">
                  {mt ? (
                    <>
                      <span className="match-time-date">{mt.date}</span>
                      <span className="match-time-time">{mt.time}</span>
                    </>
                  ) : (
                    <span className="no-match-time">{t('betList.noMatchTime')}</span>
                  )}
                </td>
                <td>
                  {bet.prediction ? (
                    <span className="prediction-tag">{predictionText(bet)}</span>
                  ) : (
                    t('betList.prediction.noPrediction')
                  )}
                </td>
                <td className="amount-cell">{formatPoints(bet.amount, locale)}</td>
                <td className="odds-cell" title={t('betList.payoutTooltip', { payout: (bet.amount * bet.odds).toFixed(0) })}>
                  {bet.odds}x
                </td>
                <td>
                  <span className="status-badge">{statusLabel(bet.status)}</span>
                </td>
                <td className="betted-at-cell">
                  <span className="betted-at-date">{bettedAt(bet.created_at).date}</span>
                  <span className="betted-at-time">{bettedAt(bet.created_at).time}</span>
                </td>
                {import.meta.env.DEV && (
                  <td className="bet-resolve-cell">
                    {bet.status === 'pending' ? (
                      <div className="bet-resolve-buttons">
                        <button
                          type="button"
                          className="bet-resolve-btn"
                          title={bet.home_team || t('betList.resolveOutcome.home')}
                          disabled={resolving === bet.id}
                          onClick={() => resolveBet(bet, 'home_win')}
                        >
                          {bet.home_team || t('betList.resolveOutcome.home')}
                        </button>
                        <button
                          type="button"
                          className="bet-resolve-btn"
                          title={t('betList.resolveOutcome.draw')}
                          disabled={resolving === bet.id}
                          onClick={() => resolveBet(bet, 'draw')}
                        >
                          X
                        </button>
                        <button
                          type="button"
                          className="bet-resolve-btn"
                          title={bet.away_team || t('betList.resolveOutcome.away')}
                          disabled={resolving === bet.id}
                          onClick={() => resolveBet(bet, 'away_win')}
                        >
                          {bet.away_team || t('betList.resolveOutcome.away')}
                        </button>
                      </div>
                    ) : (
                      <span className="bet-resolve-done">—</span>
                    )}
                  </td>
                )}
              </tr>
            );
          })}
        </tbody>
      </table>
      </div>

      {resolveError && (
        <div className="bet-resolve-error" role="alert">
          {resolveError}
          <button
            type="button"
            className="banner-dismiss"
            onClick={() => setResolveError(null)}
          >
            ×
          </button>
        </div>
      )}

      {pageCount > 1 && (
        <div className="pagination">
          <button
            className="btn-page"
            onClick={() => setPage(current - 1)}
            disabled={current === 0}
          >
            {t('betList.pagination.prev')}
          </button>
          <span className="page-indicator">
            {t('betList.pagination.page', { current: current + 1, total: pageCount })}
          </span>
          <button
            className="btn-page"
            onClick={() => setPage(current + 1)}
            disabled={current >= pageCount - 1}
          >
            {t('betList.pagination.next')}
          </button>
        </div>
      )}
    </div>
  );
}

function formatLocalDateTime(
  iso: string,
  locale: string,
): { date: string; time: string } {
  const d = new Date(iso);
  const dateFmt = new Intl.DateTimeFormat(locale, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  });
  const timeFmt = new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit',
  });
  return { date: dateFmt.format(d), time: timeFmt.format(d) };
}

function sortBets(bets: Bet[], key: SortKey): Bet[] {
  const out = [...bets];
  // Tie-break: newest betted first, then id ascending — keeps order deterministic.
  out.sort((a, b) => {
    const primary = compareByKey(a, b, key);
    if (primary !== 0) return primary;
    const created = b.created_at.localeCompare(a.created_at);
    if (created !== 0) return created;
    return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });
  return out;
}

function compareByKey(a: Bet, b: Bet, key: SortKey): number {
  switch (key) {
    case 'oddsDesc':
      return b.odds - a.odds;
    case 'oddsAsc':
      return a.odds - b.odds;
    case 'statusAsc':
      return a.status.localeCompare(b.status);
    case 'statusDesc':
      return b.status.localeCompare(a.status);
    case 'bettedAtDesc':
      return b.created_at.localeCompare(a.created_at);
    case 'bettedAtAsc':
      return a.created_at.localeCompare(b.created_at);
    case 'matchTimeDesc':
      return compareNullableDesc(a.start_time, b.start_time);
    case 'matchTimeAsc':
      return compareNullableAsc(a.start_time, b.start_time);
  }
}

function compareNullableAsc(a: string | null, b: string | null): number {
  if (a === null && b === null) return 0;
  if (a === null) return 1; // nulls last
  if (b === null) return -1;
  return a.localeCompare(b);
}

function compareNullableDesc(a: string | null, b: string | null): number {
  if (a === null && b === null) return 0;
  if (a === null) return 1; // nulls last
  if (b === null) return -1;
  return b.localeCompare(a);
}