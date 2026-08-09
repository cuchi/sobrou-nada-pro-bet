import { useState, useMemo } from 'react';
import { useTranslation, Trans } from 'react-i18next';
import { createBet, translateApiError } from '../api/client';
import { useAuth } from '../context/AuthContext';
import type { Bet, Event, Prediction } from '../types';
import EventPicker from './EventPicker';

interface Props {
  groupId: string;
  groupName: string;
  balance: number;
  bets: Bet[];
  onBetCreated: (optimistic: Bet) => void;
  onBetSettled: () => void;
  onBetFailed: () => void;
}

export default function BetForm({ groupId, groupName, balance, bets, onBetCreated, onBetSettled, onBetFailed }: Props) {
  const { t } = useTranslation();
  const { user } = useAuth();
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null);
  const [prediction, setPrediction] = useState<Prediction | null>(null);
  const [odds, setOdds] = useState<number>(0);
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pickerKey, setPickerKey] = useState(0);

  const bettedEventIds = useMemo(
    () => new Set(
      bets
        .filter((b) => b.user_id === user?.id && b.status === 'pending' && b.event_id)
        .map((b) => b.event_id!)
    ),
    [bets, user],
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
      setError(t('betForm.errors.invalidAmount'));
      return;
    }

    if (pts > balance) {
      setError(t('betForm.errors.insufficientBalance'));
      return;
    }

    if (odds < 1.01) {
      setError(t('betForm.errors.noOdds'));
      return;
    }

    setError(null);
    setLoading(true);

    const optimistic: Bet = {
      id: `optimistic-${Date.now()}`,
      user_id: user?.id || '',
      group_id: groupId,
      event_id: selectedEvent.id,
      prediction,
      amount: Number(amount),
      odds,
      status: 'pending',
      created_at: new Date().toISOString(),
      user_name: user?.name || '',
      user_email: '',
      user_avatar_url: user?.avatar_url || null,
      home_team: selectedEvent.home_team,
      away_team: selectedEvent.away_team,
      start_time: selectedEvent.start_time,
    };

    onBetCreated(optimistic);
    setAmount('');
    setOdds(0);
    setSelectedEvent(null);
    setPrediction(null);
    setPickerKey(k => k + 1);

    try {
      await createBet({
        group_id: groupId,
        event_id: selectedEvent.id,
        prediction,
        amount: Number(amount),
        odds,
      });
      onBetSettled();
    } catch (err) {
      onBetFailed();
      setError(translateApiError(err, t, 'betForm.errors.generic'));
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bet-form" noValidate>
      <h2>
        <Trans i18nKey="betForm.heading" values={{ groupName }} components={{ name: <strong /> }} />
        <span className="balance-pill">{balance.toFixed(0)} pts</span>
      </h2>

      <EventPicker resetKey={pickerKey} onSelect={handleSelect} onEventChange={handleEventChange} bettedEventIds={bettedEventIds} />

      {selectedEvent && prediction && (
        <>
          {odds > 0 && (
            <div className="odds-display">
              {t('betForm.odds', { odds })}
            </div>
          )}
          <div className="bet-inputs">
            <input
              placeholder={t('betForm.amountPlaceholder')}
              type="text"
              inputMode="numeric"
              pattern="[0-9]*"
              value={amount}
              onChange={(e) => setAmount(e.target.value.replace(/\D/g, ''))}
              onKeyDown={(e) => {
                // Enter submits the surrounding form (browser default would
                // already do this for type="submit" buttons, but this input
                // swallows it because it's not a submit button itself).
                if (e.key === 'Enter') {
                  e.preventDefault();
                  handleSubmit(e as unknown as React.FormEvent);
                }
              }}
            />
            <button type="submit" disabled={loading} className="btn-place-bet">
              {loading ? t('betForm.placing') : t('betForm.placeBet')}
            </button>
          </div>
        </>
      )}

      {error && <span className="bet-error">{error}</span>}
    </form>
  );
}
