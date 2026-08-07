import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import en from './locales/en/common.json';
import ptBR from './locales/pt-BR/common.json';

export const SUPPORTED_LANGUAGES = ['en', 'pt-BR'] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];
export const DEFAULT_LANGUAGE: SupportedLanguage = 'pt-BR';

// A `?lng=` query param wins over everything else — used by the smoke test
// (and humans who want to preview a locale without flipping localStorage).
function detectFromQuery(): string | null {
  if (typeof window === 'undefined') return null;
  const lng = new URLSearchParams(window.location.search).get('lng');
  if (!lng) return null;
  return (SUPPORTED_LANGUAGES as readonly string[]).includes(lng) ? lng : null;
}

const queryLng = detectFromQuery();

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { common: en },
      'pt-BR': { common: ptBR },
    },
    lng: queryLng ?? undefined,
    fallbackLng: DEFAULT_LANGUAGE,
    supportedLngs: SUPPORTED_LANGUAGES as unknown as string[],
    ns: ['common'],
    defaultNS: 'common',
    interpolation: { escapeValue: false },
    detection: {
      // localStorage → navigator.language → DEFAULT_LANGUAGE. The query
      // param, if present, has already been applied via `lng` above and is
      // not re-checked here.
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
      lookupLocalStorage: 'i18nextLng',
    },
    react: { useSuspense: false },
  });

export default i18n;
