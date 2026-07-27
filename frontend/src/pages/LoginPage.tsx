import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import { useAuth } from '../stores/auth';
import { setRefreshToken } from '../api/client';

export function LoginPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { login, isAuthenticated, isLoading } = useAuth();
  const [error, setError] = useState('');
  const [username, setUsername] = useState('');
  const [authMode, setAuthMode] = useState<'sso' | 'standalone' | 'loading'>('loading');
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  // SSO form state
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isRegistering, setIsRegistering] = useState(false);

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

  const handleSSOLogin = async () => {
    if (!email.trim() || !password.trim()) return;
    setIsLoggingIn(true);
    setError('');
    try {
      const res = await fetch('/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ identifier: email.trim(), password }),
      });
      if (!res.ok) {
        const errData: { error?: string } = await res.json().catch(() => ({}));
        throw new Error(errData.error || '登录失败，请检查邮箱和密码');
      }
      const data: { access_token?: string; refresh_token?: string } = await res.json();
      if (data.refresh_token) setRefreshToken(data.refresh_token);
      if (data.access_token) login(data.access_token);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : '登录失败');
      setIsLoggingIn(false);
    }
  };

  const handleSSORegister = async () => {
    if (!email.trim() || !password.trim()) return;
    setIsLoggingIn(true);
    setError('');
    try {
      const res = await fetch('/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: email.trim(), password, username: email.split('@')[0] }),
      });
      if (!res.ok) {
        const errData: { error?: string } = await res.json().catch(() => ({}));
        throw new Error(errData.error || '注册失败');
      }
      // Auto-login after successful registration
      await handleSSOLogin();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : '注册失败');
      setIsLoggingIn(false);
    }
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

  const handleOAuth = async (provider: string) => {
    setError('');
    try {
      const res = await fetch('/auth/oauth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider,
          redirect_url: window.location.origin + '/login',
        }),
      });
      if (!res.ok) {
        const errData: { message?: string } = await res.json().catch(() => ({}));
        throw new Error(errData.message || 'OAuth 登录失败');
      }
      const data: { auth_url?: string } = await res.json();
      if (data.auth_url) {
        window.location.href = data.auth_url;
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'OAuth 登录失败');
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
              : isRegistering
                ? '创建你的账号'
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
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {isRegistering ? (
              <>
                <input
                  type="email"
                  placeholder="邮箱"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSSORegister()}
                  className="auth-input"
                  autoFocus
                />
                <input
                  type="password"
                  placeholder="密码"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSSORegister()}
                  className="auth-input"
                />
                <button
                  className="btn btn-primary"
                  onClick={handleSSORegister}
                  disabled={isLoggingIn || !email.trim() || !password.trim()}
                  style={{ width: '100%', justifyContent: 'center' }}
                >
                  {isLoggingIn ? '注册中...' : '注册'}
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={() => { setError(''); setIsRegistering(false); }}
                  style={{ width: '100%', justifyContent: 'center' }}
                >
                  已有账号？登录
                </button>
              </>
            ) : (
              <>
                <input
                  type="email"
                  placeholder="邮箱"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSSOLogin()}
                  className="auth-input"
                  autoFocus
                />
                <input
                  type="password"
                  placeholder="密码"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSSOLogin()}
                  className="auth-input"
                />
                <button
                  className="btn btn-primary"
                  onClick={handleSSOLogin}
                  disabled={isLoggingIn || !email.trim() || !password.trim()}
                  style={{ width: '100%', justifyContent: 'center' }}
                >
                  {isLoggingIn ? '登录中...' : '登录'}
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={() => { setError(''); setIsRegistering(true); }}
                  style={{ width: '100%', justifyContent: 'center' }}
                >
                  没有账号？注册
                </button>
              </>
            )}
            <div className="oauth-divider"><span>或</span></div>
            <div className="oauth-buttons">
              <button className="btn btn-oauth" onClick={() => handleOAuth('github')}>
                <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                GitHub 登录
              </button>
              <button className="btn btn-oauth" onClick={() => handleOAuth('wechat')}>
                <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
                  <path d="M8.5 2C4.36 2 1 4.98 1 8.65c0 2.04 1.05 3.86 2.68 5.05L3 16.3l3.14-1.57c.73.2 1.5.32 2.36.32.17 0 .33-.02.5-.03A6.5 6.5 0 0 1 8.5 2zm0 3a1 1 0 1 1 0 2 1 1 0 0 1 0-2zm-3 1a1 1 0 1 1 0 2 1 1 0 0 1 0-2zm7.5 1c-3.04 0-5.5 2.24-5.5 5 0 1.24.55 2.36 1.44 3.2L8 17.5l2.12-1.07A6 6 0 0 0 12 16.5c.46 0 .9-.06 1.32-.15A3 3 0 0 1 13 16c0-1.66 1.34-3 3-3s3 1.34 3 3-1.34 3-3 3c-.28 0-.55-.04-.82-.1L13 20.5l1.36-.68C15.8 20.56 17.6 21 19.5 21c3.04 0 5.5-2.24 5.5-5s-2.46-5-5.5-5zm-1.5 2.5a1 1 0 1 1 0 2 1 1 0 0 1 0-2zm6 0a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"/>
                </svg>
                微信登录
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
