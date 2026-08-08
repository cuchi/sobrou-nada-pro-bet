import { useEffect } from 'react';

type Handler = (e: KeyboardEvent) => void;

/**
 * Bind a handler to a single keyboard shortcut. Skips when the user is
 * typing into a form field (input, textarea, contenteditable) so the
 * shortcut doesn't fire while writing a bet comment or invite code.
 *
 * `key` is the KeyboardEvent.key value (case-insensitive). Modifiers are
 * matched separately: ctrl, alt, shift, meta — pass `false` to require
 * the modifier *not* be pressed, `true` to require it, omit to ignore.
 *
 * Example:
 *   useHotkey('Escape', () => setOpen(false));
 *   useHotkey('k', () => openSearch(), { ctrl: true });
 */
export function useHotkey(
  key: string,
  handler: Handler,
  modifiers: { ctrl?: boolean; alt?: boolean; shift?: boolean; meta?: boolean } = {},
): void {
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (isTypingTarget(e.target)) return;
      if (e.key.toLowerCase() !== key.toLowerCase()) return;
      if (modifiers.ctrl !== undefined && e.ctrlKey !== modifiers.ctrl) return;
      if (modifiers.alt !== undefined && e.altKey !== modifiers.alt) return;
      if (modifiers.shift !== undefined && e.shiftKey !== modifiers.shift) return;
      if (modifiers.meta !== undefined && e.metaKey !== modifiers.meta) return;
      handler(e);
    }
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [key, handler, modifiers.ctrl, modifiers.alt, modifiers.shift, modifiers.meta]);
}

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  if (target.isContentEditable) return true;
  return false;
}