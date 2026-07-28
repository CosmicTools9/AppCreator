import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import { Footer } from './Footer';

describe('Footer', () => {
  it('renders copyright text', () => {
    render(<Footer copyright="© 2026 AliothStudio" />);
    expect(screen.getByText('© 2026 AliothStudio')).toBeInTheDocument();
  });

  it('renders navigation links', () => {
    render(
      <Footer
        copyright="© 2026"
        links={[
          { label: 'About', href: '/about' },
          { label: 'Privacy', href: '/privacy' },
        ]}
      />,
    );
    const aboutLink = screen.getByRole('link', { name: 'About' });
    expect(aboutLink).toHaveAttribute('href', '/about');
    expect(screen.getByRole('link', { name: 'Privacy' })).toBeInTheDocument();
  });

  it('renders version text', () => {
    render(<Footer copyright="© 2026" version="v1.0.0" />);
    expect(screen.getByText('v1.0.0')).toBeInTheDocument();
  });

  it('does not render copyright when not provided', () => {
    render(<Footer />);
    // Copyright text should not be rendered
    expect(screen.queryByText(/©/)).not.toBeInTheDocument();
  });

  it('does not render version when not provided', () => {
    render(<Footer copyright="© 2026" />);
    expect(screen.queryByText(/v\d/)).not.toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(
      <Footer copyright="© 2026" className="custom-footer" />,
    );
    const footer = container.querySelector('footer');
    expect(footer).toHaveClass('custom-footer');
  });
});
