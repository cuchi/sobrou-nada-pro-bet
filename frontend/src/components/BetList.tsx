import type { Bet } from '../types';

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

export default function BetList({ bets }: { bets: Bet[] }) {
  if (bets.length === 0) {
    return <p className="empty">No bets yet. Place your first one!</p>;
  }

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
          </tr>
        </thead>
        <tbody>
          {bets.map((bet) => (
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
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
