import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { QuickLink } from './QuickLink';
import { Home } from 'lucide-react';

describe('QuickLink', () => {
  it('renders label text', () => {
    render(<QuickLink label="Dashboard" icon={Home} />);
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });

  it('renders description when provided', () => {
    render(
      <QuickLink label="Dashboard" description="View your dashboard" icon={Home} />,
    );
    expect(screen.getByText('View your dashboard')).toBeInTheDocument();
  });

  it('renders as link when href provided', () => {
    render(<QuickLink label="Settings" icon={Home} href="/settings" />);
    const link = screen.getByRole('link');
    expect(link).toHaveAttribute('href', '/settings');
  });

  it('renders as button when no href', () => {
    render(<QuickLink label="Click Me" icon={Home} />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('handles click events', async () => {
    const handleClick = vi.fn();
    render(<QuickLink label="Clickable" icon={Home} onClick={handleClick} />);
    await userEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies custom className', () => {
    const { container } = render(
      <QuickLink label="Styled" icon={Home} className="custom-class" />,
    );
    expect(container.firstElementChild).toHaveClass('custom-class');
  });
});
