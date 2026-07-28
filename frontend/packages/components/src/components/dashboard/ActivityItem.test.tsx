import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { ActivityItem } from './ActivityItem';

describe('ActivityItem', () => {
  it('renders title', () => {
    render(<ActivityItem title="User logged in" />);
    expect(screen.getByText('User logged in')).toBeInTheDocument();
  });

  it('renders description when provided', () => {
    render(
      <ActivityItem title="Login" description="From IP 192.168.1.1" />,
    );
    expect(screen.getByText('From IP 192.168.1.1')).toBeInTheDocument();
  });

  it('renders timestamp when provided', () => {
    render(
      <ActivityItem title="Action" timestamp="5 min ago" />,
    );
    expect(screen.getByText('5 min ago')).toBeInTheDocument();
  });

  it('renders avatar when provided', () => {
    render(
      <ActivityItem
        title="Action"
        avatar={<span data-testid="custom-avatar">A</span>}
      />,
    );
    expect(screen.getByTestId('custom-avatar')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const handleClick = vi.fn();
    render(<ActivityItem title="Clickable" onClick={handleClick} />);
    await userEvent.click(screen.getByText('Clickable'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies custom className', () => {
    const { container } = render(
      <ActivityItem title="Styled" className="custom-class" />,
    );
    expect(container.firstElementChild).toHaveClass('custom-class');
  });
});
