import { useState } from 'react';
import { devLogin } from '../api/client';
import type { AuthResponse } from '../types';

export default function DevLoginButton() {
  const [loading, setLoading] = useState(false);

  const handleClick = async () => {
    setLoading(true);
    try {
      const data = (await devLogin()) as AuthResponse;
      localStorage.setItem('token', data.token);
      // Reload to pick up the token and trigger /api/auth/me
      window.location.reload();
    } catch {
      alert('Dev login failed — is the backend running?');
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
