import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { PortalCard, PortalCardStatic } from '@/components/portal/PortalCard';

// framer-motion mocked in vitest.setup.ts / vitest.config.ts
describe('PortalCard', () => {
  it('renders children', () => {
    render(<PortalCard>Card content</PortalCard>);
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('applies medium padding by default', () => {
    render(<PortalCard data-testid="card">Content</PortalCard>);
    // Inner content wrapper gets padding class
    const content = screen.getByText('Content').closest('div');
    expect(content).toHaveClass('p-6');
  });

  it('applies small padding when padding=sm', () => {
    render(<PortalCard data-testid="card" padding="sm">Content</PortalCard>);
    const content = screen.getByText('Content').closest('div');
    expect(content).toHaveClass('p-4');
  });

  it('applies large padding when padding=lg', () => {
    render(<PortalCard data-testid="card" padding="lg">Content</PortalCard>);
    const content = screen.getByText('Content').closest('div');
    expect(content).toHaveClass('p-8');
  });

  it('renders header text when header prop provided', () => {
    render(<PortalCard header="Reserve Holdings">Content</PortalCard>);
    expect(screen.getByText('Reserve Holdings')).toBeInTheDocument();
  });

  it('renders headerAction when provided', () => {
    render(
      <PortalCard header="Title" headerAction={<button>Action</button>}>
        Content
      </PortalCard>
    );
    expect(screen.getByRole('button', { name: 'Action' })).toBeInTheDocument();
  });

  it('applies custom className', () => {
    render(<PortalCard data-testid="card" className="custom-class">Content</PortalCard>);
    const card = screen.getByTestId('card');
    expect(card).toHaveClass('custom-class');
  });

  it('renders rounded corners', () => {
    render(<PortalCard data-testid="card">Content</PortalCard>);
    expect(screen.getByTestId('card')).toHaveClass('rounded-2xl');
  });
});

describe('PortalCardStatic', () => {
  it('renders children without motion wrapper', () => {
    render(<PortalCardStatic>Static content</PortalCardStatic>);
    expect(screen.getByText('Static content')).toBeInTheDocument();
  });

  it('renders header when provided', () => {
    render(<PortalCardStatic header="Section Title">Content</PortalCardStatic>);
    expect(screen.getByText('Section Title')).toBeInTheDocument();
  });

  it('applies rounded-2xl class', () => {
    render(
      <div data-testid="wrapper">
        <PortalCardStatic>Content</PortalCardStatic>
      </div>
    );
    const card = screen.getByText('Content').closest('.rounded-2xl');
    expect(card).toBeInTheDocument();
  });
});
