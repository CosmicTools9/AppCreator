import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../stores/auth";

const SSO_LOGIN_URL = import.meta.env.VITE_SSO_LOGIN_URL || "http://localhost:9002/login";
const CALLBACK_URL = `${window.location.origin}/login`;

export function LoginPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { login, isAuthenticated, isLoading } = useAuth();
  const [error, setError] = useState("");

  // Already authenticated — bounce to workspace
  useEffect(() => {
    if (isAuthenticated) navigate("/workspace", { replace: true });
  }, [isAuthenticated]);

  // Handle SSO callback — extract JWT from URL
  useEffect(() => {
    const token = searchParams.get("token");
    if (token) {
      login(token); // auth store verifies with GET /api/creator/user/me
    }
  }, [searchParams]);

  // Watch for auth success after login attempt
  useEffect(() => {
    if (isAuthenticated) {
      const next = searchParams.get("next") || "/workspace";
      navigate(next, { replace: true });
    }
  }, [isAuthenticated]);

  const handleSSOLogin = () => {
    window.location.href = `${SSO_LOGIN_URL}?redirect_uri=${encodeURIComponent(CALLBACK_URL)}&app=app-creator`;
  };

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1 className="auth-title">AppCreator</h1>
        <p className="muted-text" style={{ marginBottom: 32 }}>
          {isLoading ? "验证登录状态..." : "登录以创建你的应用"}
        </p>

        {error && <p className="form-error" style={{ marginBottom: 16 }}>{error}</p>}

        {isLoading ? (
          <div className="spinner" />
        ) : (
          <button className="btn btn-primary" onClick={handleSSOLogin}
            style={{ width: "100%", justifyContent: "center" }}>
            通过 SSO 登录
          </button>
        )}
      </div>
    </div>
  );
}
