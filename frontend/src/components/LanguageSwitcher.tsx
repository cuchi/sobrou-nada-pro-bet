import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { SUPPORTED_LANGUAGES, type SupportedLanguage } from '../i18n';

const FLAGS: Record<SupportedLanguage, string> = {
  en: '/flags/us.svg',
  'pt-BR': '/flags/br.svg',
};

function labelKey(lng: SupportedLanguage): 'portuguese' | 'en' {
  return lng === 'pt-BR' ? 'portuguese' : 'en';
}

interface Props {
  /**
   * `trigger` (default) renders a button that opens a dropdown of options.
   * `menu` renders the options list directly — for embedding inside
   * another dropdown (e.g. the user menu) so we don't nest popovers.
   */
  variant?: 'trigger' | 'menu';
}

export function LanguageSwitcher({ variant = 'trigger' }: Props) {
  if (variant === 'menu') return <LanguageOptions />;
  return <LanguageTrigger />;
}

function LanguageTrigger() {
  const { t, i18n } = useTranslation();
  const current = (i18n.resolvedLanguage ?? i18n.language ?? 'en') as SupportedLanguage;
  const [open, setOpen] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    function onDocClick(e: MouseEvent) {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false);
    }
    document.addEventListener('mousedown', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  }, [open]);

  const currentLabel = t(`header.languageSwitcher.${labelKey(current)}`);

  function pick(lng: SupportedLanguage) {
    setOpen(false);
    void i18n.changeLanguage(lng);
  }

  return (
    <div className="language-switcher" ref={wrapperRef}>
      <button
        type="button"
        className={`lang-trigger ${open ? 'open' : ''}`}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={t('header.languageSwitcher.label')}
        title={currentLabel}
        onClick={() => setOpen((o) => !o)}
      >
        <img src={FLAGS[current]} alt="" className="lang-flag" />
        <span aria-hidden="true" className="lang-caret">▾</span>
      </button>

      {open && (
        <ul
          className="lang-menu"
          role="listbox"
          aria-label={t('header.languageSwitcher.label')}
        >
          {SUPPORTED_LANGUAGES.map((lng) => {
            const active = current === lng;
            const label = t(`header.languageSwitcher.${labelKey(lng)}`);
            return (
              <li key={lng} role="none">
                <button
                  type="button"
                  role="option"
                  aria-selected={active}
                  className={`lang-option ${active ? 'active' : ''}`}
                  onClick={() => pick(lng)}
                >
                  <img src={FLAGS[lng]} alt="" className="lang-flag" />
                  <span>{label}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

function LanguageOptions() {
  const { t, i18n } = useTranslation();
  const current = (i18n.resolvedLanguage ?? i18n.language ?? 'en') as SupportedLanguage;
  return (
    <ul className="lang-options" role="listbox" aria-label={t('header.languageSwitcher.label')}>
      {SUPPORTED_LANGUAGES.map((lng) => {
        const active = current === lng;
        const label = t(`header.languageSwitcher.${labelKey(lng)}`);
        return (
          <li key={lng} role="none">
            <button
              type="button"
              role="option"
              aria-selected={active}
              className={`lang-option ${active ? 'active' : ''}`}
              onClick={() => {
                void i18n.changeLanguage(lng);
              }}
            >
              <img src={FLAGS[lng]} alt="" className="lang-flag" />
              <span>{label}</span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
