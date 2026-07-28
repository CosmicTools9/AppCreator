import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { ErrorFallback } from './error-boundary';

describe('ErrorFallback', () => {
  const testError = new Error('Something went wrong');

  it('renders with custom title when provided', () => {
    render(
      <ErrorFallback
        error={testError}
        resetErrorBoundary={vi.fn()}
        title="Custom Error Title"
      />,
    );
    expect(screen.getByText('Custom Error Title')).toBeInTheDocument();
  });

  it('shows error message text', () => {
    render(
      <ErrorFallback
        error={testError}
        resetErrorBoundary={vi.fn()}
        title="Error Title"
      />,
    );
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('shows custom description instead of error message when description provided and error has no message', () => {
    const errWithoutMsg = new Error();
    render(
      <ErrorFallback
        error={errWithoutMsg}
        resetErrorBoundary={vi.fn()}
        title="Error"
        description="Custom description text"
      />,
    );
    expect(screen.getByText('Custom description text')).toBeInTheDocument();
  });

  it('calls resetErrorBoundary on retry click', async () => {
    const handleReset = vi.fn();
    render(
      <ErrorFallback
        error={testError}
        resetErrorBoundary={handleReset}
        title="Error"
      />,
    );
    // Retry button text is from i18n mock — returns key 'components.action.retry'
    const retryButton = screen.getByRole('button', { name: /components\.action\.retry/i });
    await userEvent.click(retryButton);
    expect(handleReset).toHaveBeenCalledTimes(1);
  });

  it('shows error details when showDetails is true', () => {
    render(
      <ErrorFallback
        error={testError}
        resetErrorBoundary={vi.fn()}
        title="Error"
        showDetails={true}
      />,
    );
    expect(screen.getByText(/Something went wrong/)).toBeInTheDocument();
  });

  it('renders goHome and refresh buttons when callbacks provided', () => {
    render(
      <ErrorFallback
        error={testError}
        resetErrorBoundary={vi.fn()}
        title="Error"
        onGoHome={vi.fn()}
        onRefresh={vi.fn()}
      />,
    );
    // Button text is from i18n mock — returns the key
    expect(screen.getByRole('button', { name: /components\.error\.backHome/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /components\.error\.refreshPage/i })).toBeInTheDocument();
  });

  it('does not render goHome button when callback not provided', () => {
    render(
      <ErrorFallback
        error={testError}
        resetErrorBoundary={vi.fn()}
        title="Error"
      />,
    );
    expect(screen.queryByRole('button', { name: /components\.error\.backHome/i })).not.toBeInTheDocument();
  });
});
