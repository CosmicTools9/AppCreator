//! ESC 键回退 Hook
//!
//! 按页面上元素状态逐步退化处理：
//!   1. 取消：若当前上下文有「取消」按钮，ESC 触发取消（点击该按钮）
//!   2. 关闭：若有打开的弹窗/Drawer/对话框，ESC 触发关闭（点击关闭按钮）
//!   3. 回退：默认行为，浏览器 history.back()

import { useEffect, useCallback } from "react";

/** 判断元素是否可见 */
function isVisible(el: HTMLElement): boolean {
  const rect = el.getBoundingClientRect();
  return (
    rect.width > 0 &&
    rect.height > 0 &&
    window.getComputedStyle(el).visibility !== "hidden" &&
    window.getComputedStyle(el).display !== "none"
  );
}

/** 查找当前可见的取消按钮（优先在当前 dialog 内，其次全局） */
function findCancelButton(): HTMLElement | null {
  // 1. 优先在打开的 dialog 内部查找 data-cancel-button
  const openDialog = document.querySelector<HTMLElement>(
    '[role="dialog"][data-state="open"]',
  );
  if (openDialog) {
    const cancelBtn = openDialog.querySelector<HTMLElement>(
      "button[data-cancel-button]",
    );
    if (cancelBtn && isVisible(cancelBtn)) return cancelBtn;
  }

  // 2. 全局查找可见的取消按钮
  const allCancelBtns =
    document.querySelectorAll<HTMLElement>("button[data-cancel-button]");
  for (const b of allCancelBtns) {
    if (isVisible(b)) return b;
  }

  return null;
}

/** 查找当前打开的弹窗/Drawer/对话框的关闭按钮 */
function findCloseButton(): HTMLElement | null {
  const openDialog = document.querySelector<HTMLElement>(
    '[role="dialog"][data-state="open"]',
  );
  if (!openDialog) return null;

  // 1. 查找 data-close-button 标记的按钮（语言无关）
  const closeBtn = openDialog.querySelector<HTMLElement>(
    "button[data-close-button]",
  );
  if (closeBtn && isVisible(closeBtn)) return closeBtn;

  // 2. 兼容：查找 aria-label="Close" 的按钮（Radix UI 默认）
  const legacyCloseBtn = openDialog.querySelector<HTMLElement>(
    'button[aria-label="Close"]',
  );
  if (legacyCloseBtn && isVisible(legacyCloseBtn)) return legacyCloseBtn;

  return null;
}

export function useEscBack() {
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key !== "Escape") return;
    // 同一事件若已被其他实例处理则跳过
    if (e.defaultPrevented) return;

    // 1. 取消
    const cancelBtn = findCancelButton();
    if (cancelBtn) {
      e.preventDefault();
      cancelBtn.click();
      return;
    }

    // 2. 关闭（弹窗/Drawer/对话框）
    const closeBtn = findCloseButton();
    if (closeBtn) {
      e.preventDefault();
      closeBtn.click();
      return;
    }

    // 3. 回退
    e.preventDefault();
    window.history.back();
  }, []);

  useEffect(() => {
    // 使用捕获阶段，确保先于 Radix 等冒泡阶段的监听器执行
    document.addEventListener("keydown", handleKeyDown, true);
    return () => document.removeEventListener("keydown", handleKeyDown, true);
  }, [handleKeyDown]);
}
