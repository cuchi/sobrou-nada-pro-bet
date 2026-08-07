import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { devLogin, translateApiError } from '../api/client';
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
      alert(translateApiError(err, t, 'errors.devLoginAlert'));
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
