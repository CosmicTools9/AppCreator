import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { TodoList } from './TodoList';
import type { TodoItem } from './types';

const mockItems: TodoItem[] = [
  {
    id: 1,
    title: 'Review purchase order',
    subject: 'Alice',
    objects: [{ id: 101, name: 'PO-2026-001', type: 'bill' }],
    dueDate: '2026-06-10',
    done: false,
  },
  {
    id: 2,
    title: 'Approve design spec',
    objects: [{ id: 102, name: 'DS-001', type: 'production' }],
    done: true,
  },
  {
    id: 3,
    title: 'Simple task',
    objects: [],
    done: false,
  },
];

describe('TodoList', () => {
  it('renders all todo items', () => {
    render(<TodoList items={mockItems} />);
    expect(screen.getByText('Review purchase order')).toBeInTheDocument();
    expect(screen.getByText('Approve design spec')).toBeInTheDocument();
    expect(screen.getByText('Simple task')).toBeInTheDocument();
  });

  it('renders object names', () => {
    render(<TodoList items={mockItems} />);
    expect(screen.getByText('PO-2026-001')).toBeInTheDocument();
    expect(screen.getByText('DS-001')).toBeInTheDocument();
  });

  // NOTE: dueDate is a field on the model but not rendered in this component version

  it('renders i18n assignee label for items with subject', () => {
    render(<TodoList items={mockItems} />);
    // Subject is rendered via t('components.todoList.assignee', {subject})
    // With our i18n mock, this returns just the key
    expect(screen.getByText('components.todoList.assignee')).toBeInTheDocument();
  });

  it('calls onToggle when checkbox clicked', async () => {
    const handleToggle = vi.fn();
    render(<TodoList items={mockItems} onToggle={handleToggle} />);
    const checkbox = screen.getByRole('checkbox', { name: 'Review purchase order' });
    await userEvent.click(checkbox);
    expect(handleToggle).toHaveBeenCalledWith(1);
  });
});
