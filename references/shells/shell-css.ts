/**
 * shell-css.ts — 原型共享壳的组件样式（ESM 化，token 化 hsl(var(--*))）。
 *
 * 从原 block/module/app-shell.html 的 <style> 块逐条迁入，作为 ESM 字符串导出，
 * 设计师可通过本文件审阅壳的视觉契约。boot-skeleton 的 fade/淡出逻辑在
 * shell-shared.ts 的 BOOT_FADE_CSS 中（三壳共用），此处仅含各壳专属的骨架布局。
 *
 * 全部值引用 prototype-base.css 的 :root 设计令牌（已弃用 tailwind-utilities.css），单令牌真相源不变。
 *
 * 规约：壳的布局/视觉一律用 Tailwind 工具类（prototype-base.css 注入 runtime；已弃用 tailwind-utilities.css），
 * 禁止任何 gl-gateway-* 类名或手写壳布局 CSS。本文件仅保留：
 *   - BLOCK_SHELL_CSS：block 内容级组件（page/card/skeleton/dialog/drawer/wizard/...）样式
 *   - MODULE_SHELL_CSS：最小全局 reset + 滚动条/选区/焦点/无障碍（壳布局全部交给 Tailwind）
 */

export const BLOCK_SHELL_CSS = `
      /* ── Page ── */
      .page {
        padding: 20px 24px;
        width: 100%;
      }
      .page-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 16px;
      }
      .page-header h1 {
        font-size: 20px;
        font-weight: 700;
        color: hsl(var(--foreground));
      }

      /* ── Skeleton loading ── */
      .skel {
        display: block;
        background: linear-gradient(
          90deg,
          hsl(220 14% 96%) 0%,
          hsl(220 14% 92%) 50%,
          hsl(220 14% 96%) 100%
        );
        background-size: 200% 100%;
        animation: skel-pulse 1.5s ease-in-out infinite;
        border-radius: 4px;
      }
      .skel-text {
        height: 12px;
        width: 100%;
        margin: 4px 0;
      }
      .skel-title {
        height: 20px;
        width: 60%;
        margin: 8px 0;
      }
      .skel-button {
        width: 80px;
        height: 32px;
        border-radius: 6px;
      }
      .skel-row {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 80px;
        gap: 12px;
        padding: 12px 16px;
        align-items: center;
        border-bottom: 1px solid hsl(var(--border));
      }
      @keyframes skel-pulse {
        0%,
        100% {
          background-position: 0% 0%;
        }
        50% {
          background-position: -100% 0%;
        }
      }
      @keyframes spin {
        from {
          transform: rotate(0deg);
        }
        to {
          transform: rotate(360deg);
        }
      }

      /* ── Spinner ── */
      .spinner svg {
        animation: spin 0.8s linear infinite;
        width: 100%;
        height: 100%;
      }
      .spinner-track {
        stroke: hsl(var(--border));
      }
      .spinner-fill {
        stroke: hsl(var(--primary));
      }

      /* ── Chip grid ── */
      .chip-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
        gap: 6px;
        margin-top: 4px;
      }
      .chip-grid label {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
        padding: 6px 10px;
        border: 1px solid hsl(var(--border));
        border-radius: 4px;
        font-size: 12px;
        cursor: pointer;
        background: hsl(var(--card));
        white-space: nowrap;
        transition:
          border-color 0.15s,
          background 0.15s;
      }
      .chip-grid label:hover {
        border-color: hsl(var(--primary));
        background: hsl(var(--primary) / 0.08);
      }
      .chip-grid label input[type='radio'],
      .chip-grid label input[type='checkbox'] {
        position: absolute;
        opacity: 0;
        width: 0;
        height: 0;
        pointer-events: none;
      }

      /* ── Status chip ── */
      .chip {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 2px 8px;
        border-radius: 4px;
        font-size: 12px;
        font-weight: 500;
        line-height: 1.6;
      }
      .done {
        background: hsl(var(--success) / 0.12);
        color: hsl(var(--success));
      }
      .invalid {
        background: hsl(var(--destructive) / 0.12);
        color: hsl(var(--destructive));
      }
      .on {
        background: hsl(var(--primary) / 0.12);
        color: hsl(var(--primary));
      }
      .partial {
        background: hsl(var(--warning) / 0.12);
        color: hsl(var(--warning));
      }

      /* ── Accordion ── */
      .accordion-header {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 10px 16px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        background: none;
        border: none;
        color: hsl(var(--foreground));
        text-align: left;
      }

      /* ── Confirm dialog ── */
      .confirm-dialog {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 440px;
        max-width: calc(100vw - 32px);
        background: hsl(var(--card));
        border-radius: 12px;
        box-shadow: 0 12px 48px hsl(0 0% 0% / 0.2);
        z-index: 201;
        padding: 24px;
      }
      .confirm-overlay {
        position: fixed;
        inset: 0;
        background: hsl(0 0% 0% / 0.4);
        z-index: 200;
      }
      .confirm-message {
        font-size: 13px;
        color: hsl(var(--muted-foreground));
        line-height: 1.6;
        margin-bottom: 16px;
      }
      .confirm-dialog h3 {
        font-size: 16px;
        font-weight: 600;
        margin-bottom: 8px;
        color: hsl(var(--foreground));
      }
      .confirm-footer {
        display: flex;
        gap: 8px;
        justify-content: flex-end;
      }
      .confirm-input {
        width: 100%;
        padding: 8px 12px;
        border: 1px solid hsl(var(--border));
        border-radius: 8px;
        font-size: 13px;
        background: hsl(var(--card));
        margin-bottom: 16px;
        font-family: monospace;
        outline: none;
      }
      .confirm-input:focus {
        border-color: hsl(var(--primary));
        box-shadow: 0 0 0 2px hsl(var(--primary) / 0.15);
      }
      .confirm-input.invalid {
        border-color: hsl(var(--destructive));
      }

      /* ── Drawer ── */
      .drawer {
        position: fixed;
        top: 0;
        right: 0;
        bottom: 0;
        width: 480px;
        background: hsl(var(--card));
        z-index: 101;
        display: flex;
        flex-direction: column;
        box-shadow: -4px 0 24px hsl(0 0% 0% / 0.08);
      }
      .drawer-overlay {
        position: fixed;
        inset: 0;
        background: hsl(0 0% 0% / 0.3);
        z-index: 100;
      }
      .drawer-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px 20px;
        border-bottom: 1px solid hsl(var(--border));
      }
      .drawer-header h3 {
        font-size: 16px;
        font-weight: 600;
      }
      .drawer-body {
        flex: 1;
        overflow-y: auto;
        padding: 20px;
      }
      .drawer-body .form-group {
        margin-bottom: 16px;
      }
      .drawer-body .form-group > label {
        display: block;
        font-size: 13px;
        font-weight: 500;
        color: hsl(var(--foreground));
        margin-bottom: 4px;
      }
      .drawer-footer {
        padding: 12px 20px;
        border-top: 1px solid hsl(var(--border));
        display: flex;
        gap: 8px;
        justify-content: flex-end;
      }
      .drawer-body .form-group input[type='text'],
      .drawer-body .form-group input[type='number'],
      .drawer-body .form-group select,
      .drawer-body .form-group textarea {
        width: 100%;
        padding: 8px 12px;
        border: 1px solid hsl(var(--border));
        border-radius: 8px;
        font-size: 13px;
        background: hsl(var(--card));
        color: hsl(var(--foreground));
        outline: none;
      }
      .drawer-body .form-group input:focus,
      .drawer-body .form-group select:focus,
      .drawer-body .form-group textarea:focus {
        border-color: hsl(var(--primary));
        box-shadow: 0 0 0 2px hsl(var(--primary) / 0.15);
      }

      /* ── Wizard ── */
      .wizard-steps {
        display: flex;
        align-items: center;
        gap: 0;
        padding: 16px 20px 0;
        margin-bottom: 16px;
      }
      .wizard-step {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 13px;
        color: hsl(var(--muted-foreground));
      }
      .wizard-step-dot {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 12px;
        font-weight: 600;
        border: 2px solid hsl(var(--border));
        background: hsl(var(--card));
        color: hsl(var(--muted-foreground));
        flex-shrink: 0;
      }
      .wizard-step.active {
        color: hsl(var(--primary));
        font-weight: 600;
      }
      .wizard-step.active .wizard-step-dot {
        border-color: hsl(var(--primary));
        background: hsl(var(--primary) / 0.1);
        color: hsl(var(--primary));
      }
      .wizard-step.done {
        color: hsl(var(--success));
      }
      .wizard-step.done .wizard-step-dot {
        border-color: hsl(var(--success));
        background: hsl(var(--success));
        color: hsl(var(--success-foreground));
      }
      .wizard-connector {
        width: 32px;
        height: 2px;
        background: hsl(var(--border));
        margin: 0 8px;
        flex-shrink: 0;
      }
      .wizard-connector.done {
        background: hsl(var(--success));
      }

      /* ── Toggle ── */
      .toggle {
        position: relative;
        width: 40px;
        height: 22px;
        background: hsl(var(--border));
        border-radius: 11px;
        border: none;
        cursor: pointer;
        transition: background 0.15s;
        flex-shrink: 0;
      }
      .toggle.on {
        background: hsl(var(--primary));
      }
      .toggle.on::after {
        transform: translateX(18px);
      }
      .toggle::after {
        content: '';
        position: absolute;
        top: 2px;
        left: 2px;
        width: 18px;
        height: 18px;
        background: hsl(var(--primary-foreground));
        border-radius: 50%;
        transition: transform 0.15s;
      }

      /* ── Progress bar ── */
      .progress-bar {
        height: 6px;
        background: hsl(var(--muted));
        border-radius: 3px;
        overflow: hidden;
      }
      .progress-bar > div,
      .progress-bar .fill {
        height: 100%;
        border-radius: 3px;
        background: hsl(var(--primary));
        transition: width 0.3s;
      }
      .progress-bar .fill.full {
        background: hsl(var(--success));
      }
      .progress-bar .fill.partial {
        background: hsl(var(--warning));
      }

      /* ── Utility overrides ── */
      .text-primary-foreground {
        color: hsl(var(--primary-foreground));
      }
      .bg-primary\/8 {
        background: hsl(var(--primary) / 0.08);
      }
      .last\\:border-b-0:last-child {
        border-bottom-width: 0;
      }
      .border-l-\\[3px\\] {
        border-left-width: 3px;
      }
      .focus\\:border-primary\\/30:focus {
        border-color: hsl(var(--primary) / 0.3);
      }
      .hover\\:border-primary:hover {
        border-color: hsl(var(--primary));
      }

      /* boot-skeleton 布局已统一到 shell-shared.ts BOOT_FADE_CSS(对齐 Gateway MainLayout) */
    `;

export const MODULE_SHELL_CSS = `
      /* ════════════════════════════════════════════════════════════════
         === CSS 变量(对齐 Framework theme-base.css) ===
         ════════════════════════════════════════════════════════════════ */
      :root {
        --topbar-height: 3.5rem;            /* 56px — h-14 */
        --sidebar-width: 15rem;             /* 240px — w-60 */
        --sidebar-collapsed-width: 4rem;    /* 64px — w-16 */
      }

      /* ════════════════════════════════════════════════════════════════
         === SHELL-RESET (最小全局 reset,壳布局全部交给 Tailwind 工具类) ===
         ════════════════════════════════════════════════════════════════ */
      * { margin: 0; padding: 0; box-sizing: border-box; }
      html { -webkit-text-size-adjust: 100%; }
      body {
        font-family:
          "Inter",
          ui-sans-serif,
          system-ui,
          -apple-system,
          BlinkMacSystemFont,
          "Segoe UI",
          Roboto,
          "Helvetica Neue",
          Arial,
          "Noto Sans",
          "PingFang SC",
          "Hiragino Sans GB",
          "Microsoft YaHei",
          "WenQuanYi Micro Hei",
          sans-serif;
        background: hsl(var(--background));
        color: hsl(var(--foreground));
        font-size: 14px;
        line-height: 1.5;
        -webkit-font-smoothing: antialiased;
      }
      .font-mono,
      code, pre, kbd, samp {
        font-family:
          "JetBrains Mono",
          ui-monospace,
          SFMono-Regular,
          Menlo,
          Monaco,
          Consolas,
          "Liberation Mono",
          "Courier New",
          monospace;
      }
      button, input, textarea, select { font: inherit; color: inherit; }

      /* ════════════════════════════════════════════════════════════════
         === 全局滚动条 / 选区 / 焦点 / 无障碍(壳布局由 Tailwind 负责) ===
         ════════════════════════════════════════════════════════════════ */
      ::-webkit-scrollbar { width: 8px; height: 8px; }
      ::-webkit-scrollbar-track { background: transparent; }
      ::-webkit-scrollbar-thumb { background: hsl(var(--muted-foreground) / 0.3); border-radius: 4px; }
      ::-webkit-scrollbar-thumb:hover { background: hsl(var(--muted-foreground) / 0.5); }
      ::selection { background: hsl(var(--primary) / 0.2); color: hsl(var(--foreground)); }
      *:focus-visible { outline: 2px solid hsl(var(--primary)); outline-offset: 2px; }
      @media (prefers-reduced-motion: reduce) { *, ::before, ::after { animation-duration: 0.01ms !important; animation-iteration-count: 1 !important; transition-duration: 0.01ms !important; scroll-behavior: auto !important; } }

      /* boot-skeleton 布局已统一到 shell-shared.ts BOOT_FADE_CSS(对齐 Gateway MainLayout);壳 chrome 一律用 Tailwind,禁止 gl-gateway-* */
    `;

export const APP_SHELL_CSS = MODULE_SHELL_CSS;
