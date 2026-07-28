import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import { Avatar, AvatarImage, AvatarFallback } from './avatar';

describe('Avatar', () => {
  it('renders avatar correctly', () => {
    const { container } = render(
      <Avatar>
        <AvatarImage src="/avatar.png" alt="User" />
        <AvatarFallback>UN</AvatarFallback>
      </Avatar>
    );
    expect(container.querySelector('span')).toHaveClass('relative', 'flex', 'h-10', 'w-10');
  });

  it('renders fallback content', () => {
    render(
      <Avatar>
        <AvatarFallback>UN</AvatarFallback>
      </Avatar>
    );
    expect(screen.getByText('UN')).toBeInTheDocument();
  });

  it('applies custom className to Avatar', () => {
    const { container } = render(<Avatar className="custom-avatar" />);
    expect(container.firstChild).toHaveClass('custom-avatar');
  });

  it('applies custom className to AvatarFallback', () => {
    const { container } = render(
      <Avatar>
        <AvatarFallback className="custom-fallback">UN</AvatarFallback>
      </Avatar>
    );
    expect(container.querySelector('.custom-fallback')).toBeInTheDocument();
  });
});
