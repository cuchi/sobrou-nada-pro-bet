/**
 * Formats a kickoff countdown relative to `now`.
 *
 * Rules:
 * - Past or < 60s away              → "starting now" / "agora"
 * - < 1m away (i.e. 1–59s)          → "in <1m" / "em <1m"
 * - < 1h away                       → "in Nm" / "em Nm" (e.g. "in 47m")
 * - later today, same calendar day  → "in Hh Mm" / "em Hh Mm" (e.g. "in 3h 14m")
 * - tomorrow                        → "tomorrow HH:MM" / "amanhã HH:MM"
 * - within 7 days                   → localised weekday + HH:MM, e.g. "Sat 19:00"
 * - otherwise                       → localised short date + HH:MM
 *
 * Returns `{ text, relative }` — `relative` is true when the label is a
 * countdown phrase ("in …" / "starting now" / "em …" / "agora"), used by the
 * UI to colour it via the `.event-kickoff.relative` CSS class.
 *
 * `locale` is an explicit BCP-47 code (e.g. `"en"`, `"pt-BR"`). `t` is an
 * i18next-compatible translator; pass `i18n.t.bind(i18n)` from a component.
 */
export interface KickoffLabel {
  text: string;
  relative: boolean;
}

export type Translator = (key: string, options?: Record<string, unknown>) => string;

export function kickoffLabel(
  startIso: string,
  now: Date = new Date(),
  locale: string,
  t: Translator,
): KickoffLabel {
  const start = new Date(startIso);
  const diffMs = start.getTime() - now.getTime();

  if (diffMs <= 0) return { text: t('kickoff.startingNow'), relative: true };
  if (diffMs < 60_000) return { text: t('kickoff.inOneMinute'), relative: true };

  const minutes = Math.floor(diffMs / 60_000);
  if (minutes < 60) {
    return { text: t('kickoff.inMinutes', { count: minutes }), relative: true };
  }

  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (isSameLocalDay(start, now)) {
    return {
      text: mins === 0
        ? t('kickoff.inHours', { count: hours })
        : t('kickoff.inHoursMinutes', { hours, minutes: mins }),
      relative: true,
    };
  }

  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  if (isSameLocalDay(start, tomorrow)) {
    return { text: t('kickoff.tomorrowAt', { time: formatHm(start, locale) }), relative: false };
  }

  const weekAway = new Date(now);
  weekAway.setDate(weekAway.getDate() + 7);
  if (start < weekAway) {
    return { text: `${formatWeekday(start, locale)} ${formatHm(start, locale)}`, relative: false };
  }

  return { text: `${formatShortDate(start, locale)} ${formatHm(start, locale)}`, relative: false };
}

function isSameLocalDay(a: Date, b: Date): boolean {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
}

function formatHm(d: Date, locale: string): string {
  // `hour12: false` to force 24-hour format across all locales — a kickoff
  // app that shows "06:00 PM" feels off when countdowns mix minutes and
  // hours (e.g. "in 3h 14m" → "06:00 PM" jumps register).
  return new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(d);
}

function formatWeekday(d: Date, locale: string): string {
  return new Intl.DateTimeFormat(locale, { weekday: 'short' }).format(d);
}

function formatShortDate(d: Date, locale: string): string {
  return new Intl.DateTimeFormat(locale, { day: '2-digit', month: '2-digit' }).format(d);
}
