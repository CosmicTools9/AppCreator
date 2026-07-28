import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import {
  OrderCateBadge,
  ORDER_CATE_COLORS,
} from './badge-tf';

describe('OrderCateBadge', () => {
  it('renders nothing for null value', () => {
    const { container } = render(<OrderCateBadge value={null} />);
    expect(container.textContent).toBe('—');
  });

  it('renders sea badge', () => {
    render(<OrderCateBadge value="sea" />);
    expect(screen.getByText('海运')).toBeInTheDocument();
  });

  it('renders air badge', () => {
    render(<OrderCateBadge value="air" />);
    expect(screen.getByText('空运')).toBeInTheDocument();
  });

  it('renders land badge', () => {
    render(<OrderCateBadge value="land" />);
    expect(screen.getByText('陆运')).toBeInTheDocument();
  });

  it('renders rail badge', () => {
    render(<OrderCateBadge value="rail" />);
    expect(screen.getByText('铁路')).toBeInTheDocument();
  });

  it('treats ocean as sea', () => {
    render(<OrderCateBadge value="ocean" />);
    expect(screen.getByText('海运')).toBeInTheDocument();
  });

  it('treats road as 公路', () => {
    render(<OrderCateBadge value="road" />);
    expect(screen.getByText('公路')).toBeInTheDocument();
  });

  it('all ORDER_CATE_COLORS entries have valid cls', () => {
    for (const [key, config] of Object.entries(ORDER_CATE_COLORS)) {
      expect(typeof config.label).toBe('string');
      expect(config.cls).toContain('bg-');
    }
  });
});
