'use client';

import { cn } from '@/lib/utils';

type BadgeStatus = 'success' | 'warning' | 'error' | 'pending' | 'neutral' | 'info';
type BadgeSize = 'sm' | 'md';

interface PortalStatusBadgeProps {
  status: BadgeStatus;
  label: string;
  size?: BadgeSize;
  pulse?: boolean;
  className?: string;
}

const statusStyles: Record<BadgeStatus, { bg: string; text: string; dot: string; glow?: string }> = {
  success: {
    bg: 'bg-emerald-500/10',
    text: 'text-emerald-400',
    dot: 'bg-emerald-500',
    glow: 'shadow-[0_0_8px_rgba(16,185,129,0.4)]',
  },
  warning: {
    bg: 'bg-amber-500/10',
    text: 'text-amber-400',
    dot: 'bg-amber-500',
    glow: 'shadow-[0_0_8px_rgba(245,158,11,0.4)]',
  },
  error: {
    bg: 'bg-red-500/10',
    text: 'text-red-400',
    dot: 'bg-red-500',
    glow: 'shadow-[0_0_8px_rgba(239,68,68,0.4)]',
  },
  pending: {
    bg: 'bg-blue-500/10',
    text: 'text-blue-400',
    dot: 'bg-blue-500',
  },
  info: {
    bg: 'bg-cyan-500/10',
    text: 'text-cyan-400',
    dot: 'bg-cyan-500',
  },
  neutral: {
    bg: 'bg-gray-500/10',
    text: 'text-gray-400',
    dot: 'bg-gray-500',
  },
};

const sizeClasses = {
  sm: {
    container: 'px-2 py-1 text-xs',
    dot: 'w-1.5 h-1.5',
  },
  md: {
    container: 'px-3 py-1.5 text-xs',
    dot: 'w-2 h-2',
  },
};

export function PortalStatusBadge({
  status,
  label,
  size = 'sm',
  pulse = false,
  className,
}: PortalStatusBadgeProps) {
  const style = statusStyles[status];
  const sizeStyle = sizeClasses[size];

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full font-mono uppercase tracking-wider",
        style.bg,
        style.text,
        sizeStyle.container,
        className
      )}
    >
      <span
        className={cn(
          "rounded-full",
          style.dot,
          style.glow,
          sizeStyle.dot,
          pulse && "animate-pulse"
        )}
      />
      {label}
    </span>
  );
}

// Simple dot indicator without text
interface PortalStatusDotProps {
  status: BadgeStatus;
  pulse?: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const dotSizes = {
  sm: 'w-1.5 h-1.5',
  md: 'w-2 h-2',
  lg: 'w-3 h-3',
};

export function PortalStatusDot({ status, pulse = false, size = 'md', className }: PortalStatusDotProps) {
  const style = statusStyles[status];

  return (
    <span
      className={cn(
        "rounded-full inline-block",
        style.dot,
        style.glow,
        dotSizes[size],
        pulse && "animate-pulse",
        className
      )}
    />
  );
}
