import { useNavigate } from "react-router-dom";

const STEP_CARDS = [
  {
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
      </svg>
    ),
    title: "描述需求",
    desc: "用自然语言描述业务场景，AppCreator 理解意图并拆解为功能模块。",
  },
  {
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <rect x="3" y="3" width="18" height="18" rx="2" />
        <path d="M3 9h18M9 21V9" />
      </svg>
    ),
    title: "生成原型",
    desc: "AI 实时生成高保真 HTML 原型，可预览、可下载、可持续对话迭代。",
  },
  {
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M12 2L2 7l10 5 10-5-10-5z" />
        <path d="M2 17l10 5 10-5" />
        <path d="M2 12l10 5 10-5" />
      </svg>
    ),
    title: "部署上线",
    desc: "原型确认后，生成完整源码包（Rust + React + PostgreSQL）开箱即用。",
  },
];

export function LandingPage() {
  const navigate = useNavigate();

  return (
    <div className="welcome">
      <p className="section-label">AI 驱动的企业应用生成器</p>
      <h2>
        对话创建企业应用
        <br />
        <span className="accent">从需求到部署</span>
      </h2>
      <p>
        用自然语言描述你的管理需求——AppCreator 即时生成生产级企业应用原型。
        <br />
        管理后台、审批流、ERP 模块、数据看板，对话即可完成。
      </p>
      <div className="hero-actions">
        <button className="btn btn-primary" onClick={() => navigate("/workspace")}>
          免费开始
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
        </button>
        <a
          href={import.meta.env.DEV ? "/landing-demo" : "#"}
          className="btn btn-secondary"
          onClick={(e) => {
            if (!import.meta.env.DEV) e.preventDefault();
          }}
        >
          预览演示
        </a>
      </div>
      <div className="cards">
        {STEP_CARDS.map((card) => (
          <div className="card" key={card.title} onClick={() => navigate("/workspace")}>
            <div className="card-icon">{card.icon}</div>
            <h3>{card.title}</h3>
            <p>{card.desc}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
