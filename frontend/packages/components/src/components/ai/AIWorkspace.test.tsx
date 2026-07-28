import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';

// Mock AIChatPanel — render a simple div with data-testid to verify props are passed
vi.mock('./AIChatPanel', () => ({
  AIChatPanel: vi.fn(({ agentCode, onAgentChange, open, docked, ...props }: any) => (
    <div data-testid="ai-chat-panel" data-agent={agentCode} data-open={open} data-docked={docked}>
      AIChatPanel Mock
    </div>
  )),
  PageContext: {},
}));

// Mock page-context
vi.mock('./page-context', () => ({
  pageContextModule: {
    snapshot: () => ({ raw: null }),
    renderForAgent: () => ({ text: '' }),
  },
}));

// Mock ai-context
vi.mock('./ai-context', () => ({
  aiContextAtom: { init: { registeredAt: null } },
}));

// Mock workspace atoms
vi.mock('../workspace/workspace-atoms', () => ({
  closeWorkspaceAtom: { debugLabel: 'closeWorkspaceAtom', init: null },
  openWorkspaceAtom: { debugLabel: 'openWorkspaceAtom', init: null },
  activeWorkspaceAtom: { debugLabel: 'activeWorkspaceAtom', init: null },
}));

// Mock jotai
vi.mock('jotai', () => ({
  useAtom: (atom: any) => {
    if (atom?.debugLabel === 'closeWorkspaceAtom') return [null, vi.fn()];
    if (atom?.init && 'registeredAt' in atom.init) return [{ registeredAt: null }, vi.fn()];
    return [null, vi.fn()];
  },
  atom: (initialValue: any) => ({ init: initialValue }),
}));

import { AIWorkspace } from './AIWorkspace';

describe('AIWorkspace', () => {
  it('renders AIChatPanel', () => {
    render(<AIWorkspace />);
    expect(screen.getByTestId('ai-chat-panel')).toBeInTheDocument();
  });

  it('passes agentCode to AIChatPanel', () => {
    render(<AIWorkspace agentCode="sales-agent" />);
    const panel = screen.getByTestId('ai-chat-panel');
    expect(panel.getAttribute('data-agent')).toBe('sales-agent');
  });

  it('renders with docked=true and open=true', () => {
    render(<AIWorkspace />);
    const panel = screen.getByTestId('ai-chat-panel');
    expect(panel.getAttribute('data-docked')).toBe('true');
    expect(panel.getAttribute('data-open')).toBe('true');
  });
});
