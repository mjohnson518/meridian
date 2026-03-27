import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { PortalButton } from '@/components/portal/PortalButton';

describe('PortalButton', () => {
  it('renders children', () => {
    render(<PortalButton>Submit</PortalButton>);
    expect(screen.getByRole('button')).toHaveTextContent('Submit');
  });

  it('applies primary variant by default', () => {
    render(<PortalButton>Primary</PortalButton>);
    const btn = screen.getByRole('button');
    expect(btn).toHaveClass('from-emerald-500');
  });

  it('applies secondary variant styles', () => {
    render(<PortalButton variant="secondary">Secondary</PortalButton>);
    const btn = screen.getByRole('button');
    expect(btn).toHaveClass('bg-gray-100');
  });

  it('applies danger variant styles', () => {
    render(<PortalButton variant="danger">Delete</PortalButton>);
    const btn = screen.getByRole('button');
    expect(btn).toHaveClass('from-red-500');
  });

  it('applies ghost variant styles', () => {
    render(<PortalButton variant="ghost">Cancel</PortalButton>);
    const btn = screen.getByRole('button');
    expect(btn).toHaveClass('bg-transparent');
  });

  it('applies size classes for sm', () => {
    render(<PortalButton size="sm">Small</PortalButton>);
    expect(screen.getByRole('button')).toHaveClass('px-4', 'py-2', 'text-xs');
  });

  it('applies size classes for md (default)', () => {
    render(<PortalButton size="md">Medium</PortalButton>);
    expect(screen.getByRole('button')).toHaveClass('px-6', 'py-3', 'text-sm');
  });

  it('applies size classes for lg', () => {
    render(<PortalButton size="lg">Large</PortalButton>);
    expect(screen.getByRole('button')).toHaveClass('px-8', 'py-4', 'text-base');
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<PortalButton onClick={handleClick}>Click me</PortalButton>);
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('is disabled when disabled prop is set', () => {
    const handleClick = vi.fn();
    render(<PortalButton disabled onClick={handleClick}>Disabled</PortalButton>);
    const btn = screen.getByRole('button');
    expect(btn).toBeDisabled();
    fireEvent.click(btn);
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('shows loading spinner and is disabled when loading', () => {
    render(<PortalButton loading>Loading</PortalButton>);
    const btn = screen.getByRole('button');
    expect(btn).toBeDisabled();
    expect(btn).toHaveAttribute('aria-busy', 'true');
    // SVG spinner should be present
    expect(btn.querySelector('svg')).toBeInTheDocument();
  });

  it('renders leftIcon when provided', () => {
    render(
      <PortalButton leftIcon={<span data-testid="left-icon" />}>Button</PortalButton>
    );
    expect(screen.getByTestId('left-icon')).toBeInTheDocument();
  });

  it('renders rightIcon when provided', () => {
    render(
      <PortalButton rightIcon={<span data-testid="right-icon" />}>Button</PortalButton>
    );
    expect(screen.getByTestId('right-icon')).toBeInTheDocument();
  });

  it('applies fullWidth class when fullWidth is set', () => {
    render(<PortalButton fullWidth>Full Width</PortalButton>);
    expect(screen.getByRole('button')).toHaveClass('w-full');
  });

  it('applies custom className', () => {
    render(<PortalButton className="custom-class">Button</PortalButton>);
    expect(screen.getByRole('button')).toHaveClass('custom-class');
  });

  it('has focus ring for accessibility', () => {
    render(<PortalButton>Focusable</PortalButton>);
    expect(screen.getByRole('button')).toHaveClass('focus:ring-2');
  });
});
