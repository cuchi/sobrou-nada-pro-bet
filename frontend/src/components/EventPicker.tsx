import { useCallback, useEffect, useState } from 'react';
import { fetchEvents } from '../api/client';
import { getCrestUrl } from '../crests';
import type { Event, Prediction } from '../types';
import { Spinner } from './Spinner';
import { usePolling } from '../usePolling';

interface Props {
  onSelect: (event: Event, prediction: Prediction, odds: number) => void;
  onEventChange?: () => void;
  bettedEventIds: Set<string>;
  resetKey?: number;
}

function normalize(s: string): string {
  return s.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '');
}

function oddsLabel(ev: Event, pred: Prediction): string {
  const o = pred === 'home_win' ? ev.home_odds : pred === 'draw' ? ev.draw_odds : ev.away_odds;
  return o != null ? `${o}x` : '';
}

function TeamCrest({ name }: { name: string }) {
  return <img src={getCrestUrl(name)!} alt="" className="team-crest" />;
}

export default function EventPicker({ onSelect, onEventChange, bettedEventIds, resetKey }: Props) {
  const [events, loading] = usePolling<Event[]>(
    useCallback(() => fetchEvents('scheduled,live') as Promise<Event[]>, []),
    60_000,
  );
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [prediction, setPrediction] = useState<Prediction | null>(null);
  const [query, setQuery] = useState('');

  useEffect(() => {
    setSelectedId(null);
    setPrediction(null);
  }, [resetKey]);

  // Reset selection when events become empty
  useEffect(() => {
    if (events && events.length === 0) {
      setSelectedId(null);
      setPrediction(null);
    }
  }, [events]);

  const selected = events?.find((e) => e.id === selectedId);

  const handleEventSelect = (id: string) => {
    setSelectedId(id);
    setPrediction(null);
    onEventChange?.();
  };

  const handlePredictionSelect = (p: Prediction) => {
    setPrediction(p);
    const ev = events?.find((e) => e.id === selectedId);
    if (ev) {
      const odds = p === 'home_win' ? ev.home_odds : p === 'draw' ? ev.draw_odds : ev.away_odds;
      onSelect(ev, p, odds ?? 1.0);
    }
  };

  if (loading) return (
    <div className="event-picker">
      <h3 className="event-picker-heading">Upcoming matches</h3>
      <Spinner label="Loading matches..." />
    </div>
  );

  const all = events ?? [];
  const q = normalize(query.trim());
  const list = q
    ? all.filter((ev) => normalize(ev.home_team).includes(q) || normalize(ev.away_team).includes(q))
    : all;

  return (
    <div className="event-picker">
      <h3 className="event-picker-heading">Upcoming matches</h3>

      {all.length > 0 && (
        <input
          type="search"
          className="event-search"
          placeholder="Search team..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      )}

      {all.length === 0 ? (
        <p className="no-events">
          No upcoming matches right now. Check back later.
        </p>
      ) : list.length === 0 ? (
        <p className="no-events">No matches found for “{query}”.</p>
      ) : (
        <div className="events-grid">
          {list.map((ev) => {
            const betted = bettedEventIds.has(ev.id);
            return (
            <button
              key={ev.id}
              className={`event-card ${selectedId === ev.id ? 'selected' : ''} ${betted ? 'betted' : ''}`}
              onClick={() => !betted && handleEventSelect(ev.id)}
              disabled={betted}
            >
              <span className="team-name home">{ev.home_team}</span>
              <span className="matchup-core">
                <TeamCrest name={ev.home_team} />
                <span className="vs-label">vs</span>
                <TeamCrest name={ev.away_team} />
              </span>
              <span className="team-name away">{ev.away_team}</span>
              <div className="event-odds">
                <span className="odd-badge">{ev.home_odds != null ? ev.home_odds : '-'}</span>
                <span className="odd-badge">{ev.draw_odds != null ? ev.draw_odds : '-'}</span>
                <span className="odd-badge">{ev.away_odds != null ? ev.away_odds : '-'}</span>
              </div>
              <div className="event-info">
                <span className="event-date">
                  {new Intl.DateTimeFormat('pt-BR', {
                    day: '2-digit', month: '2-digit',
                  }).format(new Date(ev.start_time))}
                </span>
                <span className="event-time">
                  {new Intl.DateTimeFormat('pt-BR', {
                    hour: '2-digit', minute: '2-digit',
                  }).format(new Date(ev.start_time))}
                </span>
              </div>
            </button>
            );
          })}
        </div>
      )}

      {selected && (
        <div className="prediction-bar">
          <span className="prediction-label">Your pick:</span>
          <button
            type="button"
            className={`btn-prediction ${prediction === 'home_win' ? 'active' : ''}`}
            onClick={() => handlePredictionSelect('home_win')}
          >
            <span className="prediction-team">{selected.home_team}</span>
            {oddsLabel(selected, 'home_win') && (
              <span className="prediction-odds">{oddsLabel(selected, 'home_win')}</span>
            )}
          </button>
          <button
            type="button"
            className={`btn-prediction ${prediction === 'draw' ? 'active' : ''}`}
            onClick={() => handlePredictionSelect('draw')}
          >
            <span className="prediction-team">Draw</span>
            {oddsLabel(selected, 'draw') && (
              <span className="prediction-odds">{oddsLabel(selected, 'draw')}</span>
            )}
          </button>
          <button
            type="button"
            className={`btn-prediction ${prediction === 'away_win' ? 'active' : ''}`}
            onClick={() => handlePredictionSelect('away_win')}
          >
            <span className="prediction-team">{selected.away_team}</span>
            {oddsLabel(selected, 'away_win') && (
              <span className="prediction-odds">{oddsLabel(selected, 'away_win')}</span>
            )}
          </button>
        </div>
      )}
    </div>
  );
}
