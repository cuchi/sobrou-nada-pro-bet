import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, cleanup, act } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '../i18n';
import { OfflineBanner } from '../components/OfflineBanner';

afterEach(() => {
  cleanup();
  // Restore the default jsdom navigator.onLine (true).
  Object.defineProperty(navigator, 'onLine', { configurable: true, value: true });
  vi.restoreAllMocks();
});

describe('OfflineBanner', () => {
  it('renders nothing when navigator.onLine is true', () => {
    Object.defineProperty(navigator, 'onLine', { configurable: true, value: true });
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <OfflineBanner />
      </I18nextProvider>,
    );
    expect(container.querySelector('.offline-banner')).toBeNull();
  });

  it('renders the banner when navigator.onLine is false', () => {
    Object.defineProperty(navigator, 'onLine', { configurable: true, value: false });
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <OfflineBanner />
      </I18nextProvider>,
    );
    const banner = container.querySelector('.offline-banner');
    expect(banner).not.toBeNull();
    expect(banner?.getAttribute('role')).toBe('status');
  });

  it('hides the banner when an `online` event fires', () => {
    Object.defineProperty(navigator, 'onLine', { configurable: true, value: false });
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <OfflineBanner />
      </I18nextProvider>,
    );
    expect(container.querySelector('.offline-banner')).not.toBeNull();

    act(() => {
      window.dispatchEvent(new Event('online'));
    });
    expect(container.querySelector('.offline-banner')).toBeNull();
  });

  it('shows the banner when an `offline` event fires', () => {
    Object.defineProperty(navigator, 'onLine', { configurable: true, value: true });
    const { container } = render(
      <I18nextProvider i18n={i18n}>
        <OfflineBanner />
      </I18nextProvider>,
    );
    expect(container.querySelector('.offline-banner')).toBeNull();

    act(() => {
      window.dispatchEvent(new Event('offline'));
    });
    expect(container.querySelector('.offline-banner')).not.toBeNull();
  });
});