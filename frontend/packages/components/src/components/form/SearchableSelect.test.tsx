import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { SearchableSelect } from './SearchableSelect';

beforeAll(() => {
  // Mock scrollIntoView for jsdom
  Element.prototype.scrollIntoView = vi.fn();
});

const options = [
  { value: '1', label: 'Option A' },
  { value: '2', label: 'Option B' },
  { value: '3', label: 'Option C' },
];

describe('SearchableSelect', () => {
  it('renders placeholder when no value', () => {
    render(
      <SearchableSelect options={options} value="" onChange={vi.fn()} />,
    );
    expect(screen.getByText('common.pleaseSelect')).toBeInTheDocument();
  });

  it('renders selected value label', () => {
    render(
      <SearchableSelect options={options} value="2" onChange={vi.fn()} />,
    );
    expect(screen.getByText('Option B')).toBeInTheDocument();
  });

  it('opens dropdown on click', async () => {
    render(
      <SearchableSelect options={options} value="" onChange={vi.fn()} />,
    );
    const trigger = screen.getByRole('combobox');
    await userEvent.click(trigger);
    expect(screen.getByText('Option A')).toBeInTheDocument();
    expect(screen.getByText('Option C')).toBeInTheDocument();
  });

  it('calls onChange when option selected', async () => {
    const handleChange = vi.fn();
    render(
      <SearchableSelect options={options} value="" onChange={handleChange} />,
    );
    await userEvent.click(screen.getByRole('combobox'));
    await userEvent.click(screen.getByText('Option B'));
    expect(handleChange).toHaveBeenCalledWith('2');
  });

  it('supports disabled state', () => {
    render(
      <SearchableSelect options={options} value="" onChange={vi.fn()} disabled={true} />,
    );
    const combo = screen.getByRole('combobox');
    expect(combo).toBeDisabled();
  });
});
