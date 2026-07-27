import { Navigate, Outlet, useLocation } from "react-router";
import { useAuth } from "../stores/auth";

/**
 * Route guard — requires valid SSO JWT verified against backend.
 * Shows spinner while auth is being checked.
 */
export function ProtectedRoute() {
  const { isAuthenticated, isLoading } = useAuth();
  const location = useLocation();

  if (isLoading) {
    return (
      <div className="auth-page">
        <div className="spinner" />
      </div>
    );
  }

  if (!isAuthenticated) {
    const next = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?next=${next}`} replace />;
  }

  return <Outlet />;
}
