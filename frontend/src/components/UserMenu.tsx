import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useAuth } from '../context/AuthContext';
import { LanguageSwitcher } from './LanguageSwitcher';

/**
 * Compact overflow menu that contains everything that used to live in
 * `.header-right` as inline elements: a sign-out action and the language
 * picker. The user avatar + truncated name stays visible inline as a
 * read-only chip; only the actions collapse behind the ⋮ button.
 */
export function UserMenu() {
  const { t } = useTranslation();
  const { user, logout } = useAuth();
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

  if (!user) return null;

  return (
    <div className="user-menu" ref={wrapperRef}>
      <div className="user-chip" title={user.name}>
        {user.avatar_url && (
          <img src={user.avatar_url} alt="" className="avatar" />
        )}
      </div>

      <button
        type="button"
        className={`menu-trigger ${open ? 'open' : ''}`}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={t('header.userMenu.label')}
        title={t('header.userMenu.label')}
        onClick={() => setOpen((o) => !o)}
      >
        <span aria-hidden="true">⋮</span>
      </button>

      {open && (
        <div className="user-menu-panel" role="menu">
          <div className="user-menu-header">
            <span className="user-menu-name" title={user.name}>{user.name}</span>
            <span className="user-menu-email">{user.email}</span>
          </div>

          <div className="user-menu-divider" role="separator" />

          <div className="user-menu-section-label">{t('header.userMenu.language')}</div>
          <div className="user-menu-languages">
            <LanguageSwitcher variant="menu" />
          </div>

          <div className="user-menu-divider" role="separator" />

          <button
            type="button"
            role="menuitem"
            className="user-menu-item user-menu-item-danger"
            onClick={() => {
              setOpen(false);
              logout();
            }}
          >
            <span>{t('header.userMenu.logout')}</span>
          </button>
        </div>
      )}
    </div>
  );
}
