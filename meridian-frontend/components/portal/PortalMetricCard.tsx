'use client';

import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';
import { formatCurrency, formatPercentage, formatCompactNumber } from '@/lib/utils';
import { ReactNode } from 'react';

type MetricStatus = 'healthy' | 'warning' | 'critical' | 'neutral';
type MetricFormat = 'currency' | 'percentage' | 'number' | 'compact';

interface PortalMetricCardProps {
  label: string;
  value: string | number;
  format?: MetricFormat;
  status?: MetricStatus;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  icon?: ReactNode;
  className?: string;
}

const statusColors: Record<MetricStatus, { border: string; glow: string; text: string }> = {
  healthy: {
    border: 'border-emerald-200 dark:border-emerald-500/30',
    glow: 'shadow-emerald-500/10 dark:shadow-[0_0_20px_-5px_rgba(16,185,129,0.3)]',
    text: 'text-emerald-600 dark:text-emerald-400',
  },
  warning: {
    border: 'border-amber-200 dark:border-amber-500/30',
    glow: 'shadow-amber-500/10 dark:shadow-[0_0_20px_-5px_rgba(245,158,11,0.3)]',
    text: 'text-amber-600 dark:text-amber-400',
  },
  critical: {
    border: 'border-red-200 dark:border-red-500/30',
    glow: 'shadow-red-500/10 dark:shadow-[0_0_20px_-5px_rgba(239,68,68,0.3)]',
    text: 'text-red-600 dark:text-red-400',
  },
  neutral: {
    border: 'border-gray-200 dark:border-white/10',
    glow: '',
    text: 'text-gray-500 dark:text-gray-400',
  },
};

const trendIcons = {
  up: (
    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
    </svg>
  ),
  down: (
    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
    </svg>
  ),
  neutral: (
    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14" />
    </svg>
  ),
};

function formatValue(value: string | number, format?: MetricFormat): string {
  if (typeof value === 'string') return value;

  switch (format) {
    case 'currency':
      return formatCurrency(value);
    case 'percentage':
      return formatPercentage(value);
    case 'compact':
      return formatCompactNumber(value);
    case 'number':
    default:
      return value.toLocaleString();
  }
}

export function PortalMetricCard({
  label,
  value,
  format,
  status = 'neutral',
  trend,
  trendValue,
  icon,
  className,
}: PortalMetricCardProps) {
  const statusStyle = statusColors[status];

  return (
    <motion.div
      className={cn(
        "relative overflow-hidden rounded-2xl p-6",
        // Light mode: white background
        "bg-white",
        // Dark mode: subtle dark background
        "dark:bg-gray-900/80",
        "backdrop-blur-xl",
        "border shadow-sm",
        statusStyle.border,
        statusStyle.glow,
        "transition-all duration-300",
        "hover:bg-gray-50 dark:hover:bg-gray-900",
        className
      )}
      whileHover={{ y: -2 }}
      transition={{ duration: 0.2 }}
    >
      {/* Gradient overlay - dark mode only */}
      <div className="absolute inset-0 bg-gradient-to-br from-white/[0.02] to-transparent pointer-events-none hidden dark:block" />

      <div className="relative z-10">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <span className="text-xs font-mono uppercase tracking-wider text-gray-500 dark:text-gray-500">
            {label}
          </span>
          {icon && (
            <div className={cn("p-2 rounded-lg bg-gray-100 dark:bg-white/5", statusStyle.text)}>
              {icon}
            </div>
          )}
        </div>

        {/* Value */}
        <div className="flex items-end justify-between">
          <motion.div
            className="font-mono text-3xl font-bold text-gray-900 dark:text-white tabular-nums"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3 }}
          >
            {formatValue(value, format)}
          </motion.div>

          {/* Trend */}
          {trend && (
            <div
              className={cn(
                "flex items-center gap-1 text-xs font-mono",
                trend === 'up' && "text-emerald-600 dark:text-emerald-400",
                trend === 'down' && "text-red-600 dark:text-red-400",
                trend === 'neutral' && "text-gray-500"
              )}
            >
              {trendIcons[trend]}
              {trendValue && <span>{trendValue}</span>}
            </div>
          )}
        </div>

        {/* Status indicator dot */}
        {status !== 'neutral' && (
          <div className="absolute top-4 right-4">
            <div
              className={cn(
                "w-2 h-2 rounded-full",
                status === 'healthy' && "bg-emerald-500",
                status === 'warning' && "bg-amber-500",
                status === 'critical' && "bg-red-500 animate-pulse"
              )}
            />
          </div>
        )}
      </div>
    </motion.div>
  );
}

// Grid wrapper for multiple metric cards
interface PortalMetricGridProps {
  children: ReactNode;
  columns?: 2 | 3 | 4;
  className?: string;
}

export function PortalMetricGrid({ children, columns = 4, className }: PortalMetricGridProps) {
  const colClasses = {
    2: 'grid-cols-1 sm:grid-cols-2',
    3: 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3',
    4: 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-4',
  };

  return (
    <div className={cn('grid gap-4', colClasses[columns], className)}>
      {children}
    </div>
  );
}
