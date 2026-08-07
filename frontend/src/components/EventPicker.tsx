import { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { fetchEvents } from '../api/client';
import { getCrestUrl } from '../crests';
import { kickoffLabel } from '../kickoff';
import type { Event, EventStatus, Prediction } from '../types';
import { useActiveLocale } from './Points';
import { Spinner } from './Spinner';
import { usePolling } from '../usePolling';

interface Props {
  onSelect: (event: Event, prediction: Prediction, odds: number) => void;
  onEventChange?: () => void;
  bettedEventIds: Set<string>;
  resetKey?: number;
}

const SEVEN_DAYS_MS = 7 * 24 * 60 * 60 * 1000;
const RECENT_RESULTS_LIMIT = 10;

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

function StatusBadge({ status, awaitingResult }: { status: EventStatus; awaitingResult?: boolean }) {
  const { t } = useTranslation();
  // awaiting-result is a derived overlay: the backend labeled this row
  // `finished` but its stored status is still `scheduled`, meaning the match
  // window elapsed without /admin/bets/resolve being called yet. Visually
  // distinct so users don't read it as a confirmed outcome.
  if (awaitingResult && status === 'finished') {
    return (
      <span
        className="event-status-badge awaiting-result"
        aria-label={t('eventPicker.statusBadge.ariaAwaitingResult')}
      >
        {t('eventPicker.statusBadge.awaitingResult')}
      </span>
    );
  }
  const ariaKey =
    status === 'scheduled' ? 'ariaScheduled'
    : status === 'live' ? 'ariaLive'
    : status === 'finished' ? 'ariaFinished'
    : 'ariaCancelled';
  return (
    <span
      className={`event-status-badge ${status}`}
      aria-label={t(`eventPicker.statusBadge.${ariaKey}`)}
    >
      {t(`eventPicker.statusBadge.${status}`)}
    </span>
  );
}

/** True for any event that should be treated as not-pickable. */
function isClosed(status: EventStatus): boolean {
  return status !== 'scheduled';
}

export default function EventPicker({ onSelect, onEventChange, bettedEventIds, resetKey }: Props) {
  const { t } = useTranslation();
  const locale = useActiveLocale();
  const [events, loading] = usePolling<Event[]>(
    useCallback(() => fetchEvents('scheduled,live,finished,cancelled') as Promise<Event[]>, []),
    60_000,
  );
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [prediction, setPrediction] = useState<Prediction | null>(null);
  const [query, setQuery] = useState('');
  const [showRecent, setShowRecent] = useState(false);

  // Lazy locale-driven formatters — recreated when the active locale flips.
  const recentWhenFmt = useMemo(
    () => new Intl.DateTimeFormat(locale, {
      day: '2-digit',
      month: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }),
    [locale],
  );
  const cardDateFmt = useMemo(
    () => new Intl.DateTimeFormat(locale, { day: '2-digit', month: '2-digit' }),
    [locale],
  );
  const cardTimeFmt = useMemo(
    () => new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }),
    [locale],
  );

  /** Format a recent match's start time as "DD/MM HH:MM" for compact row display. */
  function formatRecentWhen(iso: string): string {
    return recentWhenFmt.format(new Date(iso));
  }

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

  const { upcoming, cancelled, awaiting, recent } = useMemo(() => {
    const all = events ?? [];
    const now = Date.now();
    const upcomingList: Event[] = [];
    const cancelledList: Event[] = [];
    const awaitingList: Event[] = [];
    const recentList: Event[] = [];
    for (const ev of all) {
      if (ev.status === 'cancelled') {
        // Cancelled matches surface in the same Recent results collapsible
        // for the same 7-day window as finished matches. After that they
        // drop off — pending bets on them remain accessible via BetList.
        const t = new Date(ev.start_time).getTime();
        if (now - t <= SEVEN_DAYS_MS) cancelledList.push(ev);
      } else if (ev.status === 'finished') {
        if (ev.awaiting_result) {
          // Window elapsed but not yet resolved by /admin/bets/resolve.
          // Grouped under the "Awaiting result" subheading inside the Recent
          // results collapsible — visually distinct from finalized Finished
          // rows so users don't read them as confirmed outcomes.
          const t = new Date(ev.start_time).getTime();
          if (now - t <= SEVEN_DAYS_MS) awaitingList.push(ev);
        } else {
          const t = new Date(ev.start_time).getTime();
          if (now - t <= SEVEN_DAYS_MS) recentList.push(ev);
        }
      } else {
        upcomingList.push(ev);
      }
    }
    cancelledList.sort((a, b) => new Date(b.start_time).getTime() - new Date(a.start_time).getTime());
    awaitingList.sort((a, b) => new Date(b.start_time).getTime() - new Date(a.start_time).getTime());
    recentList.sort((a, b) => new Date(b.start_time).getTime() - new Date(a.start_time).getTime());
    recentList.length = Math.min(recentList.length, RECENT_RESULTS_LIMIT);
    return { upcoming: upcomingList, cancelled: cancelledList, awaiting: awaitingList, recent: recentList };
  }, [events]);

  const selected = events?.find((e) => e.id === selectedId);
  const selectedIsClosed = selected ? isClosed(selected.status) : false;
  const selectedAwaitingResult = selected?.awaiting_result === true;

  // Single chronological list across cancelled, awaiting and finished rows
  // — newest first. The status badge inside each row tells the user which
  // bucket a match belongs to, so a separate subheading per bucket isn't
  // needed.
  const previous = useMemo(() => {
    return [...cancelled, ...awaiting, ...recent].sort(
      (a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime(),
    );
  }, [cancelled, awaiting, recent]);

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
      <h3 className="event-picker-heading">{t('eventPicker.heading')}</h3>
      <Spinner label={t('eventPicker.loading')} />
    </div>
  );

  const q = normalize(query.trim());
  const filterFn = (ev: Event) =>
    !q || normalize(ev.home_team).includes(q) || normalize(ev.away_team).includes(q);
  const upcomingFiltered = upcoming.filter(filterFn);
  const hasPrevious = previous.length > 0;
  const recentTotal = previous.length;

  return (
    <div className="event-picker">
      {hasPrevious && (
        <details
          className="recent-results"
          open={showRecent}
          onToggle={(e) => setShowRecent((e.target as HTMLDetailsElement).open)}
        >
          <summary className="recent-results-summary">{t('eventPicker.recentResults', { count: recentTotal })}</summary>
          <div className="recent-results-list">
            {previous.map((ev) => (
              <div key={ev.id} className={`recent-result event-status-${ev.status}`}>
                <span className="recent-result-teams">
                  <span className="recent-result-team">{ev.home_team}</span>
                  {' vs '}
                  <span className="recent-result-team">{ev.away_team}</span>
                </span>
                <div className="recent-result-meta">
                  {ev.home_score != null && ev.away_score != null && (
                    <span className="recent-result-score">
                      {ev.home_score} – {ev.away_score}
                    </span>
                  )}
                  <StatusBadge status={ev.status} awaitingResult={ev.awaiting_result} />
                  <span className="recent-result-when">
                    {formatRecentWhen(ev.start_time)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </details>
      )}

      <h3 className="event-picker-heading">{t('eventPicker.heading')}</h3>

      {upcoming.length > 0 && (
        <input
          type="search"
          className="event-search"
          placeholder={t('eventPicker.searchPlaceholder')}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      )}

      {upcoming.length === 0 ? (
        <p className="no-events">{t('eventPicker.noEvents')}</p>
      ) : upcomingFiltered.length === 0 ? (
        <p className="no-events">{t('eventPicker.noMatchesForQuery', { query })}</p>
      ) : (
        <div className="events-grid">
          {upcomingFiltered.map((ev) => {
            const betted = bettedEventIds.has(ev.id);
            const closed = isClosed(ev.status);
            const disabled = betted || closed;
            const kickoff = ev.status === 'scheduled' ? kickoffLabel(ev.start_time, new Date(), locale, t) : null;
            return (
              <button
                key={ev.id}
                type="button"
                className={`event-card ${selectedId === ev.id ? 'selected' : ''} ${betted ? 'betted' : ''} ${ev.status}`}
                onClick={() => !disabled && handleEventSelect(ev.id)}
                disabled={disabled}
                aria-label={t('eventPicker.cardAriaLabel', { home: ev.home_team, away: ev.away_team, status: t(`eventPicker.cardStatus.${ev.status}`) })}
              >
                <span className="team-name home">{ev.home_team}</span>
                <span className="matchup-core">
                  <TeamCrest name={ev.home_team} />
                  <span className="vs-label">{t('eventPicker.vs')}</span>
                  <TeamCrest name={ev.away_team} />
                </span>
                <span className="team-name away">{ev.away_team}</span>
                <div className="event-odds">
                  <span className="odd-badge">{ev.home_odds != null ? ev.home_odds : '-'}</span>
                  <span className="odd-badge">{ev.draw_odds != null ? ev.draw_odds : '-'}</span>
                  <span className="odd-badge">{ev.away_odds != null ? ev.away_odds : '-'}</span>
                </div>
                <div className="event-info">
                  <StatusBadge status={ev.status} awaitingResult={ev.awaiting_result} />
                  {kickoff ? (
                    <span className={`event-kickoff ${kickoff.relative ? 'relative' : ''}`}>
                      {kickoff.text}
                    </span>
                  ) : (
                    <>
                      <span className="event-date">
                        {cardDateFmt.format(new Date(ev.start_time))}
                      </span>
                      <span className="event-time">
                        {cardTimeFmt.format(new Date(ev.start_time))}
                      </span>
                    </>
                  )}
                </div>
              </button>
            );
          })}
        </div>
      )}

      {selected && (
        selectedIsClosed ? (
          <p className="event-closed-notice">
            {selectedAwaitingResult && t('eventPicker.closedNotice.awaitingResult')}
            {!selectedAwaitingResult && selected.status === 'live' && t('eventPicker.closedNotice.live')}
            {!selectedAwaitingResult && selected.status === 'finished' && t('eventPicker.closedNotice.finished')}
            {!selectedAwaitingResult && selected.status === 'cancelled' && t('eventPicker.closedNotice.cancelled')}
          </p>
        ) : (
          <div className="prediction-bar">
            <span className="prediction-label">{t('eventPicker.predictionBar.yourPick')}</span>
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
              <span className="prediction-team">{t('eventPicker.predictionBar.draw')}</span>
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
        )
      )}
    </div>
  );
}
