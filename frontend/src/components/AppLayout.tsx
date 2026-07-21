import { Outlet } from "react-router-dom";
import { StatusBadge } from "./StatusBadge";

export function AppLayout() {
  return (
    <div className="app-creator">
      <header className="app-header">
        <h1>AppCreator</h1>
        <StatusBadge />
      </header>
      <main className="app-main">
        <Outlet />
      </main>
    </div>
  );
}
