import { useState, useMemo } from 'react';
import { createBet } from '../api/client';
import type { Bet, Event, Prediction } from '../types';
import EventPicker from './EventPicker';

interface Props {
  groupId: string;
  groupName: string;
  balance: number;
  bets: Bet[];
  onBetCreated: () => void;
}

export default function BetForm({ groupId, groupName, balance, bets, onBetCreated }: Props) {
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null);
  const [prediction, setPrediction] = useState<Prediction | null>(null);
  const [odds, setOdds] = useState<number>(0);
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const bettedEventIds = useMemo(
    () => new Set(bets.filter((b) => b.status === 'pending' && b.event_id).map((b) => b.event_id!)),
    [bets],
  );

  const handleSelect = (ev: Event, pred: Prediction, autoOdds: number) => {
    setSelectedEvent(ev);
    setPrediction(pred);
    setOdds(autoOdds);
    setError(null);
  };

  const handleEventChange = () => {
    setSelectedEvent(null);
    setPrediction(null);
    setOdds(0);
    setAmount('');
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!selectedEvent || !prediction) {
      return;
    }

    const pts = Number(amount);
    if (!amount || pts < 1) {
      setError('Enter a valid amount');
      return;
    }

    if (pts > balance) {
      setError('Not enough points');
      return;
    }

    if (odds < 1.01) {
      setError('No odds available for this prediction');
      return;
    }

    setError(null);
    setLoading(true);
    try {
      await createBet({
        group_id: groupId,
        event_id: selectedEvent.id,
        prediction,
        amount: Number(amount),
        odds,
      });
      setAmount('');
      setOdds(0);
      setSelectedEvent(null);
      setPrediction(null);
      onBetCreated();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error creating bet');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bet-form" noValidate>
      <h2>
        Place a Bet in <strong>{groupName}</strong>
        <span className="balance-pill">{balance.toFixed(0)} pts</span>
      </h2>

      <EventPicker onSelect={handleSelect} onEventChange={handleEventChange} bettedEventIds={bettedEventIds} />

      {selectedEvent && prediction && (
        <>
          {odds > 0 && (
            <div className="odds-display">
              Odds: <strong>{odds}x</strong>
            </div>
          )}
          <div className="bet-inputs">
            <input
              placeholder="Amount (pts)"
              type="text"
              inputMode="numeric"
              pattern="[0-9]*"
              value={amount}
              onChange={(e) => setAmount(e.target.value.replace(/\D/g, ''))}
              onKeyDown={(e) => { if (e.key === 'Enter') e.preventDefault(); }}
            />
            <button type="submit" disabled={loading} className="btn-place-bet">
              {loading ? 'Placing...' : 'Place Bet'}
            </button>
          </div>
        </>
      )}

      {error && <span className="bet-error">{error}</span>}
    </form>
  );
}
