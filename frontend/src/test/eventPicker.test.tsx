import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '../i18n';
import EventPicker from '../components/EventPicker';
import type { Event, Prediction } from '../types';

function makeEvent(overrides: Partial<Event> = {}): Event {
  return {
    id: 'e1',
    external_id: 'ext-1',
    home_team: 'Flamengo',
    away_team: 'Vasco',
    championship: 'Brasileirão',
    start_time: '2026-08-09T19:00:00Z',
    status: 'scheduled',
    awaiting_result: false,
    home_score: null,
    away_score: null,
    home_odds: 1.5,
    draw_odds: 3.0,
    away_odds: 4.0,
    raw_data: null,
    created_at: '2026-08-07T00:00:00Z',
    ...overrides,
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe('EventPicker — mobile tap-to-scroll', () => {
  it('scrolls the prediction bar into view when a card is clicked', async () => {
    const events = [makeEvent({ id: 'e1' })];

    // Stub fetchEvents to return our fixture.
    globalThis.fetch = (async () =>
      new Response(JSON.stringify(events), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })) as unknown as typeof fetch;

    const scrollIntoView = vi.fn();
    // jsdom doesn't implement scrollIntoView; install it on the prototype
    // so the ref'd element picks it up.
    Element.prototype.scrollIntoView = scrollIntoView;

    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <EventPicker
          onSelect={(_ev: Event, _p: Prediction, _o: number) => {}}
          bettedEventIds={new Set()}
        />
      </I18nextProvider>,
    );

    // Wait for the card to render.
    await waitFor(() => {
      const card = container.querySelector('.event-card');
      expect(card).toBeTruthy();
    });

    const card = container.querySelector('.event-card') as HTMLButtonElement;
    fireEvent.click(card);

    await waitFor(() => {
      expect(scrollIntoView).toHaveBeenCalled();
    });
    expect(scrollIntoView.mock.calls[0][0]).toEqual({ behavior: 'smooth', block: 'center' });
  });
});
