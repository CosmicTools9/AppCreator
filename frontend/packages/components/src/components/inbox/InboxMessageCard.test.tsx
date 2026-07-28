import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { InboxMessageCard } from './InboxMessageCard';
import type { InboxMessage } from './types';

const baseMessage: InboxMessage = {
  id: 1,
  from: 'Alice',
  title: 'New order received',
  content: 'Order #12345 has been placed',
  time: '2 min ago',
  unread: true,
  type: 'message',
};

describe('InboxMessageCard', () => {
  it('renders message title', () => {
    render(<InboxMessageCard message={baseMessage} />);
    expect(screen.getByText('New order received')).toBeInTheDocument();
  });

  it('renders message content', () => {
    render(<InboxMessageCard message={baseMessage} />);
    expect(screen.getByText('Order #12345 has been placed')).toBeInTheDocument();
  });

  it('renders sender name initial as avatar fallback', () => {
    render(<InboxMessageCard message={baseMessage} />);
    expect(screen.getByText('A')).toBeInTheDocument();
  });

  it('renders unread indicator when unread is true', () => {
    const { container } = render(<InboxMessageCard message={baseMessage} />);
    // Unread dot should be present
    const dot = container.querySelector('.rounded-full.bg-destructive');
    expect(dot).toBeInTheDocument();
  });

  it('does not render unread indicator when unread is false', () => {
    const { container } = render(
      <InboxMessageCard message={{ ...baseMessage, unread: false }} />,
    );
    const dot = container.querySelector('.rounded-full.bg-destructive');
    expect(dot).not.toBeInTheDocument();
  });

  it('shows selected styling when selected', () => {
    const { container } = render(
      <InboxMessageCard message={baseMessage} selected={true} />,
    );
    const card = container.firstElementChild;
    expect(card).toHaveClass('border-primary');
  });

  it('calls onClick when card is clicked', async () => {
    const handleClick = vi.fn();
    render(
      <InboxMessageCard message={baseMessage} onClick={handleClick} />,
    );
    await userEvent.click(screen.getByText('New order received').closest('div')!);
    expect(handleClick).toHaveBeenCalledWith(baseMessage);
  });

  it('shows system icon for system messages', () => {
    const { container } = render(
      <InboxMessageCard message={{ ...baseMessage, type: 'system' }} />,
    );
    // Mail icon should be rendered (lucide-react renders SVG)
    const mailIcon = container.querySelector('svg');
    expect(mailIcon).toBeInTheDocument();
  });
});
