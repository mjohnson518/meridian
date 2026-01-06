import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MetricCard } from '@/components/ui/MetricCard';

describe('MetricCard', () => {
  it('renders label correctly', () => {
    render(<MetricCard label="Reserve Ratio" value="105%" />);
    expect(screen.getByText('Reserve Ratio')).toBeInTheDocument();
  });

  it('renders value correctly', () => {
    render(<MetricCard label="Reserve Ratio" value="105%" />);
    expect(screen.getByText('105%')).toBeInTheDocument();
  });

  it('renders numeric value correctly', () => {
    render(<MetricCard label="Total Supply" value={1000000} />);
    expect(screen.getByText('1000000')).toBeInTheDocument();
  });

  it('applies up trend styling', () => {
    render(<MetricCard label="Reserve Ratio" value="105%" trend="up" />);
    const value = screen.getByText('105%');
    expect(value).toHaveClass('text-emerald-500');
  });

  it('applies down trend styling', () => {
    render(<MetricCard label="Reserve Ratio" value="95%" trend="down" />);
    const value = screen.getByText('95%');
    expect(value).toHaveClass('text-red-500');
  });

  it('applies neutral trend styling (default color)', () => {
    render(<MetricCard label="Reserve Ratio" value="100%" trend="neutral" />);
    const value = screen.getByText('100%');
    // Should not have trend colors
    expect(value).not.toHaveClass('text-emerald-500');
    expect(value).not.toHaveClass('text-red-500');
  });

  it('applies custom className', () => {
    render(<MetricCard label="Test" value="123" className="custom-metric" />);
    // The custom class should be on the container
    const container = screen.getByText('Test').parentElement;
    expect(container).toHaveClass('custom-metric');
  });

  it('has text-center alignment', () => {
    render(<MetricCard label="Aligned" value="100" />);
    const container = screen.getByText('Aligned').parentElement;
    expect(container).toHaveClass('text-center');
  });

  it('renders label with uppercase styling', () => {
    render(<MetricCard label="Label" value="123" />);
    const label = screen.getByText('Label');
    expect(label).toHaveClass('uppercase');
    expect(label).toHaveClass('tracking-wider');
  });

  it('renders value with monospace font', () => {
    render(<MetricCard label="Test" value="456" />);
    const value = screen.getByText('456');
    expect(value).toHaveClass('font-mono');
  });

  it('renders ReactNode as value', () => {
    render(
      <MetricCard
        label="Complex"
        value={<span data-testid="complex">Complex Value</span>}
      />
    );
    expect(screen.getByTestId('complex')).toBeInTheDocument();
    expect(screen.getByText('Complex Value')).toBeInTheDocument();
  });
});
