import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, cleanup, fireEvent, act } from '@testing-library/react';
import { useHotkey } from '../useHotkey';

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function Probe({ keyName, modifiers, handler }: {
  keyName: string;
  modifiers?: Parameters<typeof useHotkey>[2];
  handler: () => void;
}) {
  useHotkey(keyName, handler, modifiers);
  return <div data-testid="probe" />;
}

describe('useHotkey', () => {
  it('fires when the matching key is pressed outside a form field', () => {
    const handler = vi.fn();
    render(<Probe keyName="Escape" handler={handler} />);
    act(() => {
      document.body.focus();
      fireEvent.keyDown(document.body, { key: 'Escape' });
    });
    expect(handler).toHaveBeenCalledOnce();
  });

  it('does not fire when the user is typing in an input', () => {
    const handler = vi.fn();
    const { container } = render(
      <>
        <input data-testid="i" />
        <Probe keyName="Escape" handler={handler} />
      </>,
    );
    const input = container.querySelector('[data-testid="i"]') as HTMLInputElement;
    act(() => {
      fireEvent.keyDown(input, { key: 'Escape' });
    });
    expect(handler).not.toHaveBeenCalled();
  });

  it('respects modifier requirements (Ctrl+K only with Ctrl held)', () => {
    const handler = vi.fn();
    render(<Probe keyName="k" handler={handler} modifiers={{ ctrl: true }} />);
    act(() => {
      document.body.focus();
      // Without Ctrl → ignored
      fireEvent.keyDown(document.body, { key: 'k' });
      // With Ctrl → fires
      fireEvent.keyDown(document.body, { key: 'k', ctrlKey: true });
    });
    expect(handler).toHaveBeenCalledOnce();
  });

  it('respects negative modifier requirements (no Shift held)', () => {
    const handler = vi.fn();
    render(<Probe keyName="/" handler={handler} modifiers={{ shift: false }} />);
    act(() => {
      document.body.focus();
      // With Shift → ignored (we require shift NOT to be held)
      fireEvent.keyDown(document.body, { key: '/', shiftKey: true });
      // Without Shift → fires
      fireEvent.keyDown(document.body, { key: '/' });
    });
    expect(handler).toHaveBeenCalledOnce();
  });

  it('case-insensitive: capital K and lowercase k both match "k"', () => {
    const handler = vi.fn();
    render(<Probe keyName="k" handler={handler} />);
    act(() => {
      document.body.focus();
      fireEvent.keyDown(document.body, { key: 'K' });
    });
    expect(handler).toHaveBeenCalledOnce();
  });

  it('does not fire for unrelated keys', () => {
    const handler = vi.fn();
    render(<Probe keyName="Escape" handler={handler} />);
    act(() => {
      document.body.focus();
      fireEvent.keyDown(document.body, { key: 'Enter' });
      fireEvent.keyDown(document.body, { key: 'a' });
    });
    expect(handler).not.toHaveBeenCalled();
  });
});