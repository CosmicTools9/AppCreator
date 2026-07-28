import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import { StatusBadge } from './status-badge';

describe('StatusBadge', () => {
  it('renders with active variant', () => {
    render(<StatusBadge variant="active" label="生效中" />);
    expect(screen.getByText('生效中')).toBeInTheDocument();
  });

  it('renders with draft variant', () => {
    render(<StatusBadge variant="draft" label="草稿" />);
    expect(screen.getByText('草稿')).toBeInTheDocument();
  });

  it('renders with pending variant', () => {
    render(<StatusBadge variant="pending" label="待处理" />);
    expect(screen.getByText('待处理')).toBeInTheDocument();
  });

  it('renders with archived variant', () => {
    render(<StatusBadge variant="archived" label="已归档" />);
    expect(screen.getByText('已归档')).toBeInTheDocument();
  });

  it('renders with rejected variant', () => {
    render(<StatusBadge variant="rejected" label="已拒绝" />);
    expect(screen.getByText('已拒绝')).toBeInTheDocument();
  });

  it('renders children when label is not provided', () => {
    render(<StatusBadge variant="active">生效中</StatusBadge>);
    expect(screen.getByText('生效中')).toBeInTheDocument();
  });

  it('renders status dot', () => {
    const { container } = render(<StatusBadge variant="active" label="生效中" />);
    const dot = container.querySelector('span[class*="rounded-full"]');
    expect(dot).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(
      <StatusBadge variant="active" label="生效中" className="custom-badge" />
    );
    expect(container.firstChild).toHaveClass('custom-badge');
  });

  describe('token mode', () => {
    it('renders with success token', () => {
      render(<StatusBadge token="success" label="已审核" />);
      expect(screen.getByText('已审核')).toBeInTheDocument();
    });

    it('renders with warning token', () => {
      render(<StatusBadge token="warning" label="待复核" />);
      expect(screen.getByText('待复核')).toBeInTheDocument();
    });

    it('renders with danger token', () => {
      render(<StatusBadge token="danger" label="异常" />);
      expect(screen.getByText('异常')).toBeInTheDocument();
    });

    it('renders with info token', () => {
      render(<StatusBadge token="info" label="进行中" />);
      expect(screen.getByText('进行中')).toBeInTheDocument();
    });

    it('renders with neutral token', () => {
      render(<StatusBadge token="neutral" label="已归档" />);
      expect(screen.getByText('已归档')).toBeInTheDocument();
    });

    it('renders with domain-specific token (active)', () => {
      render(<StatusBadge token="active" label="营业" />);
      expect(screen.getByText('营业')).toBeInTheDocument();
    });

    it('renders with domain-specific token (locked)', () => {
      render(<StatusBadge token="locked" label="锁定" />);
      expect(screen.getByText('锁定')).toBeInTheDocument();
    });

    it('renders with domain-specific token (exception)', () => {
      render(<StatusBadge token="exception" label="异常" />);
      expect(screen.getByText('异常')).toBeInTheDocument();
    });

    it('renders status dot in token mode', () => {
      const { container } = render(<StatusBadge token="success" label="已审核" />);
      const dot = container.querySelector('span[class*="rounded-full"]');
      expect(dot).toBeInTheDocument();
    });

    it('renders children when label is not provided in token mode', () => {
      render(<StatusBadge token="success">已审核</StatusBadge>);
      expect(screen.getByText('已审核')).toBeInTheDocument();
    });
  });

  it('token prop takes precedence over variant when both provided', () => {
    render(<StatusBadge variant="active" token="danger" label="覆盖" />);
    expect(screen.getByText('覆盖')).toBeInTheDocument();
  });
});
