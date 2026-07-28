import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { ApprovalCard } from './ApprovalCard';
import type { ApprovalItem } from './types';

const baseItem: ApprovalItem = {
  id: 1,
  title: 'Purchase Request #123',
  applicant: 'Alice Wang',
  dept: 'Engineering',
  code: 'PR-2024-001',
  status: 'pending',
  time: '10 min ago',
};

describe('ApprovalCard', () => {
  it('renders title and applicant', () => {
    render(<ApprovalCard item={baseItem} />);
    expect(screen.getByText('Purchase Request #123')).toBeInTheDocument();
    expect(screen.getByText('Alice Wang')).toBeInTheDocument();
  });

  it('renders department when provided', () => {
    render(<ApprovalCard item={baseItem} />);
    expect(screen.getByText('Engineering')).toBeInTheDocument();
  });

  it('renders code when provided', () => {
    render(<ApprovalCard item={baseItem} />);
    expect(screen.getByText('PR-2024-001')).toBeInTheDocument();
  });

  it('shows approve and reject buttons for pending status', () => {
    render(<ApprovalCard item={baseItem} />);
    // i18n mock returns the key as text
    expect(screen.getByText('components.approval.action.approve')).toBeInTheDocument();
    expect(screen.getByText('components.approval.action.reject')).toBeInTheDocument();
  });

  it('does not show action buttons for non-pending status', () => {
    const approvedItem: ApprovalItem = { ...baseItem, status: 'approved' };
    render(<ApprovalCard item={approvedItem} />);
    expect(screen.queryByText('components.approval.action.approve')).not.toBeInTheDocument();
    expect(screen.queryByText('components.approval.action.reject')).not.toBeInTheDocument();
  });

  it('calls onApprove when approve button clicked', async () => {
    const handleApprove = vi.fn();
    render(<ApprovalCard item={baseItem} onApprove={handleApprove} />);
    await userEvent.click(screen.getByText('components.approval.action.approve'));
    expect(handleApprove).toHaveBeenCalledWith(1);
  });

  it('calls onReject when reject button clicked', async () => {
    const handleReject = vi.fn();
    render(<ApprovalCard item={baseItem} onReject={handleReject} />);
    await userEvent.click(screen.getByText('components.approval.action.reject'));
    expect(handleReject).toHaveBeenCalledWith(1);
  });

  it('calls onClick when card body is clicked', async () => {
    const handleClick = vi.fn();
    render(<ApprovalCard item={baseItem} onClick={handleClick} />);
    await userEvent.click(screen.getByText('Purchase Request #123'));
    expect(handleClick).toHaveBeenCalledWith(baseItem);
  });
});
