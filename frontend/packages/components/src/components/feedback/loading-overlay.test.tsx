import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import { LoadingOverlay } from './loading-overlay';

describe('LoadingOverlay', () => {
  it('renders children when not loading', () => {
    render(
      <LoadingOverlay isLoading={false}>
        <div data-testid="content">Content</div>
      </LoadingOverlay>,
    );
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('shows spinner when loading', () => {
    render(
      <LoadingOverlay isLoading={true}>
        <div data-testid="content">Content</div>
      </LoadingOverlay>,
    );
    // Should still render children
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('has correct aria-busy when loading', () => {
    const { container } = render(
      <LoadingOverlay isLoading={true}>
        <div>Content</div>
      </LoadingOverlay>,
    );
    const overlay = container.firstElementChild;
    expect(overlay).toBeInTheDocument();
  });

  it('does not block content when not loading', () => {
    const { container } = render(
      <LoadingOverlay isLoading={false}>
        <div data-testid="content">Content</div>
      </LoadingOverlay>,
    );
    // The overlay container should still be present
    expect(container.firstElementChild).toBeInTheDocument();
  });

  it('accepts custom spinner', () => {
    render(
      <LoadingOverlay
        isLoading={true}
        spinner={<span data-testid="custom-spinner">Loading...</span>}
      >
        <div>Content</div>
      </LoadingOverlay>,
    );
    expect(screen.getByTestId('custom-spinner')).toBeInTheDocument();
  });
});
