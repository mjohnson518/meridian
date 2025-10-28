'use client';

import React from 'react';
import { cn } from '@/lib/utils';
import { formatWithPrecision, formatCurrency, formatPercentage } from '@/lib/utils';

interface MonoDisplayProps {
  value: number | string;
  precision?: number;
  currency?: string;
  suffix?: string;
  align?: 'left' | 'right' | 'center';
  className?: string;
  size?: 'xs' | 'sm' | 'base' | 'lg' | 'xl' | '2xl';
  color?: 'default' | 'positive' | 'negative' | 'warning';
}

export const MonoDisplay = ({
  value,
  precision = 2,
  currency,
  suffix,
  align = 'right',
  className,
  size = 'base',
  color = 'default',
}: MonoDisplayProps) => {
  const formatted = currency
    ? formatCurrency(value, currency, precision)
    : formatWithPrecision(value, precision);

  const sizeClasses = {
    xs: 'text-xs',
    sm: 'text-sm',
    base: 'text-base',
    lg: 'text-lg',
    xl: 'text-xl',
    '2xl': 'text-2xl',
  };

  const colorClasses = {
    default: 'text-sacred-black',
    positive: 'text-emerald-600',
    negative: 'text-red-600',
    warning: 'text-amber-600',
  };

  const alignClasses = {
    left: 'text-left',
    right: 'text-right',
    center: 'text-center',
  };

  return (
    <span
      className={cn(
        'font-mono tabular-nums tracking-tightest',
        sizeClasses[size],
        colorClasses[color],
        alignClasses[align],
        className
      )}
    >
      {formatted}
      {suffix && <span className="text-sacred-gray-500 ml-0.5">{suffix}</span>}
    </span>
  );
};

interface HeadingProps {
  level?: 1 | 2 | 3 | 4 | 5 | 6;
  children: React.ReactNode;
  className?: string;
  mono?: boolean;
}

export const Heading = ({ level = 2, children, className, mono = false }: HeadingProps) => {
  const Tag = `h${level}` as keyof JSX.IntrinsicElements;
  
  return (
    <Tag
      className={cn(
        'font-medium leading-tight tracking-tight',
        mono && 'font-mono',
        className
      )}
    >
      {children}
    </Tag>
  );
};

interface LabelProps {
  children: React.ReactNode;
  htmlFor?: string;
  required?: boolean;
  className?: string;
}

export const Label = ({ children, htmlFor, required, className }: LabelProps) => {
  return (
    <label
      htmlFor={htmlFor}
      className={cn(
        'text-xs font-mono uppercase tracking-widest text-sacred-gray-600',
        className
      )}
    >
      {children}
      {required && <span className="text-red-600 ml-1">*</span>}
    </label>
  );
};
