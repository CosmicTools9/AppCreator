import { useNavigate } from "react-router-dom";

const STEP_CARDS = [
  {
    icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>,
    title: "描述需求", desc: "用自然语言描述业务场景，AppCreator 理解意图并拆解为功能模块。",
  },
  {
    icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 21V9"/></svg>,
    title: "生成原型", desc: "AI 实时生成高保真 HTML 原型，可预览、可下载、可持续对话迭代。",
  },
  {
    icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>,
    title: "部署上线", desc: "原型确认后，生成完整源码包（Rust + React + PostgreSQL）开箱即用。",
  },
];

const CAPABILITIES = [
  { icon: "chat", title: "对话式需求拆解", desc: "AI 引导你逐步细化需求，从模糊描述到精确功能规格，自动产出需求文档与原型框架。" },
  { icon: "layout", title: "高保真原型即时生成", desc: "单文件 HTML 应用原型，可交互、可预览、可下载。支持无限次迭代，每次对话即更新原型。" },
  { icon: "layers", title: "企业级技术栈输出", desc: "生成 Rust + React + PostgreSQL 的完整工程代码，遵循 Alioth 四层隔离模型，生产级架构开箱即用。" },
  { icon: "clock", title: "一键构建与部署", desc: "自动生成 Docker 镜像 + docker-compose 编排 + 版本锁清单，从代码到生产环境上线仅需一条命令。" },
  { icon: "grid", title: "模块化扩展体系", desc: "生成的每个应用基于 Alioth 模块化架构，支持后期按需扩展、集成 Gateway 统一入口与 SSO 认证。" },
  { icon: "shield", title: "企业级安全合规", desc: "继承 Alioth 的 NGAC 权限模型、SSO JWT 认证、四层环境隔离。生成的代码天然符合企业安全审计要求。" },
];

const TEMPLATES = [
  { name: "管理后台", desc: "用户管理、权限配置、操作日志、数据仪表盘" },
  { name: "审批流程", desc: "报销审批、请假流程、合同审核、任务流转" },
  { name: "ERP 模块", desc: "采购管理、库存跟踪、订单处理、供应商门户" },
  { name: "数据看板", desc: "销售报表、运营指标、实时监控、趋势分析" },
];

function CapIcon({ name }: { name: string }) {
  const paths: Record<string, string> = {
    chat: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z",
    layout: "M3 9h18M9 21V9 M3 3h18v18H3z",
    layers: "M12 2L2 7l10 5 10-5-10-5z M2 17l10 5 10-5 M2 12l10 5 10-5",
    clock: "M12 6v6l4 2 M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
    grid: "M9 12h6 M12 9v6 M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
    shield: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z",
  };
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d={paths[name] || paths.chat} />
    </svg>
  );
}

function TemplateIcon({ i }: { i: number }) {
  const icons = [
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18 M9 21V9"/></svg>,
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"/><rect x="9" y="3" width="6" height="4" rx="1"/><path d="M9 14l2 2 4-4"/></svg>,
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="12" cy="12" r="10"/><path d="M16 8h-6a2 2 0 1 0 0 4h4a2 2 0 1 1 0 4H8"/><path d="M12 18V6"/></svg>,
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><rect x="4" y="4" width="16" height="16" rx="2"/><path d="M4 10h16 M10 4v16"/></svg>,
  ];
  return <>{icons[i]}</>;
}

export function LandingPage() {
  const navigate = useNavigate();

  return (
    <div className="welcome">
      {/* Hero */}
      <p className="section-label">AI 驱动的企业应用生成器</p>
      <h2>对话创建企业应用<br /><span className="accent">从需求到部署</span></h2>
      <p>用自然语言描述你的管理需求——AppCreator 即时生成生产级企业应用原型。<br />管理后台、审批流、ERP 模块、数据看板，对话即可完成。</p>
      <div className="hero-actions">
        <button className="btn btn-primary" onClick={() => navigate("/workspace")}>
          免费开始<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
        </button>
        <a href="#" className="btn btn-secondary" onClick={(e) => e.preventDefault()}>预览演示</a>
      </div>

      {/* Steps */}
      <div className="cards" style={{ marginBottom: 80 }}>
        {STEP_CARDS.map((card) => (
          <div className="card" key={card.title} onClick={() => navigate("/workspace")}>
            <div className="card-icon">{card.icon}</div>
            <h3>{card.title}</h3>
            <p>{card.desc}</p>
          </div>
        ))}
      </div>

      {/* Capabilities */}
      <div className="section-label">—— 核心能力</div>
      <h2 className="text-h1" style={{ marginBottom: 48 }}>从对话到生产，一站式覆盖</h2>
      <div className="caps-grid" style={{ textAlign: "left", maxWidth: 960, margin: "0 auto 80px" }}>
        {CAPABILITIES.map((cap) => (
          <div className="cap-card" key={cap.title}>
            <div className="card-icon"><CapIcon name={cap.icon} /></div>
            <div style={{ flex: 1 }}>
              <h3>{cap.title}</h3>
              <p className="muted-text" style={{ fontSize: 13, lineHeight: 1.6 }}>{cap.desc}</p>
            </div>
          </div>
        ))}
      </div>

      {/* Templates */}
      <div className="section-label">—— 适用场景</div>
      <h2 className="text-h1" style={{ marginBottom: 48 }}>企业应用的每一个角落</h2>
      <div className="templates-grid" style={{ maxWidth: 640, margin: "0 auto 80px" }}>
        {TEMPLATES.map((t, i) => (
          <div className="template-card" key={t.name}>
            <div className="template-icon"><TemplateIcon i={i} /></div>
            <h3>{t.name}</h3>
            <p className="muted-text" style={{ fontSize: 12 }}>{t.desc}</p>
          </div>
        ))}
      </div>

      {/* Pricing */}
      <div className="section-label">—— 定价</div>
      <h2 className="text-h1" style={{ marginBottom: 48 }}>选择适合你的方案</h2>
      <div className="pricing-grid" style={{ maxWidth: 960 }}>
        {/* 免费预览 */}
        <div className="pricing-card">
          <h3 className="pricing-tier">免费预览</h3>
          <p className="pricing-price"><span className="pricing-amount">¥0</span></p>
          <ul className="pricing-features">
            <li>AI 对话创建应用原型</li>
            <li>原型预览与下载</li>
            <li>无限次迭代修改</li>
            <li>社区支持</li>
          </ul>
          <button className="btn btn-secondary" onClick={() => navigate("/workspace")} style={{ width: "100%", justifyContent: "center" }}>免费开始</button>
        </div>
        {/* 专业订阅 */}
        <div className="pricing-card featured">
          <div className="pricing-badge">推荐</div>
          <h3 className="pricing-tier">专业订阅</h3>
          <p className="pricing-price"><span className="pricing-amount">¥1,399</span><span className="pricing-period">/月</span></p>
          <ul className="pricing-features">
            <li>每月 60 个产品原型额度</li>
            <li>每个原型无限次对话迭代</li>
            <li>首次源码包下载 <strong>¥4,999</strong></li>
            <li>重新生成不计费</li>
            <li>再次下载仅 <strong>¥19.9/次</strong></li>
            <li>优先技术支持</li>
          </ul>
          <button className="btn btn-primary" onClick={() => navigate("/workspace")} style={{ width: "100%", justifyContent: "center" }}>立即订阅</button>
        </div>
        {/* AliothStudio = 企业定制 */}
        <div className="pricing-card">
          <h3 className="pricing-tier">AliothStudio</h3>
          <p className="pricing-price" style={{ fontSize: 14, color: "var(--text-secondary)", marginBottom: 4 }}>企业定制方案</p>
          <p className="pricing-price"><span className="pricing-amount">¥499,999</span></p>
          <ul className="pricing-features">
            <li>基于元数据的自定义扩展</li>
            <li>从 `isahl_meta` 自定义实体与字段</li>
            <li>私域部署（Docker 单机或集群）</li>
            <li>独立 SSO 认证与 NGAC 权限</li>
            <li>代码包无限下载</li>
            <li>优先技术支持</li>
          </ul>
          <button className="btn btn-secondary" onClick={() => {}} style={{ width: "100%", justifyContent: "center" }}>联系销售</button>
        </div>
      </div>
    </div>
  );
}
