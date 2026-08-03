import type { Bet } from '../types';
import { resolveBet } from '../api/client';

export default function BetList({ bets, onUpdate }: { bets: Bet[]; onUpdate: () => void }) {
  const handleResolve = async (id: string, status: 'won' | 'lost') => {
    await resolveBet(id, status);
    onUpdate();
  };

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
            <th>Amount</th>
            <th>Odds</th>
            <th>Payout</th>
            <th>Status</th>
            <th>Created</th>
            <th>Actions</th>
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
              <td>{new Date(bet.created_at).toLocaleString()}</td>
              <td>
                {bet.status === 'pending' && (
                  <>
                    <button
                      className="btn-win"
                      onClick={() => handleResolve(bet.id, 'won')}
                    >
                      ✓ Won
                    </button>
                    <button
                      className="btn-lose"
                      onClick={() => handleResolve(bet.id, 'lost')}
                    >
                      ✗ Lost
                    </button>
                  </>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
