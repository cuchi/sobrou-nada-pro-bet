import { describe, it, expect, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n, { DEFAULT_LANGUAGE } from '../i18n';
import { UserMenu } from '../components/UserMenu';
import { AuthProvider } from '../context/AuthContext';

// Provide a logged-in user via mocked localStorage + fetch.
// fetchMe() is called by AuthProvider on mount; we stub it to return a user.
function setupLoggedInUser() {
  localStorage.setItem('token', 'fake-token-for-test');
  globalThis.fetch = (async (url: string | URL) => {
    const u = String(url);
    if (u.endsWith('/api/auth/me')) {
      return new Response(
        JSON.stringify({
          user: {
            id: 'u1',
            email: 'kaka@example.com',
            name: 'Kaká Henrique da Silva Santos',
            avatar_url: null,
          },
          groups: [],
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      );
    }
    return new Response('{}', { status: 200 });
  }) as typeof fetch;
}

afterEach(() => {
  cleanup();
  localStorage.clear();
  // @ts-expect-error restore
  delete globalThis.fetch;
});

describe('UserMenu', () => {
  it('renders nothing when no user is signed in', async () => {
    // No token in localStorage → AuthProvider won't call fetchMe → user stays null.
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <UserMenu />
        </AuthProvider>
      </I18nextProvider>,
    );
    // Wait one tick for AuthProvider's useEffect to settle.
    await new Promise((r) => setTimeout(r, 10));
    expect(container.querySelector('.user-menu')).toBeNull();
  });

  it('renders a disabled trigger with a spinner while AuthProvider is loading', async () => {
    // Token in localStorage → AuthProvider will call fetchMe → loading stays
    // true until the (stubbed) fetch resolves. We make the fetch hang so the
    // component stays in the loading state for the duration of the test.
    localStorage.setItem('token', 'fake-token-for-test');
    globalThis.fetch = (async () =>
      // Never resolves: AuthProvider's loading flag stays true.
      new Promise<Response>(() => {})
    ) as typeof fetch;

    await i18n.changeLanguage(DEFAULT_LANGUAGE);
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <UserMenu />
        </AuthProvider>
      </I18nextProvider>,
    );
    // Give the AuthProvider's useEffect one tick to flip loading=true (it
    // starts true anyway).
    await new Promise((r) => setTimeout(r, 10));

    // The wrapper is rendered so the header layout has something to anchor.
    expect(container.querySelector('.user-menu')).not.toBeNull();
    // The trigger is present, disabled, and busy.
    const trigger = container.querySelector<HTMLButtonElement>('.menu-trigger')!;
    expect(trigger).not.toBeNull();
    expect(trigger.disabled).toBe(true);
    expect(trigger.getAttribute('aria-busy')).toBe('true');
    // The spinner glyph is rendered inside (instead of the ⋮ dots).
    expect(container.querySelector('.menu-trigger-spinner')).not.toBeNull();
    expect(trigger.textContent).not.toContain('⋮');
    // A placeholder chip keeps the avatar footprint.
    expect(container.querySelector('.user-chip-loading')).not.toBeNull();
    // Clicking the disabled trigger must NOT open a panel.
    fireEvent.click(trigger);
    expect(container.querySelector('.user-menu-panel')).toBeNull();
  });

  it('renders the avatar + menu trigger when the user is signed in', async () => {
    setupLoggedInUser();
    await i18n.changeLanguage(DEFAULT_LANGUAGE);
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <UserMenu />
        </AuthProvider>
      </I18nextProvider>,
    );
    // Wait for AuthProvider to settle (fetchMe resolves).
    await new Promise((r) => setTimeout(r, 20));
    expect(container.querySelector('.user-menu')).not.toBeNull();
    expect(container.querySelector('.menu-trigger')).not.toBeNull();
    expect(container.querySelector('.user-chip')).not.toBeNull();
  });

  it('opens the panel when the trigger is clicked and closes when an action fires', async () => {
    setupLoggedInUser();
    await i18n.changeLanguage(DEFAULT_LANGUAGE);
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <UserMenu />
        </AuthProvider>
      </I18nextProvider>,
    );
    await new Promise((r) => setTimeout(r, 20));

    const trigger = container.querySelector<HTMLButtonElement>('.menu-trigger')!;
    expect(trigger.getAttribute('aria-expanded')).toBe('false');
    expect(container.querySelector('.user-menu-panel')).toBeNull();

    fireEvent.click(trigger);
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
    expect(container.querySelector('.user-menu-panel')).not.toBeNull();

    // The panel should contain a sign-out menuitem and language options.
    const panel = container.querySelector('.user-menu-panel')!;
    expect(panel.querySelector('[role="menuitem"]')).not.toBeNull();
    // LanguageOptions renders a <ul role="listbox"> with <button role="option"> entries.
    expect(panel.querySelectorAll('[role="option"]').length).toBe(2);

    // Clicking the sign-out menuitem should close the panel.
    // After logout() the parent App swaps <UserMenu/> for the logged-out
    // branch, so the entire chip + trigger + panel unmount together.
    // Asserting aria-expanded on the detached trigger node is racy
    // (React never gets a chance to flush the pending setOpen(false)
    // before unmount), so we check observable post-conditions instead:
    // the panel is gone and the chip is gone (user signed out).
    const signOutBtn = panel.querySelector<HTMLButtonElement>('[role="menuitem"]')!;
    fireEvent.click(signOutBtn);
    await waitFor(() => {
      expect(container.querySelector('.user-menu-panel')).toBeNull();
      expect(container.querySelector('.user-menu')).toBeNull();
    });
  });

  it('switching language via the embedded picker works', async () => {
    setupLoggedInUser();
    await i18n.changeLanguage(DEFAULT_LANGUAGE);
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <UserMenu />
        </AuthProvider>
      </I18nextProvider>,
    );
    await new Promise((r) => setTimeout(r, 20));

    // Open the menu.
    fireEvent.click(container.querySelector<HTMLButtonElement>('.menu-trigger')!);
    // Click the US option.
    const options = Array.from(container.querySelectorAll<HTMLButtonElement>('button[role="option"]'));
    const usOption = options.find((o) => o.querySelector('img')?.getAttribute('src') === '/flags/us.svg')!;
    fireEvent.click(usOption);
    expect(i18n.resolvedLanguage).toBe('en');
  });
});
