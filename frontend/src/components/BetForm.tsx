import { useState } from 'react';
import { createBet } from '../api/client';
import type { Event, Prediction } from '../types';
import EventPicker from './EventPicker';

interface Props {
  groupId: string;
  groupName: string;
  balance: number;
  onBetCreated: () => void;
}

export default function BetForm({ groupId, groupName, balance, onBetCreated }: Props) {
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null);
  const [prediction, setPrediction] = useState<Prediction | null>(null);
  const [odds, setOdds] = useState<number>(0);
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSelect = (ev: Event, pred: Prediction, autoOdds: number) => {
    setSelectedEvent(ev);
    setPrediction(pred);
    setOdds(autoOdds);
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!selectedEvent || !prediction) {
      setError('Select a match and your prediction first');
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
    <form onSubmit={handleSubmit} className="bet-form">
      <h2>
        Place a Bet in <strong>{groupName}</strong>
        <span className="balance-pill">{balance.toFixed(0)} pts</span>
      </h2>

      <EventPicker onSelect={handleSelect} />

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
              type="number"
              step="1"
              min="1"
              max={balance}
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              required
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
