import type { Bet } from '../types';

const predLabel: Record<string, string> = {
  home_win: 'Home win',
  away_win: 'Away win',
  draw: 'Draw',
};

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
            <th>Payout</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {bets.map((bet) => (
            <tr key={bet.id} className={`status-${bet.status}`}>
              <td className="user-cell">
                <span className="user-name-cell">
                  {bet.user_name || bet.user_email}
                </span>
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
                  <span className="prediction-tag">{predLabel[bet.prediction] ?? bet.prediction}</span>
                ) : (
                  '—'
                )}
              </td>
              <td>{bet.amount.toFixed(0)} pts</td>
              <td>{bet.odds}x</td>
              <td className="payout-cell">
                {bet.status === 'won'
                  ? `+${(bet.amount * bet.odds).toFixed(0)}`
                  : bet.status === 'lost'
                    ? `-${bet.amount.toFixed(0)}`
                    : `${(bet.amount * bet.odds).toFixed(0)} pts`}
              </td>
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
