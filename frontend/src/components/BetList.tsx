import { useState } from 'react';
import type { Bet } from '../types';

const PAGE_SIZE = 10;

function predictionText(bet: Bet): string {
  switch (bet.prediction) {
    case 'home_win':
      return bet.home_team || 'Home win';
    case 'away_win':
      return bet.away_team || 'Away win';
    case 'draw':
      return 'Draw';
    default:
      return '—';
  }
}

function bettedAt(iso: string): { date: string; time: string } {
  const d = new Date(iso);
  const dd = String(d.getDate()).padStart(2, '0');
  const mm = String(d.getMonth() + 1).padStart(2, '0');
  const hh = String(d.getHours()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  return { date: `${dd}/${mm}/${d.getFullYear()}`, time: `${hh}:${ss}` };
}

export default function BetList({ bets }: { bets: Bet[] }) {
  const [page, setPage] = useState(0);

  if (bets.length === 0) {
    return <p className="empty">No bets yet. Place your first one!</p>;
  }

  const pageCount = Math.max(1, Math.ceil(bets.length / PAGE_SIZE));
  const current = Math.min(page, pageCount - 1);
  const rows = bets.slice(current * PAGE_SIZE, (current + 1) * PAGE_SIZE);

  return (
    <div className="bet-list">
      <h2>All Bets ({bets.length})</h2>
      <table>
        <thead>
          <tr>
            <th>User</th>
            <th>Event</th>
            <th>Pick</th>
            <th>Amount</th>
            <th>Odds</th>
            <th>Status</th>
            <th>Betted at</th>
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
                  <span className="no-event">—</span>
                )}
              </td>
              <td>
                {bet.prediction ? (
                  <span className="prediction-tag">{predictionText(bet)}</span>
                ) : (
                  '—'
                )}
              </td>
              <td>{bet.amount.toFixed(0)} pts</td>
              <td title={`Payout: ${(bet.amount * bet.odds).toFixed(0)} pts`}>{bet.odds}x</td>
              <td>
                <span className="status-badge">{bet.status}</span>
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
            ‹ Prev
          </button>
          <span className="page-indicator">
            {current + 1} / {pageCount}
          </span>
          <button
            className="btn-page"
            onClick={() => setPage(current + 1)}
            disabled={current >= pageCount - 1}
          >
            Next ›
          </button>
        </div>
      )}
    </div>
  );
}
