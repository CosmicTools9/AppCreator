/**
 * ScrollTabs — 横向可滚动的标签页容器
 *
 * 设计目标：当标签页数量超出可视宽度时（典型场景：复杂应用的模块标签过多），
 * 提供完整的水平滚动支持：
 *
 * 1. **滚动条隐藏**：用 `hide-scrollbar` 工具类保持视觉简洁（与 HTML_DESIGN_SPEC 一致）
 * 2. **左右渐变提示**：内容超出时两端显示半透明渐变，提示可滚动
 * 3. **左右滚动按钮**：始终渲染但仅在可滚动时显示，避免按钮占用空间
 * 4. **滚轮转横向**：垂直滚轮（带 Shift 修饰键）转换为横向滚动 — 跨平台统一行为
 * 5. **激活标签可见**：路由变化时自动将激活标签滚入视口
 * 6. **键盘导航**：左右方向键在标签间切换，Home/End 跳到首/末
 *
 * 用法：包裹任意子标签集合（无需关心内部实现）：
 *
 *   <ScrollTabs aria-label="模块导航">
 *     <ModuleTabs modules={appModules} appName={appName} />
 *   </ScrollTabs>
 *
 * 容器通过 `data-tab-active` 标记查找激活标签（与 ModuleTabs 约定）。
 */

/**
 * 安全的 scrollBy：优先调用原生 scrollBy（生产环境），否则直接修改 scrollLeft
 * （jsdom 单元测试环境或老浏览器）。
 */
function scrollByAmount(el: HTMLElement, left: number): void {
  if (typeof el.scrollBy === "function") {
    el.scrollBy({ left, behavior: "auto" });
    return;
  }
  const max = el.scrollWidth - el.clientWidth;
  const next = Math.max(0, Math.min(max, el.scrollLeft + left));
  el.scrollLeft = next;
}


import * as React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "../../lib/utils";

export interface ScrollTabsProps {
  /** 标签集合（任意 ReactNode，须包含至少一个 [data-tab-active] 元素以触发自动滚入） */
  children: React.ReactNode;
  /** 滚动按钮的像素步长（默认 200） */
  scrollStep?: number;
  /** 是否启用滚轮转横向（带 Shift 修饰键，默认 true） */
  wheelScroll?: boolean;
  /** 激活标签变化时是否自动滚入视口（默认 true） */
  autoScrollActive?: boolean;
  /** 滚轮滚动方向：'x' 横滚、'y' 纵滚（覆盖默认 Shift 行为） */
  wheelAxis?: "x" | "y" | "auto";
  /** 额外 className */
  className?: string;
  /** 容器 aria-label */
  "aria-label"?: string;
}

export const ScrollTabs = React.forwardRef<HTMLDivElement, ScrollTabsProps>(
  (
    {
      children,
      scrollStep = 200,
      wheelScroll = true,
      autoScrollActive = true,
      wheelAxis = "auto",
      className,
      "aria-label": ariaLabel,
    },
    ref,
  ) => {
    const scrollerRef = React.useRef<HTMLDivElement | null>(null);
    const [canScrollLeft, setCanScrollLeft] = React.useState(false);
    const [canScrollRight, setCanScrollRight] = React.useState(false);

    // 合并外部 ref
    const setRefs = React.useCallback(
      (node: HTMLDivElement | null) => {
        scrollerRef.current = node;
        if (typeof ref === "function") ref(node);
        else if (ref) (ref as React.MutableRefObject<HTMLDivElement | null>).current = node;
      },
      [ref],
    );

    // 更新滚动状态（左右是否可滚）
    const updateScrollState = React.useCallback(() => {
      const el = scrollerRef.current;
      if (!el) return;
      const { scrollLeft, scrollWidth, clientWidth } = el;
      // 1px 容差避免亚像素抖动
      setCanScrollLeft(scrollLeft > 1);
      setCanScrollRight(scrollLeft + clientWidth < scrollWidth - 1);
    }, []);

    // 监听尺寸变化
    React.useEffect(() => {
      const el = scrollerRef.current;
      if (!el) return;

      updateScrollState();
      el.addEventListener("scroll", updateScrollState, { passive: true });
      // ResizeObserver 已在 test/setup.ts 与现代浏览器中提供
      const ro = new ResizeObserver(updateScrollState);
      ro.observe(el);
      // 内容变化（tabs 增加/删除）也会影响 scrollWidth
      const mo = new MutationObserver(updateScrollState);
      mo.observe(el, { childList: true, subtree: true, characterData: true });

      return () => {
        el.removeEventListener("scroll", updateScrollState);
        ro.disconnect();
        mo.disconnect();
      };
    }, [updateScrollState]);

    // 滚轮监听必须用 passive: false 才能 preventDefault。
    // 单独挂在原生事件上，避免与 React 合成事件重复触发。
    React.useEffect(() => {
      const el = scrollerRef.current;
      if (!el || !wheelScroll) return;
      const nativeHandler = (e: WheelEvent) => {
        // auto 模式：仅当 shift+wheel(无横滚意图)才转横滚，避免与正常竖滚冲突
        if (wheelAxis === "auto" && !(e.shiftKey && e.deltaX === 0)) return;
        if (wheelAxis === "y") return;

        // 已在边界则不阻止
        const { scrollLeft, scrollWidth, clientWidth } = el;
        const atLeft = scrollLeft <= 0;
        const atRight = scrollLeft + clientWidth >= scrollWidth;
        if ((e.deltaY < 0 && atLeft) || (e.deltaY > 0 && atRight)) return;

        e.preventDefault();
        scrollByAmount(el, e.deltaY);
      };
      el.addEventListener("wheel", nativeHandler, { passive: false });
      return () => el.removeEventListener("wheel", nativeHandler);
    }, [wheelScroll, wheelAxis]);

    // 滚动按钮
    const scrollBy = React.useCallback(
      (dir: -1 | 1) => {
        const el = scrollerRef.current;
        if (!el) return;
        scrollByAmount(el, dir * scrollStep);
      },
      [scrollStep],
    );

    // 自动滚入激活标签
    React.useEffect(() => {
      if (!autoScrollActive) return;
      const el = scrollerRef.current;
      if (!el) return;
      const active = el.querySelector<HTMLElement>("[data-tab-active='true']");
      if (!active) return;

      // scrollIntoView 在 test/setup.ts 中已 mock
      const elRect = el.getBoundingClientRect();
      const aRect = active.getBoundingClientRect();
      const margin = 24;
      const deltaLeft = aRect.left - elRect.left - margin;
      const deltaRight = aRect.right - elRect.right + margin;
      if (aRect.left < elRect.left + margin) {
        scrollByAmount(el, deltaLeft);
      } else if (aRect.right > elRect.right - margin) {
        scrollByAmount(el, deltaRight);
      }
    }, [autoScrollActive, children]);

    // 键盘导航：ArrowLeft/Right 在激活标签的同级 [role="tab"] 间切换
    const handleKeyDown = React.useCallback((e: React.KeyboardEvent<HTMLDivElement>) => {
      const target = e.target as HTMLElement;
      const tab = target.closest<HTMLElement>('[role="tab"]');
      if (!tab) return;

      const tabsContainer = tab.parentElement;
      if (!tabsContainer) return;
      const tabs = Array.from(
        tabsContainer.querySelectorAll<HTMLElement>('[role="tab"]:not([aria-disabled="true"])'),
      );
      const idx = tabs.indexOf(tab);
      if (idx < 0) return;

      let next: HTMLElement | undefined;
      switch (e.key) {
        case "ArrowLeft":
          next = tabs[idx - 1] ?? tabs[tabs.length - 1];
          break;
        case "ArrowRight":
          next = tabs[idx + 1] ?? tabs[0];
          break;
        case "Home":
          next = tabs[0];
          break;
        case "End":
          next = tabs[tabs.length - 1];
          break;
      }
      if (next) {
        e.preventDefault();
        next.focus();
        // 若该标签是 <a>，Enter 也会激活；为了支持纯键盘切换，模拟点击
        // 注意：滚动按钮拦截不到原生 click
        if (next.tagName === "A") {
          // 触发 react-router 的 Link 点击
          next.click();
        }
      }
    }, []);

    const showLeftButton = canScrollLeft;
    const showRightButton = canScrollRight;

    return (
      <div
        className={cn("relative flex items-center min-w-0 flex-1", className)}
        data-testid="scroll-tabs"
      >
        {/* 左侧滚动按钮 */}
        <button
          type="button"
          aria-label="向左滚动标签"
          tabIndex={-1}
          onClick={() => scrollBy(-1)}
          className={cn(
            "shrink-0 h-7 w-6 flex items-center justify-center rounded-md",
            "text-muted-foreground hover:bg-accent hover:text-foreground transition-all",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            showLeftButton
              ? "opacity-100 pointer-events-auto"
              : "opacity-0 pointer-events-none w-0 -mr-0.5",
          )}
        >
          <ChevronLeft className="w-3.5 h-3.5" />
        </button>

        {/* 左侧渐变 */}
        <div
          aria-hidden="true"
          className={cn(
            "pointer-events-none absolute left-6 top-0 bottom-0 w-8 z-10 transition-opacity",
            "bg-gradient-to-r from-background to-transparent",
            showLeftButton ? "opacity-100" : "opacity-0",
          )}
        />

        {/* 可滚动容器 */}
        <div
          ref={setRefs}
          role="tablist"
          aria-label={ariaLabel}
          onKeyDown={handleKeyDown}
          className={cn(
            "flex items-center gap-0.5 overflow-x-auto hide-scrollbar",
            "min-w-0 flex-1 scroll-smooth",
            "focus-within:outline-none",
          )}
        >
          {children}
        </div>

        {/* 右侧渐变 */}
        <div
          aria-hidden="true"
          className={cn(
            "pointer-events-none absolute right-6 top-0 bottom-0 w-8 z-10 transition-opacity",
            "bg-gradient-to-l from-background to-transparent",
            showRightButton ? "opacity-100" : "opacity-0",
          )}
        />

        {/* 右侧滚动按钮 */}
        <button
          type="button"
          aria-label="向右滚动标签"
          tabIndex={-1}
          onClick={() => scrollBy(1)}
          className={cn(
            "shrink-0 h-7 w-6 flex items-center justify-center rounded-md",
            "text-muted-foreground hover:bg-accent hover:text-foreground transition-all",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            showRightButton
              ? "opacity-100 pointer-events-auto"
              : "opacity-0 pointer-events-none w-0 -gl-0.5",
          )}
        >
          <ChevronRight className="w-3.5 h-3.5" />
        </button>
      </div>
    );
  },
);
ScrollTabs.displayName = "ScrollTabs";
