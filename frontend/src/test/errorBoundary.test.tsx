import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, cleanup, fireEvent } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '../i18n';
import { ErrorBoundary } from '../components/ErrorBoundary';

afterEach(() => {
  cleanup();
});

// A child that throws on demand so we can verify the boundary catches it.
function CrashingChild({ shouldThrow }: { shouldThrow: boolean }) {
  if (shouldThrow) throw new Error('boom');
  return <div data-testid="happy-child">all good</div>;
}

describe('ErrorBoundary', () => {
  it('renders children when nothing throws', () => {
    const { getByTestId } = render(
      <I18nextProvider i18n={i18n}>
        <ErrorBoundary scope="test">
          <CrashingChild shouldThrow={false} />
        </ErrorBoundary>
      </I18nextProvider>,
    );
    expect(getByTestId('happy-child')).toBeTruthy();
  });

  it('renders the default fallback when a child throws', () => {
    // Suppress the noisy React error log from the intentional throw.
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const { getByRole, queryByTestId } = render(
      <I18nextProvider i18n={i18n}>
        <ErrorBoundary scope="test">
          <CrashingChild shouldThrow={true} />
        </ErrorBoundary>
      </I18nextProvider>,
    );
    expect(getByRole('alert')).toBeTruthy();
    expect(queryByTestId('happy-child')).toBeNull();
    errSpy.mockRestore();
  });

  it('recovers after the Reload button is clicked (state resets)', () => {
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    let shouldThrow = true;
    function MaybeCrash() {
      if (shouldThrow) throw new Error('boom');
      return <div data-testid="happy-child">all good</div>;
    }
    const { rerender, getByRole, queryByTestId } = render(
      <I18nextProvider i18n={i18n}>
        <ErrorBoundary scope="test">
          <MaybeCrash />
        </ErrorBoundary>
      </I18nextProvider>,
    );
    expect(getByRole('alert')).toBeTruthy();

    // Flip the flag and click Reload — boundary should re-render children.
    shouldThrow = false;
    fireEvent.click(getByRole('button'));
    rerender(
      <I18nextProvider i18n={i18n}>
        <ErrorBoundary scope="test">
          <MaybeCrash />
        </ErrorBoundary>
      </I18nextProvider>,
    );
    expect(queryByTestId('happy-child')).toBeTruthy();
    errSpy.mockRestore();
  });
});