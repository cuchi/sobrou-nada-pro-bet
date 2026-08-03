import { useState } from 'react';
import { createBet } from '../api/client';

export default function BetForm({ onBetCreated }: { onBetCreated: () => void }) {
  const [amount, setAmount] = useState('');
  const [odds, setOdds] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      await createBet({ amount: Number(amount), odds: Number(odds) });
      setAmount('');
      setOdds('');
      onBetCreated();
    } catch {
      alert('Error creating bet');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bet-form">
      <h2>Place a Bet</h2>
      <input
        placeholder="Amount ($)"
        type="number"
        step="0.01"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        required
      />
      <input
        placeholder="Odds (e.g. 2.5)"
        type="number"
        step="0.01"
        value={odds}
        onChange={(e) => setOdds(e.target.value)}
        required
      />
      <button type="submit" disabled={loading}>
        {loading ? 'Placing...' : 'Place Bet'}
      </button>
    </form>
  );
}
