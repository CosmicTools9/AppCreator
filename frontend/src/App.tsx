import { Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider } from "./stores/provider";
import { LandingPage } from "./pages/LandingPage";
import { LoginPage } from "./pages/LoginPage";
import { WorkspacePage } from "./pages/WorkspacePage";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { useAuth } from "./stores/auth";

/** Redirect authenticated users from / to /workspace. */
function IndexRedirect() {
  const { isAuthenticated } = useAuth();
  if (isAuthenticated) return <Navigate to="/workspace" replace />;
  return <LandingPage />;
}

function App() {
  return (
    <AuthProvider>
      <Routes>
        {/* Public: login page (no AppLayout) */}
        <Route path="/login" element={<LoginPage />} />

        {/* Public: landing page (standalone, owns its own .landing-nav header) */}
        <Route path="/" element={<IndexRedirect />} />

        {/* Protected: workspace requires auth */}
        <Route element={<ProtectedRoute />}>
          <Route path="/workspace" element={<WorkspacePage />} />
        </Route>

        {/* Catch-all */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AuthProvider>
  );
}

export default App;
