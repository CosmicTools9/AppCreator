import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from '../stores/auth';
import { setRefreshToken } from '../api/client';

const SSO_LOGIN_URL = import.meta.env.VITE_SSO_LOGIN_URL || 'http://localhost:9002/login';
const CALLBACK_URL = `${window.location.origin}/login`;

export function LoginPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { login, isAuthenticated, isLoading } = useAuth();
  const [error, setError] = useState('');
  const [username, setUsername] = useState('');
  const [authMode, setAuthMode] = useState<'sso' | 'standalone' | 'loading'>('loading');
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  useEffect(() => {
    fetch('/api/creator/status')
      .then((r) => r.json())
      .then((data: { auth_mode?: string }) => {
        if (data.auth_mode === 'standalone') setAuthMode('standalone');
        else setAuthMode('sso');
      })
      .catch(() => setAuthMode('sso'));
  }, []);

  useEffect(() => {
    if (isAuthenticated) navigate('/workspace', { replace: true });
  }, [isAuthenticated]);

  useEffect(() => {
    const t = searchParams.get('token');
    if (t) login(t);
  }, [searchParams]);

  useEffect(() => {
    if (isAuthenticated) {
      const next = searchParams.get('next') || '/workspace';
      navigate(next, { replace: true });
    }
  }, [isAuthenticated]);

  const handleSSOLogin = () => {
    window.location.href = `${SSO_LOGIN_URL}?redirect_uri=${encodeURIComponent(CALLBACK_URL)}&app=app-creator`;
  };

  const handleStandaloneLogin = async () => {
    if (!username.trim()) return;
    setIsLoggingIn(true);
    setError('');
    try {
      const res = await fetch('/api/creator/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: username.trim() }),
      });
      if (!res.ok) {
        const errData: { message?: string } = await res.json().catch(() => ({}));
        throw new Error(errData.message || 'Login failed');
      }
      const data: { token?: string; refresh_token?: string } = await res.json();
      if (data.refresh_token) setRefreshToken(data.refresh_token);
      if (data.token) login(data.token);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : '登录失败');
      setIsLoggingIn(false);
    }
  };

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1 className="auth-title">AppCreator</h1>
        <p className="muted-text" style={{ marginBottom: 32 }}>
          {authMode === 'loading'
            ? '加载中...'
            : authMode === 'standalone'
              ? '输入用户名开始使用'
              : isLoading
                ? '验证登录状态...'
                : '登录以创建你的应用'}
        </p>

        {error && (
          <p className="form-error" style={{ marginBottom: 16 }}>
            {error}
          </p>
        )}

        {authMode === 'loading' ? (
          <div className="spinner" />
        ) : authMode === 'standalone' ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <input
              type="text"
              placeholder="用户名"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleStandaloneLogin()}
              className="auth-input"
              autoFocus
            />
            <button
              className="btn btn-primary"
              onClick={handleStandaloneLogin}
              disabled={isLoggingIn || !username.trim()}
              style={{ width: '100%', justifyContent: 'center' }}
            >
              {isLoggingIn ? '登录中...' : '登录'}
            </button>
          </div>
        ) : (
          <button
            className="btn btn-primary"
            onClick={handleSSOLogin}
            style={{ width: '100%', justifyContent: 'center' }}
          >
            通过 SSO 登录
          </button>
        )}
      </div>
    </div>
  );
}
