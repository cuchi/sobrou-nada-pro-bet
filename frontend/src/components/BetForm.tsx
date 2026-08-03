import { useState } from 'react';
import { createBet } from '../api/client';

interface Props {
  groupId: string;
  groupName: string;
  balance: number;
  onBetCreated: () => void;
}

export default function BetForm({ groupId, groupName, balance, onBetCreated }: Props) {
  const [amount, setAmount] = useState('');
  const [odds, setOdds] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      await createBet({ group_id: groupId, amount: Number(amount), odds: Number(odds) });
      setAmount('');
      setOdds('');
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
      <input
        placeholder="Odds (e.g. 2.5)"
        type="number"
        step="0.01"
        min="1.01"
        value={odds}
        onChange={(e) => setOdds(e.target.value)}
        required
      />
      <button type="submit" disabled={loading}>
        {loading ? 'Placing...' : 'Place Bet'}
      </button>
      {error && <span className="bet-error">{error}</span>}
    </form>
  );
}
