import { useState, useRef, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useAtom } from "jotai";
import { useAuth } from "../stores/auth";
import {
  currentSessionIdAtom,
  messagesAtom,
  isGeneratingAtom,
  prototypeUrlAtom,
  type ChatMessage,
} from "../stores/chat";
import { api, ApiError, type ChatSession } from "../api/client";

export function WorkspacePage() {
  const navigate = useNavigate();
  const { token, logout, user } = useAuth();
  const [sessionId, setSessionId] = useAtom(currentSessionIdAtom);
  const [messages, setMessages] = useAtom(messagesAtom);
  const [isGenerating, setIsGenerating] = useAtom(isGeneratingAtom);
  const [prototypeUrl, setPrototypeUrl] = useAtom(prototypeUrlAtom);
  const [input, setInput] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const chatEnd = useRef<HTMLDivElement>(null);

  const opts = { token };

  const refreshSessions = useCallback(async () => {
    try {
      const res = await api.listSessions(null, opts);
      setSessions(res.sessions ?? []);
    } catch {
      // 列表失败不阻塞主流程
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token]);

  useEffect(() => {
    refreshSessions();
  }, [refreshSessions]);

  useEffect(() => {
    chatEnd.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const newSession = () => {
    setSessionId(null);
    setMessages([]);
    setPrototypeUrl(null);
    setInput("");
    setError(null);
  };

  const loadSession = async (id: number) => {
    try {
      const session = await api.getSession(id, opts);
      setSessionId(id);
      setMessages(session.messages ?? []);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "加载会话失败");
    }
  };

  const sendMessage = async () => {
    if (!input.trim() || isGenerating) return;
    const text = input.trim();
    setInput("");
    setError(null);

    let id = sessionId;

    try {
      if (!id) {
        const session = await api.createSession(
          {
            title: text.slice(0, 80),
            namespace: "",
          },
          opts
        );
        id = session.id;
        setSessionId(id);
      }

      await api.addMessage(id, { content: text, role: "user" }, opts);
      setMessages((prev) => [
        ...prev,
        {
          id: Date.now(),
          session_id: id!,
          role: "user",
          content: text,
          created_at: new Date().toISOString(),
        },
      ]);
      setIsGenerating(true);

      const step = await api.generateResponse(id, opts);

      const assistantText = step.message?.trim() || (step.is_terminal ? "✅ 已完成" : "...");
      const assistantMsg: ChatMessage = {
        id: Date.now() + 1,
        session_id: id!,
        role: "assistant",
        content: assistantText,
        created_at: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, assistantMsg]);

      if (step.is_terminal) {
        setPrototypeUrl(String(id));
      }

      // Sync with persisted messages (including any extra system messages from the agent)
      await loadSession(id);
      refreshSessions();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "生成失败，请重试");
      setIsGenerating(false);
      return;
    }

    setIsGenerating(false);
  };

  return (
    <div className="workspace-layout">
      {/* Sidebar */}
      <aside className="workspace-sidebar">
        <div className="workspace-sidebar-header">
          <div className="workspace-logo">AC</div>
          <span style={{ fontSize: 13, fontWeight: 600 }}>AppCreator</span>
        </div>
        <button className="workspace-new-btn" onClick={newSession}>+ 新建会话</button>
        <div className="workspace-sidebar-list">
          {sessions.map((s) => (
            <div
              key={s.id}
              className={`workspace-session-item${s.id === sessionId ? " active" : ""}`}
              style={{ cursor: "pointer" }}
              onClick={() => loadSession(s.id)}
            >
              {s.title || `会话 #${s.id}`}
            </div>
          ))}
          {sessions.length === 0 && (
            <p className="muted-text" style={{ padding: 16, fontSize: 12, textAlign: "center" }}>
              开始新对话
            </p>
          )}
        </div>
        {user && (
          <div className="workspace-sidebar-footer">
            <span style={{ fontSize: 11, color: "var(--text-secondary)", display: "block", marginBottom: 8, padding: "0 4px" }}>
              {user.username}
            </span>
            <button className="workspace-logout-btn" onClick={() => { logout(); navigate("/"); }}>
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
              <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.3">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
            </div>
            <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 4 }}>开始创建你的应用</h3>
            <p className="muted-text" style={{ fontSize: 14 }}>
              在下方输入你的需求，AppCreator 将为你生成企业应用原型
            </p>
          </div>
        ) : (
          <>
            <div className="workspace-messages">
              {messages.map((m) => (
                <div key={m.id} className={`workspace-msg ${m.role}`}>
                  <div className={`workspace-msg-avatar ${m.role}`}>
                    {m.role === "assistant" ? "AI" : "U"}
                  </div>
                  <div className="workspace-msg-bubble">{m.content}</div>
                </div>
              ))}
              {isGenerating && (
                <div className="workspace-msg assistant">
                  <div className="workspace-msg-avatar assistant">AI</div>
                  <div className="workspace-msg-bubble sending">
                    <span className="dot" /><span className="dot" /><span className="dot" />
                  </div>
                </div>
              )}
              <div ref={chatEnd} />
            </div>

            {prototypeUrl && (
              <div className="workspace-prototype-banner">
                <span>✅ 原型已就绪</span>
                <button className="btn btn-primary" style={{ padding: "6px 16px", fontSize: 13 }}
                  onClick={async () => {
                    try {
                      const html = await api.fetchPrototype(Number(prototypeUrl), opts);
                      const blob = new Blob([html], { type: "text/html" });
                      window.open(URL.createObjectURL(blob), "_blank");
                    } catch (e) {
                      setError(e instanceof ApiError ? e.message : "加载原型失败");
                    }
                  }}>
                  预览应用
                </button>
              </div>
            )}

            {error && (
              <div className="workspace-error" style={{ padding: "8px 16px", color: "var(--error)", fontSize: 13 }}>
                {error}
              </div>
            )}

            <div className="workspace-input-bar">
              <input className="workspace-input" type="text" value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && sendMessage()}
                placeholder="描述你的应用需求..." disabled={isGenerating} />
              <button className="workspace-send-btn" onClick={sendMessage}
                disabled={isGenerating || !input.trim()}>
                {isGenerating ? "生成中..." : "发送"}
              </button>
            </div>
          </>
        )}
      </main>
    </div>
  );
}
