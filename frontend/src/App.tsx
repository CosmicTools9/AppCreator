import { Routes, Route, Navigate } from "react-router-dom";
import { useEffect, useState } from "react";

function App() {
  const [backendStatus, setBackendStatus] = useState<string>("checking...");

  useEffect(() => {
    fetch("/api/creator/status")
      .then((r) => r.json())
      .then((data) => setBackendStatus(data.status ?? "error"))
      .catch(() => setBackendStatus("unreachable"));
  }, []);

  return (
    <div className="app-creator">
      <header className="app-header">
        <h1>AppCreator</h1>
        <span className="status-badge" data-status={backendStatus}>
          {backendStatus}
        </span>
      </header>
      <main className="app-main">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
    </div>
  );
}

function HomePage() {
  return (
    <div className="welcome">
      <h2>Welcome to AppCreator</h2>
      <p>可视化创建、配置和引导新应用与模块。</p>
      <div className="cards">
        <div className="card">
          <h3>创建新应用</h3>
          <p>从模板快速搭建全新应用</p>
        </div>
        <div className="card">
          <h3>管理模块</h3>
          <p>配置已有模块的依赖与扩展</p>
        </div>
        <div className="card">
          <h3>一键部署</h3>
          <p>将应用发布到目标环境</p>
        </div>
      </div>
    </div>
  );
}

export default App;
