import { Navigate, Outlet, useLocation } from "react-router-dom";
import { useAuth } from "../stores/auth";

/**
 * Route guard — requires valid SSO JWT.
 * Redirects to root if not authenticated.
 */
export function ProtectedRoute() {
  const { isAuthenticated } = useAuth();
  const location = useLocation();

  if (!isAuthenticated) {
    // Preserve intended URL for post-login redirect
    return <Navigate to="/" state={{ from: location }} replace />;
  }
  return <Outlet />;
}
