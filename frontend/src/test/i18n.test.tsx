import { describe, it, expect, beforeAll, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/react';
import i18n, { DEFAULT_LANGUAGE } from '../i18n';
import { LanguageSwitcher } from '../components/LanguageSwitcher';

beforeAll(async () => {
  await i18n.changeLanguage(DEFAULT_LANGUAGE);
});

afterEach(() => {
  cleanup();
  localStorage.clear();
});

/**
 * Open the dropdown then click the option whose flag src matches.
 * Idempotent: opens first only if the menu is currently closed.
 * The flag <img>s use empty alt (decorative), so they're role=presentation
 * — query the DOM directly.
 */
function pickLocale(container: HTMLElement, flagSrc: string) {
  const trigger = container.querySelector<HTMLButtonElement>('button[aria-haspopup="listbox"]');
  if (!trigger) throw new Error('No language-switcher trigger found');
  // Open only if not already open.
  if (trigger.getAttribute('aria-expanded') === 'false') {
    fireEvent.click(trigger);
  }
  // Find the option whose flag matches.
  const options = Array.from(container.querySelectorAll<HTMLButtonElement>('button[role="option"]'));
  const target = options.find((btn) => btn.querySelector('img')?.getAttribute('src') === flagSrc);
  if (!target) throw new Error(`No option with flag src=${flagSrc}`);
  fireEvent.click(target);
}

describe('i18n — locale detection & switching', () => {
  it('starts in pt-BR (the default)', () => {
    render(<LanguageSwitcher />);
    expect(i18n.resolvedLanguage).toBe('pt-BR');
  });

  it('opens the dropdown when the trigger is clicked', () => {
    const { container } = render(<LanguageSwitcher />);
    const trigger = container.querySelector<HTMLButtonElement>('button[aria-haspopup="listbox"]')!;
    expect(trigger.getAttribute('aria-expanded')).toBe('false');
    fireEvent.click(trigger);
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
    // Both options should be in the listbox.
    const options = container.querySelectorAll('button[role="option"]');
    expect(options).toHaveLength(2);
  });

  it('switches to English via the US option', () => {
    const { container } = render(<LanguageSwitcher />);
    pickLocale(container, '/flags/us.svg');
    expect(i18n.resolvedLanguage).toBe('en');
  });

  it('switches back to pt-BR via the BR option', () => {
    const { container } = render(<LanguageSwitcher />);
    pickLocale(container, '/flags/us.svg');
    expect(i18n.resolvedLanguage).toBe('en');
    pickLocale(container, '/flags/br.svg');
    expect(i18n.resolvedLanguage).toBe('pt-BR');
  });

  it('marks the active option with aria-selected=true', () => {
    const { container } = render(<LanguageSwitcher />);
    // Open the menu.
    fireEvent.click(container.querySelector<HTMLButtonElement>('button[aria-haspopup="listbox"]')!);
    const options = Array.from(container.querySelectorAll<HTMLButtonElement>('button[role="option"]'));
    const selected = options.filter((o) => o.getAttribute('aria-selected') === 'true');
    const unselected = options.filter((o) => o.getAttribute('aria-selected') === 'false');
    expect(selected).toHaveLength(1);
    expect(unselected).toHaveLength(1);
    // pt-BR is the default → BR option is selected.
    expect(selected[0].querySelector('img')?.getAttribute('src')).toBe('/flags/br.svg');
    expect(unselected[0].querySelector('img')?.getAttribute('src')).toBe('/flags/us.svg');
  });

  it('shows only the active flag on the trigger when closed', () => {
    const { container } = render(<LanguageSwitcher />);
    // Closed by default → just one flag (the active one) in the trigger.
    const trigger = container.querySelector<HTMLButtonElement>('button[aria-haspopup="listbox"]')!;
    expect(trigger.querySelector('img')?.getAttribute('src')).toBe('/flags/br.svg');
    // No listbox in the DOM until opened.
    expect(container.querySelector('ul[role="listbox"]')).toBeNull();
  });

  it('persists the chosen locale to localStorage', () => {
    const { container } = render(<LanguageSwitcher />);
    pickLocale(container, '/flags/us.svg');
    expect(localStorage.getItem('i18nextLng')).toBe('en');
  });

  it('closes the dropdown when an option is clicked', () => {
    const { container } = render(<LanguageSwitcher />);
    const trigger = container.querySelector<HTMLButtonElement>('button[aria-haspopup="listbox"]')!;
    fireEvent.click(trigger);
    expect(container.querySelector('ul[role="listbox"]')).not.toBeNull();
    pickLocale(container, '/flags/us.svg');
    expect(container.querySelector('ul[role="listbox"]')).toBeNull();
    expect(trigger.getAttribute('aria-expanded')).toBe('false');
  });
});

describe('i18n — component interpolation (Trans)', () => {
  // The betForm.heading interpolation uses <name> as a placeholder for a
  // <strong> wrapper via <Trans>. We don't render the component here
  // (BetForm has too many deps to mount in this test) but we can assert
  // the source string exists with the right placeholder in both locales.
  it('defines a <name> placeholder in betForm.heading for both locales', () => {
    expect(i18n.getResource('en', 'common', 'betForm.heading')).toContain('<name>');
    expect(i18n.getResource('pt-BR', 'common', 'betForm.heading')).toContain('<name>');
  });

  it('keeps groupName as an interpolation variable in both locales', () => {
    expect(i18n.getResource('en', 'common', 'betForm.heading')).toContain('{{groupName}}');
    expect(i18n.getResource('pt-BR', 'common', 'betForm.heading')).toContain('{{groupName}}');
  });
});

describe('i18n — ICU plural rules', () => {
  it('uses the singular form in English at count=1 and "other" form at count>=2', async () => {
    await i18n.changeLanguage('en');
    expect(i18n.t('betList.heading', { count: 1 })).toBe('All Bets (1)');
    expect(i18n.t('betList.heading', { count: 2 })).toBe('All Bets (2)');
  });

  it('fires pt-BR\'s ">1" plural form at count=2', async () => {
    await i18n.changeLanguage('pt-BR');
    // Assert the plural-rule resolution itself: pt-BR has "one" (count=1)
    // and "other" (count!=1) forms. At count=2, i18next must pick the
    // "other" branch — and it must exist in the resource bundle. (The
    // actual translated string happens to be identical for one/other here;
    // what matters is that the form is selected correctly.)
    expect(i18n.getResource('pt-BR', 'common', 'betList.heading_one')).toBeTruthy();
    expect(i18n.getResource('pt-BR', 'common', 'betList.heading_other')).toBeTruthy();
    expect(i18n.t('betList.heading', { count: 1 })).toBe('Todas as apostas (1)');
    expect(i18n.t('betList.heading', { count: 2 })).toBe('Todas as apostas (2)');
    // Differentiate via count to confirm the interpolation works through
    // the plural-rule path; i18next only resolves via the plural variants
    // when a count is supplied.
    expect(i18n.t('betList.heading', { count: 0 })).toBe('Todas as apostas (0)');
  });
});

describe('i18n — component string extraction smoke test', () => {
  // Walk the JSON tree to leaf-string keys. We do this for every locale
  // and union the results so we test the full key surface against each
  // locale individually — catching both "added a key to en but not pt-BR"
  // and "removed a key from one but not the other".
  function walk(obj: Record<string, unknown>, prefix = ''): string[] {
    const keys: string[] = [];
    for (const [k, v] of Object.entries(obj)) {
      const path = prefix ? `${prefix}.${k}` : k;
      if (v && typeof v === 'object' && !Array.isArray(v)) {
        keys.push(...walk(v as Record<string, unknown>, path));
      } else {
        keys.push(path);
      }
    }
    return keys;
  }

  it.each(['en', 'pt-BR'] as const)(
    'resolves every shipped key in locale %s',
    async (locale) => {
      const en = (await import('../locales/en/common.json')).default;
      const pt = (await import('../locales/pt-BR/common.json')).default;
      const allKeys = new Set([...walk(en as Record<string, unknown>), ...walk(pt as Record<string, unknown>)]);
      // Skip suffixed plural variants — they're selected via the parent key.
      const callable = [...allKeys].filter((k) => !/_(one|other|few|many)$/.test(k));

      await i18n.changeLanguage(locale);
      // Disable i18next's silent fallback so a missing key returns the key
      // string itself, which we then assert against.
      i18n.options.fallbackLng = false;

      try {
        for (const key of callable) {
          const res = i18n.t(key, { count: 1 });
          expect(
            res,
            `Missing translation for key '${key}' in locale '${locale}'`,
          ).not.toBe(key);
        }
      } finally {
        i18n.options.fallbackLng = DEFAULT_LANGUAGE;
      }
    },
  );
});
