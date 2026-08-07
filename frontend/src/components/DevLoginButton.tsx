import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { devLogin, ApiError, codeToLocaleKey } from '../api/client';
import type { AuthResponse } from '../types';

export default function DevLoginButton() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);

  const handleClick = async () => {
    setLoading(true);
    try {
      const data = (await devLogin()) as AuthResponse;
      localStorage.setItem('token', data.token);
      // Reload to pick up the token and trigger /api/auth/me
      window.location.reload();
    } catch (err) {
      let msg: string;
      if (err instanceof ApiError) {
        const key = `errors.${codeToLocaleKey(err.code)}`;
        msg = t(key, err.params ?? {});
        if (msg === key) msg = err.message;
      } else {
        msg = err instanceof Error ? err.message : t('errors.devLoginAlert');
      }
      alert(msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <button onClick={handleClick} disabled={loading} className="btn-dev-login">
      {loading ? '...' : 'Dev Login'}
    </button>
  );
}
