/**
 * Formats a kickoff countdown relative to `now`.
 *
 * Rules:
 * - Past or < 60s away        → "starting now"
 * - < 1h away                 → "in Nm" (e.g. "in 47m")
 * - later today (< 24h, same calendar day) → "in Hh Mm" (e.g. "in 3h 14m")
 * - tomorrow                  → "tomorrow HH:MM"
 * - within 7 days             → weekday + HH:MM in the browser locale, e.g. "Sat 19:00"
 * - otherwise                 → locale date + HH:MM
 *
 * Returns `{ text, relative }` — `relative` is true when the label is a
 * countdown phrase ("in …" / "starting now"), used by the UI to colour it.
 */
export function kickoffLabel(startIso: string, now: Date = new Date()): { text: string; relative: boolean } {
  const start = new Date(startIso);
  const diffMs = start.getTime() - now.getTime();

  if (diffMs <= 0) return { text: 'starting now', relative: true };
  if (diffMs < 60_000) return { text: 'in <1m', relative: true };

  const minutes = Math.floor(diffMs / 60_000);
  if (minutes < 60) return { text: `in ${minutes}m`, relative: true };

  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (isSameLocalDay(start, now)) {
    return { text: mins === 0 ? `in ${hours}h` : `in ${hours}h ${mins}m`, relative: true };
  }

  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  if (isSameLocalDay(start, tomorrow)) {
    return { text: `tomorrow ${formatHm(start)}`, relative: false };
  }

  const weekAway = new Date(now);
  weekAway.setDate(weekAway.getDate() + 7);
  if (start < weekAway) {
    return { text: `${formatWeekday(start)} ${formatHm(start)}`, relative: false };
  }

  return { text: `${formatShortDate(start)} ${formatHm(start)}`, relative: false };
}

function isSameLocalDay(a: Date, b: Date): boolean {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
}

function formatHm(d: Date): string {
  return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(d);
}

function formatWeekday(d: Date): string {
  return new Intl.DateTimeFormat(undefined, { weekday: 'short' }).format(d);
}

function formatShortDate(d: Date): string {
  return new Intl.DateTimeFormat(undefined, { day: '2-digit', month: '2-digit' }).format(d);
}