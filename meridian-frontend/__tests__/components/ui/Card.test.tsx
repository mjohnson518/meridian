import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Card } from '@/components/ui/Card';

describe('Card', () => {
  it('renders children correctly', () => {
    render(<Card>Card content</Card>);
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('applies base styling', () => {
    render(<Card data-testid="card">Content</Card>);
    const card = screen.getByTestId('card');
    expect(card).toHaveClass('bg-white');
    expect(card).toHaveClass('border');
    expect(card).toHaveClass('rounded-xl');
  });

  it('applies hover styles by default', () => {
    render(<Card data-testid="card">Content</Card>);
    const card = screen.getByTestId('card');
    expect(card).toHaveClass('hover-lift', 'cursor-pointer');
  });

  it('disables hover styles when hover is false', () => {
    render(<Card data-testid="card" hover={false}>Content</Card>);
    const card = screen.getByTestId('card');
    expect(card).not.toHaveClass('hover-lift');
    expect(card).not.toHaveClass('cursor-pointer');
  });

  it('applies padding classes correctly', () => {
    const { rerender } = render(<Card data-testid="card" padding="sm">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('p-4');

    rerender(<Card data-testid="card" padding="md">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('p-6');

    rerender(<Card data-testid="card" padding="lg">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('p-8');

    rerender(<Card data-testid="card" padding="xl">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('p-10');
  });

  it('applies default lg padding when not specified', () => {
    render(<Card data-testid="card">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('p-8');
  });

  it('applies custom className', () => {
    render(<Card data-testid="card" className="custom-class">Content</Card>);
    expect(screen.getByTestId('card')).toHaveClass('custom-class');
  });

  it('renders with dark mode classes', () => {
    render(<Card data-testid="card">Content</Card>);
    const card = screen.getByTestId('card');
    expect(card).toHaveClass('dark:bg-black', 'dark:border-gray-800');
  });

  it('can contain complex nested content', () => {
    render(
      <Card>
        <h2>Title</h2>
        <p>Description</p>
        <button>Action</button>
      </Card>
    );
    expect(screen.getByText('Title')).toBeInTheDocument();
    expect(screen.getByText('Description')).toBeInTheDocument();
    expect(screen.getByRole('button')).toHaveTextContent('Action');
  });
});
