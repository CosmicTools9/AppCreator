import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, act } from '@/test/test-utils';
import { ScrollTabs } from './ScrollTabs';

/**
 * 构造可测量的可滚动 DOM
 * 真实 jsdom 不计算 layout，所以手动 mock clientWidth/scrollWidth/scrollLeft
 */
interface ScrollMetrics {
  clientWidth: number;
  scrollWidth: number;
  scrollLeft: number;
}

function applyMetrics(el: HTMLElement, metrics: ScrollMetrics) {
  Object.defineProperty(el, 'clientWidth', { configurable: true, get: () => metrics.clientWidth });
  Object.defineProperty(el, 'scrollWidth', { configurable: true, get: () => metrics.scrollWidth });
  Object.defineProperty(el, 'scrollLeft', {
    configurable: true,
    get: () => metrics.scrollLeft,
    set: (v: number) => { metrics.scrollLeft = v; },
  });
  Object.defineProperty(el, 'scrollBy', {
    configurable: true,
    writable: true,
    value: (opts: { left?: number; behavior?: ScrollBehavior } | number, _y?: number) => {
      if (typeof opts === 'number') {
        metrics.scrollLeft = Math.max(0, metrics.scrollLeft + opts);
      } else if (typeof opts === 'object' && opts !== null && typeof opts.left === 'number') {
        metrics.scrollLeft = Math.max(0, metrics.scrollLeft + opts.left);
      }
    },
  });
}

describe('ScrollTabs', () => {
  describe('scroll state', () => {
    it('hides both scroll buttons when content fits viewport', () => {
      // 5 个 tab × 50px = 250，clientWidth=500，可全部显示
      const metrics: ScrollMetrics = { clientWidth: 500, scrollWidth: 250, scrollLeft: 0 };
      const { container } = render(
        <ScrollTabs>
          {Array.from({ length: 5 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      // 找到滚动容器
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      // 触发 scroll 事件让组件重算
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      // 滚动按钮 opacity 应为 0
      const leftBtn = screen.getByLabelText('向左滚动标签');
      const rightBtn = screen.getByLabelText('向右滚动标签');
      expect(leftBtn.className).toMatch(/opacity-0/);
      expect(rightBtn.className).toMatch(/opacity-0/);
    });

    it('shows right scroll button when content overflows to the right', () => {
      // 10 个 tab × 50px = 500，clientWidth=200，scrollLeft=0 → 可向右
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 0 };
      const { container } = render(
        <ScrollTabs>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      const rightBtn = screen.getByLabelText('向右滚动标签');
      expect(rightBtn.className).toMatch(/opacity-100/);
      const leftBtn = screen.getByLabelText('向左滚动标签');
      expect(leftBtn.className).toMatch(/opacity-0/);
    });

    it('shows left scroll button after scrolling to the right', () => {
      // 初始 scrollLeft=0，模拟点击 right 按钮 → scrollLeft=200
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 200 };
      const { container } = render(
        <ScrollTabs>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      const leftBtn = screen.getByLabelText('向左滚动标签');
      const rightBtn = screen.getByLabelText('向右滚动标签');
      expect(leftBtn.className).toMatch(/opacity-100/);
      // scrollLeft+clientWidth=400 < scrollWidth=500 → 仍可向右
      expect(rightBtn.className).toMatch(/opacity-100/);
    });

    it('hides right button when scrolled to the end', () => {
      // scrollLeft=300, clientWidth=200, scrollWidth=500 → 300+200=500=scrollWidth
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 300 };
      const { container } = render(
        <ScrollTabs>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      const rightBtn = screen.getByLabelText('向右滚动标签');
      expect(rightBtn.className).toMatch(/opacity-0/);
      const leftBtn = screen.getByLabelText('向左滚动标签');
      expect(leftBtn.className).toMatch(/opacity-100/);
    });
  });

  describe('scroll buttons', () => {
    it('right button calls scrollBy on the scroller with positive step', () => {
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 0 };
      const { container } = render(
        <ScrollTabs scrollStep={100}>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      const rightBtn = screen.getByLabelText('向右滚动标签');
      act(() => {
        fireEvent.click(rightBtn);
      });
      expect(metrics.scrollLeft).toBe(100);
    });

    it('left button decreases scrollLeft', () => {
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 200 };
      const { container } = render(
        <ScrollTabs scrollStep={100}>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      const leftBtn = screen.getByLabelText('向左滚动标签');
      act(() => {
        fireEvent.click(leftBtn);
      });
      expect(metrics.scrollLeft).toBe(100);
    });

    it('does not allow scrollLeft to go negative', () => {
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 50 };
      const { container } = render(
        <ScrollTabs scrollStep={100}>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      const leftBtn = screen.getByLabelText('向左滚动标签');
      act(() => {
        fireEvent.click(leftBtn);
      });
      expect(metrics.scrollLeft).toBe(0); // 50 - 100 = -50 → clamp 到 0
    });
  });

  describe('wheel-to-horizontal', () => {
    it('translates vertical wheel (with shift) to horizontal scroll', () => {
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 100 };
      const { container } = render(
        <ScrollTabs>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });

      // shift+wheel 转为横滚
      const evt = new WheelEvent('wheel', {
        shiftKey: true,
        deltaY: 80,
        deltaX: 0,
        bubbles: true,
        cancelable: true,
      });
      const preventDefaultSpy = vi.spyOn(evt, 'preventDefault');
      act(() => {
        scroller.dispatchEvent(evt);
      });
      expect(preventDefaultSpy).toHaveBeenCalled();
      expect(metrics.scrollLeft).toBe(180); // 100 + 80
    });

    it('does not preventDefault on plain vertical wheel (no shift)', () => {
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 100 };
      const { container } = render(
        <ScrollTabs>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });

      const evt = new WheelEvent('wheel', { deltaY: 80, bubbles: true, cancelable: true });
      const preventDefaultSpy = vi.spyOn(evt, 'preventDefault');
      act(() => {
        scroller.dispatchEvent(evt);
      });
      expect(preventDefaultSpy).not.toHaveBeenCalled();
      expect(metrics.scrollLeft).toBe(100); // unchanged
    });

    it('does not scroll at boundaries', () => {
      // 已经在最左 → shift+wheel 向上仍应不滚动
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 0 };
      const { container } = render(
        <ScrollTabs>
          {Array.from({ length: 10 }, (_, i) => (
            <button key={i} type="button" role="tab">Tab {i + 1}</button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      applyMetrics(scroller, metrics);
      act(() => {
        scroller.dispatchEvent(new Event('scroll'));
      });
      const evt = new WheelEvent('wheel', {
        shiftKey: true,
        deltaY: -50,
        bubbles: true,
        cancelable: true,
      });
      act(() => {
        scroller.dispatchEvent(evt);
      });
      expect(metrics.scrollLeft).toBe(0);
    });
  });

  describe('auto-scroll active into view', () => {
    it('scrolls active tab into view when scrolled out to the right', () => {
      const metrics: ScrollMetrics = { clientWidth: 200, scrollWidth: 500, scrollLeft: 250 };
      const { container, rerender } = render(
        <ScrollTabs>
          {Array.from({ length: 10 }, (_, i) => (
            <button
              key={i}
              type="button"
              role="tab"
              data-tab-active={i === 8 ? 'true' : undefined}
            >
              Tab {i + 1}
            </button>
          ))}
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;

      // 全部副作用在 act 内完成：注入 metrics → 模拟 getBoundingClientRect → 触发 rerender
      act(() => {
        applyMetrics(scroller, metrics);

        const active = scroller.querySelector<HTMLElement>('[data-tab-active="true"]')!;
        Object.defineProperty(active, 'getBoundingClientRect', {
          configurable: true,
          value: () => ({ left: 600, right: 660, top: 0, bottom: 0, width: 60, height: 30, x: 600, y: 0, toJSON: () => '' }),
        });
        Object.defineProperty(scroller, 'getBoundingClientRect', {
          configurable: true,
          value: () => ({ left: 0, right: 200, top: 0, bottom: 30, width: 200, height: 30, x: 0, y: 0, toJSON: () => '' }),
        });

        rerender(
          <ScrollTabs>
            {Array.from({ length: 10 }, (_, i) => (
              <button
                key={i}
                type="button"
                role="tab"
                data-tab-active={i === 8 ? 'true' : undefined}
              >
                Tab {i + 1} v2
              </button>
            ))}
          </ScrollTabs>,
        );
      });

      // 激活标签的 right=660 > scroller 的 right=200 → 滚动到使其可见
      let observed = 0;
      act(() => {
        observed = metrics.scrollLeft;
      });
      expect(observed).toBeGreaterThan(250);
    });
  });


  describe('keyboard navigation', () => {
    it('ArrowRight focuses next tab', () => {
      const { container } = render(
        <ScrollTabs>
          <button type="button" role="tab">A</button>
          <button type="button" role="tab">B</button>
          <button type="button" role="tab">C</button>
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      const tabs = scroller.querySelectorAll<HTMLElement>('[role="tab"]');
      // 聚焦 A
      act(() => {
        tabs[0].focus();
      });
      act(() => {
        fireEvent.keyDown(tabs[0], { key: 'ArrowRight' });
      });
      expect(document.activeElement).toBe(tabs[1]);
    });

    it('ArrowLeft focuses previous tab', () => {
      const { container } = render(
        <ScrollTabs>
          <button type="button" role="tab">A</button>
          <button type="button" role="tab">B</button>
          <button type="button" role="tab">C</button>
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      const tabs = scroller.querySelectorAll<HTMLElement>('[role="tab"]');
      act(() => {
        tabs[2].focus();
      });
      act(() => {
        fireEvent.keyDown(tabs[2], { key: 'ArrowLeft' });
      });
      expect(document.activeElement).toBe(tabs[1]);
    });

    it('ArrowRight at last wraps to first', () => {
      const { container } = render(
        <ScrollTabs>
          <button type="button" role="tab">A</button>
          <button type="button" role="tab">B</button>
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      const tabs = scroller.querySelectorAll<HTMLElement>('[role="tab"]');
      act(() => {
        tabs[1].focus();
      });
      act(() => {
        fireEvent.keyDown(tabs[1], { key: 'ArrowRight' });
      });
      expect(document.activeElement).toBe(tabs[0]);
    });

    it('Home focuses first tab', () => {
      const { container } = render(
        <ScrollTabs>
          <button type="button" role="tab">A</button>
          <button type="button" role="tab">B</button>
          <button type="button" role="tab">C</button>
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      const tabs = scroller.querySelectorAll<HTMLElement>('[role="tab"]');
      act(() => {
        tabs[2].focus();
      });
      act(() => {
        fireEvent.keyDown(tabs[2], { key: 'Home' });
      });
      expect(document.activeElement).toBe(tabs[0]);
    });

    it('End focuses last tab', () => {
      const { container } = render(
        <ScrollTabs>
          <button type="button" role="tab">A</button>
          <button type="button" role="tab">B</button>
          <button type="button" role="tab">C</button>
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      const tabs = scroller.querySelectorAll<HTMLElement>('[role="tab"]');
      act(() => {
        tabs[0].focus();
      });
      act(() => {
        fireEvent.keyDown(tabs[0], { key: 'End' });
      });
      expect(document.activeElement).toBe(tabs[2]);
    });
  });

  describe('accessibility', () => {
    it('sets role=tablist and aria-label on the scroller', () => {
      const { container } = render(
        <ScrollTabs aria-label="模块导航">
          <button type="button" role="tab">A</button>
        </ScrollTabs>,
      );
      const scroller = container.querySelector<HTMLElement>('[role="tablist"]')!;
      expect(scroller).toHaveAttribute('aria-label', '模块导航');
    });

    it('scroll buttons are not in tab order (tabIndex=-1)', () => {
      render(
        <ScrollTabs>
          <button type="button" role="tab">A</button>
        </ScrollTabs>,
      );
      const leftBtn = screen.getByLabelText('向左滚动标签');
      const rightBtn = screen.getByLabelText('向右滚动标签');
      expect(leftBtn).toHaveAttribute('tabindex', '-1');
      expect(rightBtn).toHaveAttribute('tabindex', '-1');
    });
  });
});
