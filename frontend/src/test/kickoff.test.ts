import { describe, it, expect } from 'vitest';
import { kickoffLabel, type Translator } from '../kickoff';

// Build a translator that returns the actual localised string for a given key.
function makeT(translations: Record<string, string>): Translator {
  return (key, opts) => {
    const raw = translations[key] ?? key;
    return raw.replace(/\{\{(\w+)\}\}/g, (_, name) => String(opts?.[name] ?? `{{${name}}}`));
  };
}

const EN: Record<string, string> = {
  'kickoff.startingNow': 'starting now',
  'kickoff.inOneMinute': 'in <1m',
  'kickoff.inMinutes': 'in {{count}}m',
  'kickoff.inHours': 'in {{count}}h',
  'kickoff.inHoursMinutes': 'in {{hours}}h {{minutes}}m',
  'kickoff.tomorrowAt': 'tomorrow {{time}}',
};

const PT: Record<string, string> = {
  'kickoff.startingNow': 'agora',
  'kickoff.inOneMinute': 'em <1m',
  'kickoff.inMinutes': 'em {{count}}m',
  'kickoff.inHours': 'em {{count}}h',
  'kickoff.inHoursMinutes': 'em {{hours}}h {{minutes}}m',
  'kickoff.tomorrowAt': 'amanhã {{time}}',
};

const NOW = new Date('2025-06-15T12:00:00');
const enT = makeT(EN);
const ptT = makeT(PT);

describe('kickoffLabel — countdown phrases', () => {
  it('returns "starting now" when the match has begun', () => {
    const result = kickoffLabel('2025-06-15T11:59:00', NOW, 'en', enT);
    expect(result).toEqual({ text: 'starting now', relative: true });
  });

  it('returns "in <1m" for sub-minute countdowns (en)', () => {
    const result = kickoffLabel('2025-06-15T12:00:30', NOW, 'en', enT);
    expect(result).toEqual({ text: 'in <1m', relative: true });
  });

  it('returns "em <1m" for sub-minute countdowns (pt-BR)', () => {
    const result = kickoffLabel('2025-06-15T12:00:30', NOW, 'pt-BR', ptT);
    expect(result).toEqual({ text: 'em <1m', relative: true });
  });

  it('returns "in Nm" for under-an-hour countdowns (en)', () => {
    const result = kickoffLabel('2025-06-15T12:05:00', NOW, 'en', enT);
    expect(result).toEqual({ text: 'in 5m', relative: true });
  });

  it('returns "em Nm" for under-an-hour countdowns (pt-BR)', () => {
    const result = kickoffLabel('2025-06-15T12:05:00', NOW, 'pt-BR', ptT);
    expect(result).toEqual({ text: 'em 5m', relative: true });
  });

  it('returns "in Hh" when minutes are 0 and same calendar day (en)', () => {
    const result = kickoffLabel('2025-06-15T15:00:00', NOW, 'en', enT);
    expect(result).toEqual({ text: 'in 3h', relative: true });
  });

  it('returns "em Hh" when minutes are 0 and same calendar day (pt-BR)', () => {
    const result = kickoffLabel('2025-06-15T15:00:00', NOW, 'pt-BR', ptT);
    expect(result).toEqual({ text: 'em 3h', relative: true });
  });

  it('returns "in Hh Mm" for same-day hour+minute countdowns (en)', () => {
    const result = kickoffLabel('2025-06-15T15:14:00', NOW, 'en', enT);
    expect(result).toEqual({ text: 'in 3h 14m', relative: true });
  });

  it('returns "em Hh Mm" for same-day hour+minute countdowns (pt-BR)', () => {
    const result = kickoffLabel('2025-06-15T15:14:00', NOW, 'pt-BR', ptT);
    expect(result).toEqual({ text: 'em 3h 14m', relative: true });
  });
});

describe('kickoffLabel — absolute phrases (relative=false)', () => {
  it('returns "tomorrow HH:MM" with locale-formatted time (en)', () => {
    const result = kickoffLabel('2025-06-16T19:00:00', NOW, 'en', enT);
    expect(result.text).toMatch(/^tomorrow \d{2}:\d{2}$/);
    expect(result.relative).toBe(false);
  });

  it('returns "amanhã HH:MM" with locale-formatted time (pt-BR)', () => {
    const result = kickoffLabel('2025-06-16T19:00:00', NOW, 'pt-BR', ptT);
    expect(result.text).toMatch(/^amanhã \d{2}:\d{2}$/);
    expect(result.relative).toBe(false);
  });

  it('returns "<weekday> HH:MM" for within-7-days matches (en)', () => {
    const result = kickoffLabel('2025-06-18T20:30:00', NOW, 'en', enT);
    // Just verify the shape — exact weekday depends on the locale data.
    expect(result.text).toMatch(/^[A-Za-z]{3} \d{2}:\d{2}$/);
    expect(result.relative).toBe(false);
  });

  it('uses the active locale for the weekday short form (en uses 3-letter English)', () => {
    const result = kickoffLabel('2025-06-18T20:30:00', NOW, 'en', enT);
    // 2025-06-18 is a Wednesday → "Wed" in en.
    expect(result.text).toBe('Wed 20:30');
  });

  it('returns "<DD/MM> HH:MM" for matches more than 7 days out (en)', () => {
    const result = kickoffLabel('2025-07-01T18:00:00', NOW, 'en', enT);
    expect(result.text).toMatch(/^\d{2}\/\d{2} \d{2}:\d{2}$/);
    expect(result.relative).toBe(false);
  });

  it('returns "<DD/MM> HH:MM" for matches more than 7 days out (pt-BR)', () => {
    const result = kickoffLabel('2025-07-01T18:00:00', NOW, 'pt-BR', ptT);
    expect(result.text).toMatch(/^\d{2}\/\d{2} \d{2}:\d{2}$/);
    expect(result.relative).toBe(false);
  });
});

describe('kickoffLabel — passes the locale to Intl.DateTimeFormat', () => {
  it('formats HH:MM with the explicit locale, not the browser default', () => {
    // en-US default uses 12-hour format; "en" with hour12 undefined keeps
    // 24-hour in ICU. We assert the field count and pattern, not a specific
    // locale's convention, to stay portable.
    const result = kickoffLabel('2025-07-01T18:00:00', NOW, 'en', enT);
    // "01/07 18:00" — short date is DD/MM in pt-BR locale but we asked for
    // "en", so the format follows en conventions (MM/DD here).
    expect(result.text).toBe('07/01 18:00');
  });

  it('pt-BR locale renders DD/MM, not MM/DD', () => {
    const result = kickoffLabel('2025-07-01T18:00:00', NOW, 'pt-BR', ptT);
    expect(result.text).toBe('01/07 18:00');
  });
});
