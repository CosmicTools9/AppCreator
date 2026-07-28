import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';

// Mock jotai
vi.mock('jotai', () => ({
  useAtom: (atom: any) => {
    // Return [value, setter] — value is the initial atom value
    if (atom.debugLabel === 'activeWorkspaceAtom') return [null, vi.fn()];
    if (atom.debugLabel === 'toggleWorkspaceAtom') return [null, vi.fn()];
    return [null, vi.fn()];
  },
  atom: (initialValue: any) => ({
    init: initialValue,
    debugLabel: undefined,
  }),
}));

// Mock workspace atoms
vi.mock('./workspace-atoms', () => ({
  activeWorkspaceAtom: { debugLabel: 'activeWorkspaceAtom', init: null },
  toggleWorkspaceAtom: { debugLabel: 'toggleWorkspaceAtom', init: null },
  closeWorkspaceAtom: { debugLabel: 'closeWorkspaceAtom', init: null },
  WorkspaceId: {},
}));

import { WorkspaceTrigger } from './WorkspaceTrigger';
import { ActivityIcon } from 'lucide-react';

describe('WorkspaceTrigger', () => {
  it('renders with icon', () => {
    render(
      <WorkspaceTrigger id="inbox" icon={<ActivityIcon data-testid="icon" />} />,
    );
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('renders pending count badge', () => {
    render(
      <WorkspaceTrigger id="inbox" icon={<ActivityIcon />} pendingCount={5} />,
    );
    expect(screen.getByText('5')).toBeInTheDocument();
  });

  it('renders with title as aria-label', () => {
    render(
      <WorkspaceTrigger id="inbox" icon={<ActivityIcon />} title="Inbox Messages" />,
    );
    expect(screen.getByRole('button', { name: 'Inbox Messages' })).toBeInTheDocument();
  });
});
