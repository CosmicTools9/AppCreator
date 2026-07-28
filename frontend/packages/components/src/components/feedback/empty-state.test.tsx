import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { EmptyState } from './empty-state';

describe('EmptyState', () => {
  it('renders title', () => {
    render(<EmptyState title="No items found" />);
    expect(screen.getByText('No items found')).toBeInTheDocument();
  });

  it('renders description when provided', () => {
    render(
      <EmptyState title="No items" description="Try adjusting your filters" />,
    );
    expect(screen.getByText('Try adjusting your filters')).toBeInTheDocument();
  });

  it('renders action button when provided', () => {
    const handleClick = vi.fn();
    render(
      <EmptyState
        title="No items"
        action={{ label: 'Add item', onClick: handleClick }}
      />,
    );
    const button = screen.getByRole('button', { name: 'Add item' });
    expect(button).toBeInTheDocument();
  });

  it('triggers action on button click', async () => {
    const handleClick = vi.fn();
    render(
      <EmptyState
        title="No items"
        action={{ label: 'Add item', onClick: handleClick }}
      />,
    );
    await userEvent.click(screen.getByRole('button', { name: 'Add item' }));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies custom className', () => {
    const { container } = render(
      <EmptyState title="Test" className="custom-class" />,
    );
    const root = container.firstElementChild;
    expect(root).toHaveClass('custom-class');
  });
});
