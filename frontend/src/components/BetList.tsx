import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { formatPoints, useActiveLocale } from './Points';
import type { Bet } from '../types';

const PAGE_SIZE = 10;

export default function BetList({ bets }: { bets: Bet[] }) {
  const { t } = useTranslation();
  const locale = useActiveLocale();
  const [page, setPage] = useState(0);

  if (bets.length === 0) {
    return <p className="empty">{t('betList.empty')}</p>;
  }

  const pageCount = Math.max(1, Math.ceil(bets.length / PAGE_SIZE));
  const current = Math.min(page, pageCount - 1);
  const rows = bets.slice(current * PAGE_SIZE, (current + 1) * PAGE_SIZE);

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

  return (
    <div className="bet-list">
      <h2>{t('betList.heading', { count: bets.length })}</h2>
      <table>
        <thead>
          <tr>
            <th>{t('betList.columns.user')}</th>
            <th>{t('betList.columns.event')}</th>
            <th>{t('betList.columns.pick')}</th>
            <th>{t('betList.columns.amount')}</th>
            <th>{t('betList.columns.odds')}</th>
            <th>{t('betList.columns.status')}</th>
            <th>{t('betList.columns.bettedAt')}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((bet) => (
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
            </tr>
          ))}
        </tbody>
      </table>

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
