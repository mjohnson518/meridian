'use client';

import { cn } from '@/lib/utils';
import { MonoDisplay, Label } from '../sacred/Typography';
import { formatTimeAgo, formatCompactNumber, formatTimestamp } from '@/lib/utils';

interface MetricDisplayProps {
  label: string;
  value: number | string | Date;
  format?: 'currency' | 'percentage' | 'number' | 'compact' | 'timeago' | 'timestamp';
  precision?: number;
  trend?: number;
  threshold?: number;
  status?: 'healthy' | 'warning' | 'critical';
  className?: string;
}

export const MetricDisplay = ({
  label,
  value,
  format = 'number',
  precision = 2,
  trend,
  threshold,
  status,
  className,
}: MetricDisplayProps) => {
  const formatValue = () => {
    switch (format) {
      case 'currency':
        return (
          <MonoDisplay
            value={typeof value === 'number' || typeof value === 'string' ? value : 0}
            precision={precision}
            currency="USD"
            size="xl"
          />
        );
      case 'percentage':
        return (
          <MonoDisplay
            value={typeof value === 'number' || typeof value === 'string' ? value : 0}
            precision={precision}
            suffix="%"
            size="xl"
            color={
              threshold && typeof value === 'number'
                ? value >= threshold
                  ? 'positive'
                  : 'negative'
                : 'default'
            }
          />
        );
      case 'compact':
        return (
          <span className="font-mono text-xl tabular-nums">
            {formatCompactNumber(typeof value === 'number' || typeof value === 'string' ? value : 0)}
          </span>
        );
      case 'timeago':
        return (
          <span className="font-mono text-xl">
            {formatTimeAgo(value as number | Date)}
          </span>
        );
      case 'timestamp':
        return (
          <span className="font-mono text-lg">
            {formatTimestamp(value as number | Date)}
          </span>
        );
      default:
        return (
          <MonoDisplay
            value={typeof value === 'number' || typeof value === 'string' ? value : 0}
            precision={0}
            size="xl"
          />
        );
    }
  };

  const getStatusColor = () => {
    if (status) {
      switch (status) {
        case 'healthy':
          return 'text-emerald-600';
        case 'warning':
          return 'text-amber-600';
        case 'critical':
          return 'text-red-600';
      }
    }
    return '';
  };

  return (
    <div className={cn('space-y-2', className)}>
      <Label>{label}</Label>
      
      <div className="flex items-baseline justify-between">
        <div className={cn(getStatusColor())}>
          {formatValue()}
        </div>
        
        {trend !== undefined && (
          <div
            className={cn(
              'flex items-center text-xs font-mono',
              trend > 0 ? 'text-emerald-600' : trend < 0 ? 'text-red-600' : 'text-sacred-gray-500'
            )}
          >
            {trend > 0 ? '↑' : trend < 0 ? '↓' : '→'}
            <span className="ml-1">
              {Math.abs(trend).toFixed(2)}%
            </span>
          </div>
        )}
      </div>
      
      {status && (
        <div className="flex items-center space-x-1">
          <div
            className={cn(
              'w-2 h-2 rounded-full',
              status === 'healthy' && 'bg-emerald-600',
              status === 'warning' && 'bg-amber-600',
              status === 'critical' && 'bg-red-600'
            )}
          />
          <span className="text-xs font-mono text-sacred-gray-600 uppercase">
            {status}
          </span>
        </div>
      )}
    </div>
  );
};
