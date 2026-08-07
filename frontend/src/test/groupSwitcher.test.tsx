import { describe, it, expect, afterEach } from 'vitest';
import { render, cleanup, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '../i18n';
import GroupSwitcher from '../components/GroupSwitcher';
import { AuthProvider } from '../context/AuthContext';

const OWNER_ID = 'u-owner';
const MEMBER_ID = 'u-member';

afterEach(() => {
  cleanup();
  localStorage.clear();
  // @ts-expect-error restore the unstubbed fetch (vitest's jsdom default).
  delete globalThis.fetch;
});

function setupLoggedInUser(currentUserId: string, groups: Array<{ id: string; name: string; balance: number; owner_id: string }>) {
  localStorage.setItem('token', 'fake-token-for-test');
  globalThis.fetch = (async (url: string | URL) => {
    const u = String(url);
    if (u.endsWith('/api/auth/me')) {
      return new Response(
        JSON.stringify({
          user: {
            id: currentUserId,
            email: `${currentUserId}@example.com`,
            name: currentUserId === OWNER_ID ? 'Owner User' : 'Member User',
            avatar_url: null,
          },
          groups,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      );
    }
    return new Response('{}', { status: 200 });
  }) as typeof fetch;
}

describe('GroupSwitcher — invite button ownership', () => {
  // Bug regression: the Invite button used to render for any selected
  // group, even when the current user wasn't the group's owner. Only the
  // group owner can fetch/regenerate the invite code on the backend, so
  // showing the button to non-owners invited a click that 403'd.
  it('shows the Invite button when the current user is the group owner', async () => {
    setupLoggedInUser(OWNER_ID, [
      { id: 'g1', name: 'My group', balance: 1000, owner_id: OWNER_ID },
    ]);

    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <GroupSwitcher selectedGroupId="g1" onSelect={() => {}} />
        </AuthProvider>
      </I18nextProvider>,
    );

    await waitFor(() => {
      expect(container.querySelector('.btn-invite')).not.toBeNull();
    });
  });

  it('hides the Invite button when the current user is not the group owner', async () => {
    setupLoggedInUser(MEMBER_ID, [
      { id: 'g1', name: 'Someone else\'s group', balance: 1000, owner_id: OWNER_ID },
    ]);

    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <AuthProvider>
          <GroupSwitcher selectedGroupId="g1" onSelect={() => {}} />
        </AuthProvider>
      </I18nextProvider>,
    );

    // Wait for AuthProvider to settle (fetchMe resolves, user is loaded).
    // The button must NOT appear once the user is known to be a non-owner.
    await waitFor(() => {
      expect(container.querySelector('.btn-invite')).toBeNull();
    });
  });
});