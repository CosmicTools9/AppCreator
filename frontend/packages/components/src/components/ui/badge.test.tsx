import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import { Badge } from './badge';

describe('Badge', () => {
  it('renders with default variant', () => {
    const { container } = render(<Badge>Default Badge</Badge>);
    expect(container.querySelector('.bg-primary')).toBeInTheDocument();
    expect(screen.getByText('Default Badge')).toBeInTheDocument();
  });

  it('renders with secondary variant', () => {
    const { container } = render(<Badge variant="secondary">Secondary</Badge>);
    expect(container.querySelector('.bg-secondary')).toBeInTheDocument();
  });

  it('renders with destructive variant', () => {
    const { container } = render(<Badge variant="destructive">Destructive</Badge>);
    expect(container.querySelector('.bg-destructive')).toBeInTheDocument();
  });

  it('renders with outline variant', () => {
    const { container } = render(<Badge variant="outline">Outline</Badge>);
    expect(container.querySelector('.border')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<Badge className="custom-badge">Badge</Badge>);
    expect(container.firstChild).toHaveClass('custom-badge');
  });

  it('renders with correct element type', () => {
    const { container } = render(<Badge>Badge</Badge>);
    expect(container.querySelector('div')).toBeInTheDocument();
  });
});
