'use client';

import { cn } from '@/lib/utils';

interface SkeletonProps {
  className?: string;
  variant?: 'text' | 'circular' | 'rectangular' | 'rounded';
  width?: string | number;
  height?: string | number;
  animation?: 'pulse' | 'wave' | 'none';
}

export function Skeleton({
  className,
  variant = 'text',
  width,
  height,
  animation = 'pulse',
}: SkeletonProps) {
  const baseClasses = 'bg-gray-200 dark:bg-gray-700';

  const animationClasses = {
    pulse: 'animate-pulse',
    wave: 'animate-shimmer bg-gradient-to-r from-gray-200 via-gray-100 to-gray-200 dark:from-gray-700 dark:via-gray-600 dark:to-gray-700 bg-[length:200%_100%]',
    none: '',
  };

  const variantClasses = {
    text: 'rounded h-4 w-full',
    circular: 'rounded-full',
    rectangular: '',
    rounded: 'rounded-lg',
  };

  const style: React.CSSProperties = {};
  if (width) style.width = typeof width === 'number' ? `${width}px` : width;
  if (height) style.height = typeof height === 'number' ? `${height}px` : height;

  return (
    <div
      className={cn(
        baseClasses,
        animationClasses[animation],
        variantClasses[variant],
        className
      )}
      style={style}
      aria-hidden="true"
    />
  );
}

interface SkeletonTextProps {
  lines?: number;
  className?: string;
  lastLineWidth?: string;
}

export function SkeletonText({
  lines = 3,
  className,
  lastLineWidth = '60%',
}: SkeletonTextProps) {
  return (
    <div className={cn('space-y-2', className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          variant="text"
          className={i === lines - 1 ? '' : 'w-full'}
          width={i === lines - 1 ? lastLineWidth : undefined}
        />
      ))}
    </div>
  );
}

interface SkeletonCardProps {
  className?: string;
  hasImage?: boolean;
  lines?: number;
}

export function SkeletonCard({
  className,
  hasImage = false,
  lines = 3,
}: SkeletonCardProps) {
  return (
    <div className={cn('p-4 border border-gray-200 dark:border-gray-700 rounded-lg', className)}>
      {hasImage && (
        <Skeleton variant="rounded" className="w-full h-48 mb-4" />
      )}
      <Skeleton variant="text" className="h-6 w-3/4 mb-4" />
      <SkeletonText lines={lines} />
    </div>
  );
}

// Sacred design system skeleton variants
interface SacredSkeletonProps {
  className?: string;
}

export function SacredMetricSkeleton({ className }: SacredSkeletonProps) {
  return (
    <div className={cn('space-y-2', className)}>
      <Skeleton variant="text" className="h-3 w-24 bg-sacred-gray-200" />
      <Skeleton variant="text" className="h-8 w-32 bg-sacred-gray-200" />
    </div>
  );
}

export function SacredCardSkeleton({ className }: SacredSkeletonProps) {
  return (
    <div className={cn('p-6 bg-sacred-white border border-sacred-gray-200 rounded', className)}>
      <Skeleton variant="text" className="h-5 w-40 mb-4 bg-sacred-gray-200" />
      <div className="space-y-3">
        <Skeleton variant="text" className="h-4 w-full bg-sacred-gray-200" />
        <Skeleton variant="text" className="h-4 w-full bg-sacred-gray-200" />
        <Skeleton variant="text" className="h-4 w-3/4 bg-sacred-gray-200" />
      </div>
    </div>
  );
}
