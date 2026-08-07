import { describe, it, expect, afterEach } from 'vitest';
import { render, cleanup, waitFor, act } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '../i18n';
import { ApiError, codeToLocaleKey } from '../api/client';
import { AuthProvider, useAuth } from '../context/AuthContext';
import en from '../locales/en/common.json';
import ptBR from '../locales/pt-BR/common.json';

// Codes that the contract requires every locale to have a non-empty string for.
const CONTRACT_CODES = [
  'authGoogleFailed',
  'authGoogleInvalid',
  'authNotOnAllowlist',
  'notGroupMember',
  'insufficientBalance',
  'alreadyBetOnEvent',
  'eventNotFound',
  'bettingClosed',
  'invalidInviteCode',
  'alreadyInGroup',
  'internal',
] as const;

afterEach(() => {
  cleanup();
  localStorage.clear();
  // @ts-expect-error restore the unstubbed fetch (vitest's jsdom default).
  delete globalThis.fetch;
});

describe('ApiError — locale coverage', () => {
  it.each(['en', 'pt-BR'] as const)(
    'every contract code has a non-empty string in locale %s',
    (locale) => {
      const bundle = locale === 'en' ? en : ptBR;
      const errors = (bundle as { errors: Record<string, string> }).errors;
      for (const code of CONTRACT_CODES) {
        expect(
          typeof errors[code],
          `Locale '${locale}' is missing errors.${code}`,
        ).toBe('string');
        expect(
          errors[code].length,
          `Locale '${locale}' errors.${code} must be non-empty`,
        ).toBeGreaterThan(0);
      }
    },
  );

  it('insufficientBalance interpolates {{have}} and {{bet}} in English', async () => {
    await i18n.changeLanguage('en');
    const out = i18n.t('errors.insufficientBalance', { have: 200, bet: 50 });
    expect(out).toContain('200');
    expect(out).toContain('50');
    // The English template explicitly mentions both numbers in this order.
    expect(out).toBe('Not enough points — you have 200 and tried to bet 50');
  });

  it('insufficientBalance interpolates {{have}} and {{bet}} in pt-BR', async () => {
    await i18n.changeLanguage('pt-BR');
    const out = i18n.t('errors.insufficientBalance', { have: 200, bet: 50 });
    expect(out).toContain('200');
    expect(out).toContain('50');
    expect(out).toBe('Pontos insuficientes — você tem 200 e tentou apostar 50');
  });

  it('authNotOnAllowlist keeps the email address verbatim in both locales', () => {
    const enErrors = (en as { errors: Record<string, string> }).errors;
    const ptErrors = (ptBR as { errors: Record<string, string> }).errors;
    expect(enErrors.authNotOnAllowlist).toContain('paulo@cuchi.me');
    expect(ptErrors.authNotOnAllowlist).toContain('paulo@cuchi.me');
  });
});

describe('ApiError — translation flow through AuthContext.login', () => {
  // Stub the network so googleLogin() rejects with the given wire payload.
  function stubGoogleLoginError(status: number, body: unknown) {
    globalThis.fetch = (async () => {
      return new Response(JSON.stringify(body), {
        status,
        headers: { 'Content-Type': 'application/json' },
      });
    }) as typeof fetch;
  }

  // Probe component: lets the test invoke login() and read the resulting
  // loginError from the AuthContext. Mirrors what the .login-error-banner
  // in App.tsx renders — but without mounting the full App tree.
  function LoginProbe() {
    const { login, loginError } = useAuth();
    // Surface both to the test via the DOM so jsdom-style queries work.
    return (
      <div>
        <button
          className="trigger-login"
          onClick={() => {
            void login('fake-credential');
          }}
        >
          login
        </button>
        <span className="login-error-banner">{loginError ?? ''}</span>
      </div>
    );
  }

  function renderLoginProbe() {
    return render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <LoginProbe />
        </AuthProvider>
      </I18nextProvider>,
    );
  }

  it('renders the localized insufficient_balance message in the banner (en)', async () => {
    await i18n.changeLanguage('en');
    stubGoogleLoginError(400, {
      code: 'insufficient_balance',
      params: { have: 200, bet: 50 },
      message: 'Insufficient balance. You have 200 points, bet is 50.',
    });

    const { container } = renderLoginProbe();
    act(() => {
      container.querySelector<HTMLButtonElement>('.trigger-login')!.click();
    });

    await waitFor(() => {
      const banner = container.querySelector('.login-error-banner')!.textContent;
      expect(banner).toContain('Not enough points');
      expect(banner).toContain('200');
      expect(banner).toContain('50');
    });
    // Belt-and-braces: assert the exact interpolated string for en.
    expect(container.querySelector('.login-error-banner')!.textContent).toBe(
      'Not enough points — you have 200 and tried to bet 50',
    );
  });

  it('renders the localized insufficient_balance message in the banner (pt-BR)', async () => {
    await i18n.changeLanguage('pt-BR');
    stubGoogleLoginError(400, {
      code: 'insufficient_balance',
      params: { have: 200, bet: 50 },
      message: 'Insufficient balance. You have 200 points, bet is 50.',
    });

    const { container } = renderLoginProbe();
    act(() => {
      container.querySelector<HTMLButtonElement>('.trigger-login')!.click();
    });

    await waitFor(() => {
      const banner = container.querySelector('.login-error-banner')!.textContent;
      expect(banner).toContain('Pontos insuficientes');
      expect(banner).toContain('200');
      expect(banner).toContain('50');
    });
    expect(container.querySelector('.login-error-banner')!.textContent).toBe(
      'Pontos insuficientes — você tem 200 e tentou apostar 50',
    );
  });

  it('falls back to ApiError.message when the locale has no key for the code', async () => {
    await i18n.changeLanguage('en');
    stubGoogleLoginError(500, {
      code: 'not_a_real_code',
      params: null,
      message: 'Some fallback message from the server',
    });

    const { container } = renderLoginProbe();
    act(() => {
      container.querySelector<HTMLButtonElement>('.trigger-login')!.click();
    });

    await waitFor(() => {
      const banner = container.querySelector('.login-error-banner')!.textContent;
      expect(banner).toBe('Some fallback message from the server');
    });
  });

  it('translates the auth_not_on_allowlist code in the banner (en)', async () => {
    await i18n.changeLanguage('en');
    stubGoogleLoginError(403, {
      code: 'auth_not_on_allowlist',
      params: null,
      message: 'This app is currently in closed beta. Contact paulo@cuchi.me to request access.',
    });

    const { container } = renderLoginProbe();
    act(() => {
      container.querySelector<HTMLButtonElement>('.trigger-login')!.click();
    });

    await waitFor(() => {
      const banner = container.querySelector('.login-error-banner')!.textContent;
      expect(banner).toContain('closed beta');
      expect(banner).toContain('paulo@cuchi.me');
    });
  });
});

describe('ApiError — class shape', () => {
  it('carries code, params, and message', () => {
    const err = new ApiError('insufficient_balance', { have: 200, bet: 50 }, 'whatever');
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(ApiError);
    expect(err.name).toBe('ApiError');
    expect(err.code).toBe('insufficient_balance');
    expect(err.params).toEqual({ have: 200, bet: 50 });
    expect(err.message).toBe('whatever');
  });

  it('accepts a null params object', () => {
    const err = new ApiError('internal', null, 'boom');
    expect(err.params).toBeNull();
    expect(err.code).toBe('internal');
  });
});

describe('codeToLocaleKey', () => {
  it('maps every contract snake_case code to its camelCase locale key', () => {
    expect(codeToLocaleKey('auth_google_failed')).toBe('authGoogleFailed');
    expect(codeToLocaleKey('auth_google_invalid')).toBe('authGoogleInvalid');
    expect(codeToLocaleKey('auth_not_on_allowlist')).toBe('authNotOnAllowlist');
    expect(codeToLocaleKey('not_group_member')).toBe('notGroupMember');
    expect(codeToLocaleKey('insufficient_balance')).toBe('insufficientBalance');
    expect(codeToLocaleKey('already_bet_on_event')).toBe('alreadyBetOnEvent');
    expect(codeToLocaleKey('event_not_found')).toBe('eventNotFound');
    expect(codeToLocaleKey('betting_closed')).toBe('bettingClosed');
    expect(codeToLocaleKey('invalid_invite_code')).toBe('invalidInviteCode');
    expect(codeToLocaleKey('already_in_group')).toBe('alreadyInGroup');
    expect(codeToLocaleKey('internal')).toBe('internal');
  });

  it('is a no-op for codes with no underscores', () => {
    expect(codeToLocaleKey('internal')).toBe('internal');
  });

  it('returns the input unchanged for unknown future codes (best-effort)', () => {
    // We don't want to silently swallow a brand-new code; if it has no
    // underscores, the mapper returns it verbatim and the catch-block's
    // "key equals input" check falls back to err.message.
    expect(codeToLocaleKey('some_new_code')).toBe('someNewCode');
  });
});