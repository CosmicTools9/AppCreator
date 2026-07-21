import { Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider } from "./stores/provider";
import { AppLayout } from "./components/AppLayout";
import { LandingPage } from "./pages/LandingPage";
import { WorkspacePage } from "./pages/WorkspacePage";
import { ProtectedRoute } from "./components/ProtectedRoute";

function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/" element={<LandingPage />} />
          <Route element={<ProtectedRoute />}>
            <Route path="/workspace" element={<WorkspacePage />} />
          </Route>
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AuthProvider>
  );
}

export default App;
