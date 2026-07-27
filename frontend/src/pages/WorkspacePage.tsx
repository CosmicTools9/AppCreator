import { useState, useRef, useEffect, useCallback } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAtom } from 'jotai';
import { useAuth } from '../stores/auth';
import {
  currentSessionIdAtom,
  messagesAtom,
  isGeneratingAtom,
  prototypeUrlAtom,
  type ChatMessage,
} from '../stores/chat';
import { api, ApiError, type ChatSession } from '../api/client';

const TEMPLATES = [
  {
    name: '客户管理后台',
    prompt: '请创建一个客户管理后台，包含客户列表、客户详情、客户表单等功能',
    desc: '列表、详情、表单',
  },
  {
    name: '审批流程',
    prompt: '请创建一个审批流程应用，支持报销审批、请假审批、合同审批等场景',
    desc: '报销、请假、合同',
  },
  {
    name: 'ERP 模块',
    prompt: '请创建一个 ERP 管理模块，包含采购、库存、订单等核心业务功能',
    desc: '采购、库存、订单',
  },
  {
    name: '数据看板',
    prompt: '请创建一个数据看板应用，包含销售报表和运营指标的实时展示',
    desc: '销售报表、运营指标',
  },
];
const TEMPLATE_ICONS = [
  { bg: 'linear-gradient(135deg, #2563EB, #1D4ED8)', shape: 'rect' }, // 管理后台 — 方形
  { bg: 'linear-gradient(135deg, #059669, #047857)', shape: 'circle' }, // 审批 — 圆形
  { bg: 'linear-gradient(135deg, #D97706, #B45309)', shape: 'tri' }, // ERP — 三角
  { bg: 'linear-gradient(135deg, #7C3AED, #6D28D9)', shape: 'bar' }, // 数据看板 — 条形
];

export function WorkspacePage() {
  const navigate = useNavigate();
  const location = useLocation();
  const { token, logout, user } = useAuth();
  const [sessionId, setSessionId] = useAtom(currentSessionIdAtom);
  const [messages, setMessages] = useAtom(messagesAtom);
  const [isGenerating, setIsGenerating] = useAtom(isGeneratingAtom);
  const [prototypeUrl, setPrototypeUrl] = useAtom(prototypeUrlAtom);
  const [input, setInput] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [progress, setProgress] = useState<{ state: string; percent: number } | null>(null);
  const chatEnd = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const interruptedRef = useRef(false);
  const tokenRef = useRef(token);
  tokenRef.current = token;
  const optsRef = useRef({ token: tokenRef.current });
  optsRef.current = { token: tokenRef.current };
  const refreshSessions = useCallback(async () => {
    try {
      const res = await api.listSessions(null, optsRef.current);
      setSessions(res.sessions ?? []);
    } catch {
      // list failure is non-blocking
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token]);

  useEffect(() => {
    refreshSessions();
  }, [refreshSessions]);
  useEffect(() => {
    chatEnd.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Mount: load session from Landing navigation or existing atom; auto-trigger if app_creating
  useEffect(() => {
    const state = location.state as { sessionId?: number } | null;
    const sid = state?.sessionId ?? sessionId;
    if (sid && messages.length === 0) {
      loadSession(sid).then((session) => {
        if (
          session &&
          session.status === 'app_creating' &&
          !session.messages.some((m) => m.role === 'assistant')
        ) {
          runGenerateLoop(sid);
        }
      });
    }
    // Clear location state to prevent re-trigger on re-render
    if (state?.sessionId) window.history.replaceState({}, document.title);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const newSession = () => {
    setSessionId(null);
    setMessages([]);
    setPrototypeUrl(null);
    setInput('');
    setError(null);
    setProgress(null);
    // Keep sessions list intact — user can still return to old sessions
  };

  const loadSession = async (id: number): Promise<ChatSession | null> => {
    try {
      const session = await api.getSession(id, optsRef.current);
      setSessionId(id);
      setMessages(session.messages ?? []);
      return session;
    } catch (e) {
      setError(e instanceof ApiError ? e.message : '加载会话失败');
      return null;
    }
  };

  const runGenerateLoop = async (id: number) => {
    setIsGenerating(true);
    setError(null);
    setProgress({ state: '准备中', percent: 0 });
    interruptedRef.current = false;
    let terminal = false;

    for (let i = 0; i < 30; i++) {
      if (interruptedRef.current) break;
      try {
        const step = await api.generateResponse(id, optsRef.current);
        setProgress({ state: step.state_after, percent: step.progress_percent });
        terminal = step.is_terminal;
        if (terminal) break;
      } catch (e) {
        setError(e instanceof ApiError ? e.message : '生成失败');
        setProgress(null);
        break;
      }
    }

    // Sync persisted messages as single truth source
    const session = await loadSession(id);
    refreshSessions();
    setIsGenerating(false);
    setProgress(null);

    if (terminal && session?.status === 'completed') {
      setPrototypeUrl(String(id));
    }
  };

  const handleInterrupt = async () => {
    interruptedRef.current = true;
    if (sessionId) {
      try {
        await api.interrupt(sessionId, optsRef.current);
      } catch {
        // best-effort
      }
    }
  };

  const sendMessage = async () => {
    if (!input.trim() || isGenerating) return;
    const text = input.trim();
    setInput('');
    setError(null);
    setProgress(null);

    let id = sessionId;

    try {
      if (!id) {
        const session = await api.createSession(
          { title: text.slice(0, 80), namespace: '' },
          optsRef.current,
        );
        id = session.id;
        setSessionId(id);
      }

      await api.addMessage(id, { content: text, role: 'user' }, optsRef.current);
      // Optimistic user message
      setMessages((prev) => [
        ...prev,
        {
          id: Date.now(),
          session_id: id!,
          role: 'user',
          content: text,
          created_at: new Date().toISOString(),
        },
      ]);

      await runGenerateLoop(id);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : '生成失败，请重试');
      setIsGenerating(false);
    }
  };

  const handleTemplate = (prompt: string) => {
    setInput(prompt);
    inputRef.current?.focus();
  };

  return (
    <div className="workspace-layout">
      {/* Sidebar */}
      <aside className="workspace-sidebar">
        <div className="workspace-sidebar-header">
          <div className="workspace-logo">AC</div>
          <span style={{ fontSize: 13, fontWeight: 600 }}>AppCreator</span>
        </div>
        <button className="workspace-new-btn" onClick={newSession}>
          + 新建会话
        </button>
        <div className="workspace-sidebar-list">
          {sessions.map((s) => (
            <div
              key={s.id}
              className={`workspace-session-item${s.id === sessionId ? ' active' : ''}`}
              style={{ cursor: 'pointer' }}
              onClick={() => loadSession(s.id)}
            >
              {s.title || `会话 #${s.id}`}
            </div>
          ))}
          {sessions.length === 0 && (
            <p className="muted-text" style={{ padding: 16, fontSize: 12, textAlign: 'center' }}>
              开始新对话
            </p>
          )}
        </div>
        {user && (
          <div className="workspace-sidebar-footer">
            <span
              style={{
                fontSize: 11,
                color: 'var(--text-secondary)',
                display: 'block',
                marginBottom: 8,
                padding: '0 4px',
              }}
            >
              {user.username}
            </span>
            <button
              className="workspace-logout-btn"
              onClick={() => {
                logout();
                navigate('/');
              }}
            >
              退出登录
            </button>
          </div>
        )}
      </aside>

      {/* Main chat */}
      <main className="workspace-chat">
        {messages.length === 0 ? (
          <div className="workspace-empty">
            <div className="workspace-empty-icon">
              <svg
                width="40"
                height="40"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                opacity="0.3"
              >
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
              </svg>
            </div>
            <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 4 }}>选择场景，开始创建</h3>
            <p className="muted-text" style={{ fontSize: 14 }}>
              选择一个模板或直接输入你的需求
            </p>
            <div className="workspace-templates-quick" style={{ marginTop: 24 }}>
              {TEMPLATES.map((t, i) => (
                <button
                  key={t.name}
                  className="template-quick-btn"
                  onClick={() => handleTemplate(t.prompt)}
                >
                  <div
                    className="tpl-icon"
                    style={{
                      width: 32,
                      height: 32,
                      borderRadius: i === 1 ? '50%' : 8,
                      background: TEMPLATE_ICONS[i]?.bg || 'var(--accent)',
                      marginBottom: 4,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                    }}
                  >
                    <svg
                      width="16"
                      height="16"
                      viewBox="0 0 16 16"
                      fill="none"
                      stroke="#fff"
                      strokeWidth="1.5"
                    >
                      {i === 0 && <rect x="2" y="2" width="12" height="12" rx="2" />}
                      {i === 1 && <circle cx="8" cy="8" r="6" />}
                      {i === 2 && <polygon points="8,2 14,13 2,13" />}
                      {i === 3 && (
                        <>
                          <rect x="2" y="9" width="3" height="5" />
                          <rect x="6.5" y="5" width="3" height="9" />
                          <rect x="11" y="2" width="3" height="12" />
                        </>
                      )}
                    </svg>
                  </div>
                  <strong>{t.name}</strong>
                  <span>{t.desc}</span>
                </button>
              ))}
            </div>
          </div>
        ) : (
          <div className="workspace-messages">
            {messages.map((m) => (
              <div key={m.id} className={`workspace-msg ${m.role}`}>
                <div className={`workspace-msg-avatar ${m.role}`}>
                  {m.role === 'assistant' ? 'AI' : 'U'}
                </div>
                <div className="workspace-msg-bubble">{m.content}</div>
              </div>
            ))}
            {isGenerating && (
              <div className="workspace-msg assistant">
                <div className="workspace-msg-avatar assistant">AI</div>
                <div className="workspace-msg-bubble sending">
                  <span className="dot" />
                  <span className="dot" />
                  <span className="dot" />
                </div>
              </div>
            )}
            <div ref={chatEnd} />
          </div>
        )}

        {/* Progress card (visible during generation regardless of messages) */}
        {isGenerating && progress && (
          <div className="workspace-progress-bar">
            <div className="workspace-progress-info">
              <span>
                <strong>{progress.state}</strong> · {progress.percent}%
              </span>
              <button className="workspace-stop-btn" onClick={handleInterrupt}>
                停止
              </button>
            </div>
            <div className="workspace-progress-track">
              <div className="workspace-progress-fill" style={{ width: `${progress.percent}%` }} />
            </div>
          </div>
        )}

        {/* Prototype banner */}
        {prototypeUrl && (
          <div className="workspace-prototype-banner">
            <span>✅ 原型已就绪</span>
            <button
              className="btn btn-primary"
              style={{ padding: '6px 16px', fontSize: 13 }}
              onClick={async () => {
                try {
                  const html = await api.fetchPrototype(Number(prototypeUrl), optsRef.current);
                  const blob = new Blob([html], { type: 'text/html' });
                  window.open(URL.createObjectURL(blob), '_blank');
                } catch (e) {
                  setError(e instanceof ApiError ? e.message : '加载原型失败');
                }
              }}
            >
              预览应用
            </button>
          </div>
        )}

        {/* Error banner (always visible) */}
        {error && (
          <div
            className="workspace-error"
            style={{ padding: '8px 16px', color: 'var(--error)', fontSize: 13 }}
          >
            {error}
          </div>
        )}

        {/* Input bar (always visible per design workspace-v1.html) */}
        <div className="workspace-input-bar">
          <input
            className="workspace-input"
            type="text"
            value={input}
            ref={inputRef}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && sendMessage()}
            placeholder="描述你的应用需求..."
            disabled={isGenerating}
          />
          <button
            className="workspace-send-btn"
            onClick={sendMessage}
            disabled={isGenerating || !input.trim()}
          >
            {isGenerating ? '生成中...' : '发送'}
          </button>
        </div>
      </main>
    </div>
  );
}
