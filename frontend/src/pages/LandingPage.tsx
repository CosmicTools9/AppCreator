// 单一真相源：AppCreator Landing Page 的权威实现。
// 旧设计稿 design/landing-v1.html 已于 2026-07-22 归档至 design/_archive/（如需恢复：git mv 回 design/ 即可）。
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useSetAtom } from "jotai";
import { useAuth } from "../stores/auth";
import { currentSessionIdAtom } from "../stores/chat";
import { api } from "../api/client";

const STEPS = [
  {
    title: "描述需求",
    desc: "用自然语言描述业务场景，AppCreator 理解意图并拆解为功能模块。",
  },
  {
    title: "生成原型",
    desc: "AI 实时生成高保真 HTML 原型，可预览、可下载、可持续对话迭代。",
  },
  {
    title: "部署上线",
    desc: "原型确认后，生成完整源码包（Rust + React + PostgreSQL）开箱即用。",
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

const FAQS = [
  { q: "免费预览能做什么？", a: "用 AI 对话创建应用原型，预览、下载、无限次迭代全部免费。只有需要可部署的源码包时才付费。" },
  { q: "「每月 60 个产品原型额度」怎么算？", a: "每开启一个全新应用算 1 个额度；同一应用内的对话迭代不消耗额度，重新生成也不计费。" },
  { q: "源码包怎么收费？", a: "专业订阅内含首次源码包下载（¥4,999）；后续再次下载仅 ¥19.9/次；原型迭代本身免费。" },
  { q: "生成的应用用什么技术栈？", a: "Rust 后端 + React 前端 + PostgreSQL，遵循 Alioth 四层隔离架构，Docker 编排开箱即用。" },
  { q: "支持私有部署与数据安全吗？", a: "免费版与专业版为云服务；企业版（AliothStudio）支持 Docker 私域单机 / 集群部署，配套独立 SSO 与 NGAC 权限模型。" },
  { q: "没有技术团队也能上线吗？", a: "源码包附带 Docker 编排与部署文档，一条命令完成构建部署；企业版可联系我们提供上线协助。" },
];

function FaqItem({ q, a }: { q: string; a: string }) {
  const [open, setOpen] = useState(false);
  return (
    <div className={`faq-item${open ? " open" : ""}`}>
      <button className="faq-q" onClick={() => setOpen((v) => !v)} aria-expanded={open}>
        {q}
        <svg className="faq-chevron" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M6 9l6 6 6-6"/></svg>
      </button>
      {open && <div className="faq-a">{a}</div>}
    </div>
  );
}

export function LandingPage() {
  const navigate = useNavigate();
  const { token } = useAuth();
  const setSessionId = useSetAtom(currentSessionIdAtom);
  const [showDialog, setShowDialog] = useState(false);
  const [appName, setAppName] = useState("");
  const [appDesc, setAppDesc] = useState("");
  const [creating, setCreating] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!appName.trim() || !appDesc.trim()) return;
    setCreating(true);
    setCreateError(null);
    try {
      const res = await api.createApp(
        { name: appName.trim(), description: appDesc.trim() },
        { token }
      );
      setSessionId(res.session.id);
      navigate("/workspace", { state: { sessionId: res.session.id } });
    } catch (e) {
      setCreateError(e instanceof Error ? e.message : "创建失败");
    } finally {
      setCreating(false);
    }
  };

  const handleStartClick = () => setShowDialog(true);
  const scrollTo = (id: string) => (e: { preventDefault: () => void }) => {
    e.preventDefault();
    document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
  };
  const scrollToDemo = () => {
    document.getElementById("hero-demo")?.scrollIntoView({ behavior: "smooth", block: "center" });
  };

  return (
    <>
      {/* Top nav */}
      <nav className="landing-nav">
        <div className="container">
          <a href="#" className="nav-brand" onClick={(e) => e.preventDefault()}>
            <span className="nav-brand-icon">AC</span>
            AppCreator
          </a>
          <ul className="nav-links">
            <li><a href="#how" onClick={scrollTo("how")}>工作原理</a></li>
            <li><a href="#caps" onClick={scrollTo("caps")}>能力</a></li>
            <li><a href="#templates" onClick={scrollTo("templates")}>模板</a></li>
          </ul>
          <div className="nav-cta">
            <button className="btn btn-primary" onClick={handleStartClick}>开始使用</button>
          </div>
        </div>
      </nav>

      <div className="welcome">
        {/* Hero */}
        <div className="hero-badge">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          AI 驱动的企业应用生成器
        </div>
        <h2 className="hero-title">对话创建企业应用<br /><span className="accent">从需求到部署</span></h2>
        <p className="hero-sub">用自然语言描述你的管理需求——AppCreator 即时生成生产级企业应用原型。管理后台、审批流、ERP 模块、数据看板，对话即可完成。</p>
        <div className="hero-actions">
          <button className="btn btn-primary" onClick={handleStartClick}>
            免费开始<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
          </button>
          <button className="btn btn-secondary" onClick={scrollToDemo}>预览演示</button>
        </div>

        {/* Hero chat demo */}
        <div className="hero-demo" id="hero-demo">
          <div className="demo-chat">
            <div className="demo-sidebar">
              <div className="demo-sidebar-header">AppCreator</div>
              <div className="demo-session active">客户管理后台</div>
              <div className="demo-session">审批流程 v2</div>
              <div className="demo-session">库存看板</div>
            </div>
            <div className="demo-main">
              <div className="demo-msg-row user">
                <div className="demo-avatar user">U</div>
                <div className="demo-bubble">帮我创建一个客户管理后台，需要包含客户列表、详情页和新建表单</div>
              </div>
              <div className="demo-msg-row ai">
                <div className="demo-avatar ai">AI</div>
                <div className="demo-bubble">
                  <strong>好的，我来为你生成客户管理后台原型。</strong><br /><br />
                  该应用将包含以下功能模块：<br /><br />
                  <strong>📋 客户列表</strong><br />
                  · 搜索 / 筛选 / 分页表格<br />
                  · 批量操作（删除、导出）<br />
                  · 快速查看联系人<br /><br />
                  <strong>📝 客户详情页</strong><br />
                  · 基本信息展示（名称、行业、联系方式）<br />
                  · 关联订单与合同列表<br />
                  · 操作日志时间线<br /><br />
                  <strong>➕ 新建客户表单</strong><br />
                  · 字段：公司名称、行业、联系人、电话、邮箱、备注<br />
                  · 保存后自动跳转详情页<br /><br />
                  是否开始生成？
                </div>
              </div>
              <div className="demo-msg-row user">
                <div className="demo-avatar user">U</div>
                <div className="demo-bubble">开始生成</div>
              </div>
              <div className="demo-msg-row ai">
                <div className="demo-avatar ai">AI</div>
                <div className="demo-bubble success">
                  ✅ 原型已生成！<br />
                  <a href="#" onClick={(e) => e.preventDefault()}>预览客户管理后台 →</a>
                </div>
              </div>
              <div className="demo-input-bar">
                <input className="demo-input" type="text" value="开始生成" readOnly disabled />
                <button className="demo-send-btn" disabled>发送</button>
              </div>
            </div>
          </div>
        </div>

        {/* How it works */}
        <section className="how-section" id="how">
          <div className="section-label">—— 工作原理</div>
          <h2 className="text-h1">三步生成企业应用</h2>
          <div className="how-grid">
            {STEPS.map((step, i) => (
              <div className="how-step" key={step.title}>
                <div className="how-step-number">{String(i + 1).padStart(2, "0")}</div>
                <h3>{step.title}</h3>
                <p>{step.desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Capabilities */}
        <div className="section-label" id="caps">—— 核心能力</div>
        <h2 className="text-h1 section-heading">从对话到生产，一站式覆盖</h2>
        <div className="caps-grid">
          {CAPABILITIES.map((cap) => (
            <div className="cap-card" key={cap.title}>
              <div className="card-icon"><CapIcon name={cap.icon} /></div>
              <div>
                <h3>{cap.title}</h3>
                <p className="muted-text">{cap.desc}</p>
              </div>
            </div>
          ))}
        </div>

        {/* Templates */}
        <div className="section-label" id="templates">—— 适用场景</div>
        <h2 className="text-h1 section-heading">企业应用的每一个角落</h2>
        <div className="templates-grid">
          {TEMPLATES.map((t, i) => (
            <div className="template-card" key={t.name}>
              <div className="template-icon"><TemplateIcon i={i} /></div>
              <h3>{t.name}</h3>
              <p className="muted-text">{t.desc}</p>
            </div>
          ))}
        </div>

        {/* Social proof */}
        <section className="stats-section">
          <div className="stats-grid">
            <div className="stat-item">
              <div className="stat-number">10,000<span className="stat-suffix">+</span></div>
              <div className="stat-label">企业应用原型已生成</div>
            </div>
            <div className="stat-item">
              <div className="stat-number">500<span className="stat-suffix">+</span></div>
              <div className="stat-label">企业团队正在使用</div>
            </div>
            <div className="stat-item">
              <div className="stat-number">8<span className="stat-suffix"> 分钟</span></div>
              <div className="stat-label">平均产出首个可用原型</div>
            </div>
            <div className="stat-item">
              <div className="stat-number">99.9<span className="stat-suffix">%</span></div>
              <div className="stat-label">原型生成成功率</div>
            </div>
          </div>
          {/* TODO: 下方数字/行业标签/证言为占位，请由市场或运营替换为真实运营数据 */}
          <div className="logos-strip">
            <span className="logos-strip-label">已被各行业团队用于</span>
            <div className="logos-row">
              <span className="logo-pill">制造企业</span>
              <span className="logo-pill">物流供应链</span>
              <span className="logo-pill">零售连锁</span>
              <span className="logo-pill">金融服务</span>
              <span className="logo-pill">教育培训</span>
            </div>
          </div>
          <div className="testimonial">
            <p className="testimonial-quote">“我们用 AppCreator 在两天内搭出了完整的采购审批流，省下了一个月的外包开发量。”</p>
            <p className="testimonial-author">— 某制造企业 IT 负责人</p>
          </div>
        </section>

        {/* Pricing */}
        <div className="section-label">—— 定价</div>
        <h2 className="text-h1 section-heading">选择适合你的方案</h2>
        <div className="pricing-grid">
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
            <button className="btn btn-secondary btn-block" onClick={() => navigate("/workspace")}>免费开始</button>
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
            <button className="btn btn-primary btn-block" onClick={() => navigate("/workspace")}>立即订阅</button>
          </div>
          {/* AliothStudio = 企业定制 */}
          <div className="pricing-card">
            <h3 className="pricing-tier">AliothStudio</h3>
            <p className="pricing-price-sub">企业定制方案</p>
            <p className="pricing-price"><span className="pricing-amount">¥499,999</span></p>
            <ul className="pricing-features">
              <li>基于元数据的自定义扩展</li>
              <li>从元数据自定义实体与字段</li>
              <li>私域部署（Docker 单机或集群）</li>
              <li>独立 SSO 认证与 NGAC 权限</li>
              <li>代码包无限下载</li>
              <li>优先技术支持</li>
            </ul>
            <button className="btn btn-secondary btn-block" onClick={() => {}}>联系销售</button>
          </div>
        </div>

        <p className="pricing-note">
          计费说明：原型生成与迭代全程免费；源码包为一次性下载（专业版首单 ¥4,999，后续 ¥19.9/次）；企业版按定制方案报价。
        </p>

        {/* FAQ */}
        <section className="faq-section">
          <div className="section-label">—— 常见问题</div>
          <h2 className="text-h1 section-heading-sm">关于定价与部署，你可能想问</h2>
          <div className="faq-list">
            {FAQS.map((f) => (
              <FaqItem key={f.q} q={f.q} a={f.a} />
            ))}
          </div>
        </section>

        {/* CTA */}
        <section className="cta-section">
          <h2 className="text-h1">用对话开启你的第一个应用</h2>
          <p>免费预览和下载生成的原型。确认满意后获取可部署的完整源码包。</p>
          <button className="btn btn-primary" onClick={handleStartClick}>
            免费开始创建
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
          </button>
        </section>
      </div>

      {/* Footer */}
      <footer className="landing-footer">
        <div className="container">
          <a href="#" className="nav-brand" onClick={(e) => e.preventDefault()}>
            <span className="nav-brand-icon">AC</span>
            AppCreator
          </a>
          <p>Enterprise apps from conversation</p>
        </div>
      </footer>

      {/* Create App Dialog */}
      {showDialog && (
        <div className="dialog-overlay" onClick={() => setShowDialog(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <h3>创建新应用</h3>
            <p className="muted-text">描述你的业务需求，AppCreator 将即时生成企业应用原型。</p>
            <label>
              应用名称
              <input
                type="text"
                placeholder="例如：采购管理系统"
                value={appName}
                onChange={(e) => setAppName(e.target.value)}
                autoFocus
              />
            </label>
            <label>
              需求描述
              <textarea
                placeholder="例如：一个采购管理系统，包含供应商管理、采购订单、入库验收、退货处理等功能"
                value={appDesc}
                onChange={(e) => setAppDesc(e.target.value)}
                rows={4}
              />
            </label>
            {createError && <p className="error-text">{createError}</p>}
            <div className="dialog-actions">
              <button className="btn btn-secondary" onClick={() => setShowDialog(false)} disabled={creating}>
                取消
              </button>
              <button className="btn btn-primary" onClick={handleCreate} disabled={creating || !appName.trim() || !appDesc.trim()}>
                {creating ? "创建中..." : "开始创建"}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
