import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { fetchLeaderboard } from '../api/client';
import { formatPoints, useActiveLocale } from './Points';
import type { LeaderboardEntry } from '../types';
import { Spinner } from './Spinner';
import { usePolling } from '../usePolling';

export default function Leaderboard({ groupId, refreshKey }: { groupId: string; refreshKey: number }) {
  const { t } = useTranslation();
  const locale = useActiveLocale();
  const [entries, loading] = usePolling<LeaderboardEntry[]>(
    useCallback(
      () => fetchLeaderboard(groupId) as Promise<LeaderboardEntry[]>,
      [groupId, refreshKey],
    ),
    60_000,
  );

  if (loading) return (
    <div className="leaderboard">
      <h2>{t('leaderboard.heading')}</h2>
      <Spinner label={t('leaderboard.loading')} />
    </div>
  );
  if (!entries || entries.length === 0) return null;

  const podium = ['🥇', '🥈', '🥉'];

  return (
    <div className="leaderboard">
      <h2>{t('leaderboard.heading')}</h2>
      <table>
        <thead>
          <tr>
            <th>{t('leaderboard.columns.rank')}</th>
            <th>{t('leaderboard.columns.player')}</th>
            <th>{t('leaderboard.columns.balance')}</th>
            <th>{t('leaderboard.columns.atRisk')}</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry, i) => (
            <tr key={entry.user_id} className={i < 3 ? 'podium-row' : ''}>
              <td className="rank-cell">
                {i < 3 ? podium[i] : i + 1}
              </td>
              <td className="lb-name-cell">
                {entry.avatar_url && (
                  <img src={entry.avatar_url} alt="" className="lb-avatar" />
                )}
                <span className="lb-name" title={entry.name}>{entry.name}</span>
              </td>
              <td className="balance-cell">
                {formatPoints(entry.balance, locale)}
              </td>
              <td className="betted-cell">
                {entry.betted > 0
                  ? t('units.pointsAtRisk', { amount: entry.betted.toFixed(0) })
                  : t('leaderboard.atRiskEmpty')}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
