'use client';

import { motion, HTMLMotionProps } from 'framer-motion';
import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface PortalButtonProps extends Omit<HTMLMotionProps<'button'>, 'children'> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline';
  size?: 'sm' | 'md' | 'lg';
  children: ReactNode;
  className?: string;
  loading?: boolean;
  fullWidth?: boolean;
  leftIcon?: ReactNode;
  rightIcon?: ReactNode;
}

function LoadingSpinner({ className }: { className?: string }) {
  return (
    <svg
      className={cn('animate-spin h-4 w-4', className)}
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>
  );
}

export function PortalButton({
  variant = 'primary',
  size = 'md',
  children,
  className,
  loading = false,
  fullWidth = false,
  leftIcon,
  rightIcon,
  disabled,
  ...props
}: PortalButtonProps) {
  const baseClasses = cn(
    "inline-flex items-center justify-center gap-2",
    "font-medium rounded-full",
    "transition-all duration-200",
    "focus:outline-none focus:ring-2 focus:ring-offset-2",
    "focus:ring-offset-white dark:focus:ring-offset-gray-950",
    "disabled:opacity-50 disabled:cursor-not-allowed"
  );

  const variantClasses = {
    primary: cn(
      "bg-gradient-to-r from-emerald-500 to-teal-500 text-white",
      "shadow-lg shadow-emerald-500/25",
      "hover:shadow-xl hover:shadow-emerald-500/30",
      "dark:shadow-[0_0_20px_-5px_rgba(16,185,129,0.5)]",
      "dark:hover:shadow-[0_0_30px_-5px_rgba(16,185,129,0.6)]",
      "focus:ring-emerald-500"
    ),
    secondary: cn(
      // Light mode: visible gray background
      "bg-gray-100 text-gray-700 border border-gray-200",
      "hover:bg-gray-200 hover:border-emerald-500/30",
      // Dark mode: solid dark background for visibility
      "dark:bg-gray-800/80 dark:text-white dark:border-gray-700",
      "dark:hover:bg-gray-700/80 dark:hover:border-emerald-500/30",
      "focus:ring-emerald-500"
    ),
    outline: cn(
      // Light mode
      "bg-transparent text-gray-700 border border-gray-300",
      "hover:bg-gray-50 hover:border-emerald-500/50",
      // Dark mode: more visible border
      "dark:text-white dark:border-gray-600",
      "dark:hover:bg-gray-800/50 dark:hover:border-emerald-400/50",
      "dark:hover:shadow-[0_0_20px_-5px_rgba(16,185,129,0.2)]",
      "focus:ring-emerald-500"
    ),
    ghost: cn(
      // Light mode
      "bg-transparent text-gray-600",
      "hover:bg-gray-100 hover:text-gray-900",
      // Dark mode
      "dark:text-gray-400",
      "dark:hover:bg-gray-800/50 dark:hover:text-white",
      "focus:ring-gray-500"
    ),
    danger: cn(
      "bg-gradient-to-r from-red-500 to-rose-500 text-white",
      "shadow-lg shadow-red-500/25",
      "hover:shadow-xl hover:shadow-red-500/30",
      "dark:shadow-[0_0_20px_-5px_rgba(239,68,68,0.5)]",
      "dark:hover:shadow-[0_0_30px_-5px_rgba(239,68,68,0.6)]",
      "focus:ring-red-500"
    ),
  };

  const sizeClasses = {
    sm: 'px-4 py-2 text-xs',
    md: 'px-6 py-3 text-sm',
    lg: 'px-8 py-4 text-base font-semibold',
  };

  const isDisabled = disabled || loading;

  return (
    <motion.button
      className={cn(
        baseClasses,
        variantClasses[variant],
        sizeClasses[size],
        fullWidth && 'w-full',
        className
      )}
      whileHover={isDisabled ? {} : { scale: 1.02 }}
      whileTap={isDisabled ? {} : { scale: 0.98 }}
      transition={{ duration: 0.15 }}
      disabled={isDisabled}
      aria-busy={loading}
      {...props}
    >
      {loading ? (
        <>
          <LoadingSpinner />
          <span>{children}</span>
        </>
      ) : (
        <>
          {leftIcon}
          <span>{children}</span>
          {rightIcon}
        </>
      )}
    </motion.button>
  );
}
