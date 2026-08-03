import { useCallback, useEffect, useState } from 'react';
import { fetchEvents } from '../api/client';
import type { Event, Prediction } from '../types';

interface Props {
  onSelect: (event: Event, prediction: Prediction, odds: number) => void;
}

function oddsLabel(ev: Event, pred: Prediction): string {
  const o = pred === 'home_win' ? ev.home_odds : pred === 'draw' ? ev.draw_odds : ev.away_odds;
  return o != null ? ` (${o}x)` : '';
}

export default function EventPicker({ onSelect }: Props) {
  const [events, setEvents] = useState<Event[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [prediction, setPrediction] = useState<Prediction | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = (await fetchEvents('scheduled,live')) as Event[];
      setEvents(data);
      if (data.length === 0) {
        setSelectedId(null);
        setPrediction(null);
      }
    } catch {
      console.error('Failed to load events');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleSync = async () => {
    setSyncing(true);
    try {
      const resp = await fetch('/api/events/sync', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${localStorage.getItem('token')}`,
        },
      });
      if (!resp.ok) {
        const err = await resp.json().catch(() => ({}));
        throw new Error((err as { error?: string }).error || 'Sync failed');
      }
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to sync. Check FOOTBALLDATA_API_KEY.');
    } finally {
      setSyncing(false);
    }
  };

  const selected = events.find((e) => e.id === selectedId);

  const handleEventSelect = (id: string) => {
    setSelectedId(id);
    setPrediction(null);
  };

  const handlePredictionSelect = (p: Prediction) => {
    setPrediction(p);
    const ev = events.find((e) => e.id === selectedId);
    if (ev) {
      const odds = p === 'home_win' ? ev.home_odds : p === 'draw' ? ev.draw_odds : ev.away_odds;
      onSelect(ev, p, odds ?? 1.0);
    }
  };

  if (loading) return <p className="event-picker-loading">Loading events...</p>;

  return (
    <div className="event-picker">
      <div className="event-picker-header">
        <h3>Upcoming matches</h3>
        <button onClick={handleSync} disabled={syncing} className="btn-sync">
          {syncing ? 'Syncing...' : 'Sync fixtures'}
        </button>
      </div>

      {events.length === 0 ? (
        <p className="no-events">
          No upcoming matches. Click <strong>Sync fixtures</strong> to pull them from footballdata.io.
        </p>
      ) : (
        <div className="events-grid">
          {events.map((ev) => (
            <button
              key={ev.id}
              className={`event-card ${selectedId === ev.id ? 'selected' : ''} ${ev.status}`}
              onClick={() => handleEventSelect(ev.id)}
            >
              <span className="event-teams">
                {ev.home_team} vs {ev.away_team}
              </span>
              <span className="event-meta">
                {ev.championship} · {new Date(ev.start_time).toLocaleDateString()}
              </span>
              <span className={`event-status event-status-${ev.status}`}>
                {ev.status === 'live' ? '🔴 LIVE' : ev.status === 'finished' ? '✓ Done' : '⏳ Upcoming'}
              </span>
            </button>
          ))}
        </div>
      )}

      {selected && (
        <div className="prediction-bar">
          <span className="prediction-label">Your pick:</span>
          <button
            className={`btn-prediction ${prediction === 'home_win' ? 'active' : ''}`}
            onClick={() => handlePredictionSelect('home_win')}
          >
            {selected.home_team}{oddsLabel(selected, 'home_win')}
          </button>
          <button
            className={`btn-prediction ${prediction === 'draw' ? 'active' : ''}`}
            onClick={() => handlePredictionSelect('draw')}
          >
            Draw{oddsLabel(selected, 'draw')}
          </button>
          <button
            className={`btn-prediction ${prediction === 'away_win' ? 'active' : ''}`}
            onClick={() => handlePredictionSelect('away_win')}
          >
            {selected.away_team}{oddsLabel(selected, 'away_win')}
          </button>
        </div>
      )}
    </div>
  );
}
