import { useCallback, useEffect, useState } from 'react';
import { fetchLeaderboard } from '../api/client';
import type { LeaderboardEntry } from '../types';

export default function Leaderboard({ groupId, refreshKey }: { groupId: string; refreshKey: number }) {
  const [entries, setEntries] = useState<LeaderboardEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = (await fetchLeaderboard(groupId)) as LeaderboardEntry[];
      setEntries(data);
    } catch {
      console.error('Failed to load leaderboard');
    } finally {
      setLoading(false);
    }
  }, [groupId]);

  useEffect(() => {
    load();
  }, [load, refreshKey]);

  if (loading) return <p className="leaderboard-loading">Loading leaderboard...</p>;
  if (entries.length === 0) return null;

  const podium = ['🥇', '🥈', '🥉'];

  return (
    <div className="leaderboard">
      <h2>Leaderboard</h2>
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>Player</th>
            <th>Balance</th>
            <th>At risk</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry, i) => (
            <tr key={entry.user_id} className={i < 3 ? 'podium-row' : ''}>
              <td className="rank-cell">
                {i < 3 ? podium[i] : i + 1}
              </td>
              <td>
                {entry.avatar_url && (
                  <img src={entry.avatar_url} alt="" className="lb-avatar" />
                )}
                <span className="lb-name">{entry.name}</span>
              </td>
              <td className="balance-cell">
                <strong>{entry.balance.toFixed(0)}</strong> pts
              </td>
              <td className="betted-cell">
                {entry.betted > 0 ? `-${entry.betted.toFixed(0)} pts` : '—'}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
