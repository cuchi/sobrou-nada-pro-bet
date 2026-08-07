import { useTranslation } from 'react-i18next';
import i18n, { DEFAULT_LANGUAGE, SUPPORTED_LANGUAGES } from '../i18n';

/**
 * Returns the active i18next language, falling back to the default if it's
 * not one of our supported locales. Use this to feed `Intl.NumberFormat` and
 * `Intl.DateTimeFormat` so date/number formatting tracks the UI locale.
 */
export function useActiveLocale(): string {
  const { i18n: instance } = useTranslation();
  const lang = instance.resolvedLanguage ?? instance.language ?? DEFAULT_LANGUAGE;
  return (SUPPORTED_LANGUAGES as readonly string[]).includes(lang) ? lang : DEFAULT_LANGUAGE;
}

/**
 * Format a number as a localised integer + " pts" suffix. Locale-aware so
 * `1234` renders as `1,234 pts` in en and `1.234 pts` in pt-BR.
 */
export function formatPoints(amount: number, locale: string = i18n.resolvedLanguage ?? DEFAULT_LANGUAGE): string {
  const intl = new Intl.NumberFormat(locale, { maximumFractionDigits: 0 });
  return `${intl.format(amount)} pts`;
}

/**
 * Render `formatPoints(amount)` inline so all point-amount cells in the app
 * stay in sync. Pass `tag` to wrap in a different element (default span).
 */
export function Points({ amount, tag: Tag = 'span' }: { amount: number; tag?: 'span' | 'strong' }) {
  const locale = useActiveLocale();
  return <Tag>{formatPoints(amount, locale)}</Tag>;
}
