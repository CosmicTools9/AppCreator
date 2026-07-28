import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { StatCard } from './StatCard';
import { ActivityIcon } from 'lucide-react';

describe('StatCard', () => {
  it('renders label and value', () => {
    render(<StatCard label="Total Orders" value="1,234" icon={ActivityIcon} />);
    expect(screen.getByText('Total Orders')).toBeInTheDocument();
    expect(screen.getByText('1,234')).toBeInTheDocument();
  });

  it('renders unit separately from value', () => {
    render(<StatCard label="Revenue" value="99.9" unit="K" icon={ActivityIcon} />);
    expect(screen.getByText('99.9')).toBeInTheDocument();
    expect(screen.getByText('K')).toBeInTheDocument();
  });

  it('shows positive trend as absolute percentage', () => {
    render(<StatCard label="Sales" value="500" change={12.5} icon={ActivityIcon} />);
    expect(screen.getByText('12.5%')).toBeInTheDocument();
  });

  it('shows negative trend as absolute percentage', () => {
    render(<StatCard label="Sales" value="500" change={-5.3} icon={ActivityIcon} />);
    // change value is rendered as Math.abs(change)% for numbers
    expect(screen.getByText('5.3%')).toBeInTheDocument();
  });

  it('shows explicit up trend', () => {
    render(
      <StatCard label="Sales" value="500" change="+20%" trend="up" icon={ActivityIcon} />,
    );
    expect(screen.getByText('+20%')).toBeInTheDocument();
  });

  it('shows explicit down trend', () => {
    render(
      <StatCard label="Sales" value="500" change="-15%" trend="down" icon={ActivityIcon} />,
    );
    expect(screen.getByText('-15%')).toBeInTheDocument();
  });

  it('handles click events', async () => {
    const handleClick = vi.fn();
    render(
      <StatCard label="Clickable" value="42" icon={ActivityIcon} onClick={handleClick} />,
    );
    await userEvent.click(screen.getByText('Clickable').closest('div')!);
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies custom className', () => {
    const { container } = render(
      <StatCard label="Test" value="1" icon={ActivityIcon} className="custom-class" />,
    );
    expect(container.firstElementChild).toHaveClass('custom-class');
  });
});
