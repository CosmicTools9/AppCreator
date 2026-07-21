import { useNavigate } from "react-router-dom";
import { useT } from "../locales";

export function WorkspacePage() {
  const navigate = useNavigate();
  const { t } = useT();

  return (
    <div className="welcome">
      <p className="section-label">{t("workspace.title")}</p>
      <h2>{t("workspace.title")}</h2>
      <p className="muted-text" style={{ marginBottom: 40, maxWidth: 520, marginInline: "auto" }}>
        {t("workspace.subtitle")}
      </p>
      <div className="cards" style={{ maxWidth: 520 }}>
        <div className="card" onClick={() => navigate("/")}>
          <h3>{t("workspace.newProject")}</h3>
          <p className="muted-text">{t("landing.badge")}</p>
        </div>
      </div>
      <div style={{ marginTop: 32 }}>
        <button className="btn btn-secondary" onClick={() => navigate("/")}>
          {t("workspace.back")}
        </button>
      </div>
    </div>
  );
}
