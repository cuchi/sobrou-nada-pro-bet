import { GoogleLogin } from '@react-oauth/google';
import { useAuth } from '../context/AuthContext';

export default function GoogleLoginButton() {
  const { login } = useAuth();

  return (
    <GoogleLogin
      onSuccess={(res) => {
        if (res.credential) login(res.credential);
      }}
      onError={() => console.error('Google login failed')}
      theme="filled_black"
      size="medium"
      shape="pill"
      text="signin_with"
    />
  );
}
