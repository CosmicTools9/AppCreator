import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import { Label } from './label';

describe('Label', () => {
  it('renders label correctly', () => {
    render(<Label>Label Text</Label>);
    expect(screen.getByText('Label Text')).toBeInTheDocument();
  });

  it('renders as label element', () => {
    render(<Label>Label</Label>);
    expect(screen.getByText('Label').tagName).toBe('LABEL');
  });

  it('associates with input via htmlFor', () => {
    render(
      <>
        <Label htmlFor="input-id">Input Label</Label>
        <input id="input-id" />
      </>
    );
    expect(screen.getByLabelText('Input Label')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<Label className="custom-label">Label</Label>);
    expect(container.firstChild).toHaveClass('custom-label');
  });
});
