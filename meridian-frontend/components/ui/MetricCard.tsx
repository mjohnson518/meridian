'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface MetricCardProps {
  label: string;
  value: ReactNode;
  trend?: 'up' | 'down' | 'neutral';
  className?: string;
}

export function MetricCard({ label, value, trend, className }: MetricCardProps) {
  return (
    <div className={cn('text-center', className)}>
      <p className="text-xs font-medium uppercase tracking-wider text-gray-600 dark:text-gray-400 mb-3">
        {label}
      </p>
      <div className={cn(
        'font-mono text-4xl md:text-5xl lg:text-6xl tabular-nums font-bold text-black dark:text-white',
        trend === 'up' && 'text-emerald-500',
        trend === 'down' && 'text-red-500'
      )}>
        {value}
      </div>
    </div>
  );
}

