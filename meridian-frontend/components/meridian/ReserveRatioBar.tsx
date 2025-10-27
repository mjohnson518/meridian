'use client';

import { cn } from '@/lib/utils';
import { MonoDisplay } from '../sacred/Typography';

interface ReserveRatioBarProps {
  ratio: number;
  threshold?: number;
  showLabels?: boolean;
  className?: string;
}

export const ReserveRatioBar = ({
  ratio,
  threshold = 100,
  showLabels = true,
  className,
}: ReserveRatioBarProps) => {
  const status = ratio >= threshold ? 'healthy' : ratio >= 95 ? 'warning' : 'critical';
  
  const statusColors = {
    healthy: 'bg-sacred-black',
    warning: 'bg-amber-600',
    critical: 'bg-red-600',
  };

  return (
    <div className={cn('w-full', className)}>
      {showLabels && (
        <div className="flex justify-between mb-1">
          <span className="text-xs font-mono uppercase text-sacred-gray-600">
            Reserve Ratio
          </span>
          <MonoDisplay
            value={ratio}
            precision={2}
            suffix="%"
            size="xs"
            color={status === 'healthy' ? 'default' : status === 'warning' ? 'warning' : 'negative'}
          />
        </div>
      )}
      
      <div className="relative h-2 bg-sacred-gray-200 rounded overflow-hidden">
        {/* Threshold indicator */}
        <div
          className="absolute top-0 bottom-0 w-px bg-sacred-gray-400 z-10"
          style={{ left: `${threshold}%` }}
        />
        
        {/* Ratio bar */}
        <div
          className={cn(
            'h-full rounded transition-all duration-500',
            statusColors[status]
          )}
          style={{ width: `${Math.min(ratio, 120)}%` }}
        />
      </div>
      
      {showLabels && (
        <div className="flex justify-between mt-1">
          <span className="text-xxs text-sacred-gray-500 font-mono">0%</span>
          <span className="text-xxs text-sacred-gray-500 font-mono">100%</span>
          <span className="text-xxs text-sacred-gray-500 font-mono">120%</span>
        </div>
      )}
    </div>
  );
};
