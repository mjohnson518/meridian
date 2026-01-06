'use client';

import { motion, HTMLMotionProps } from 'framer-motion';
import { cn } from '@/lib/utils';
import { ReactNode } from 'react';

interface PortalCardBaseProps {
  children: ReactNode;
  className?: string;
  padding?: 'none' | 'sm' | 'md' | 'lg';
  header?: ReactNode;
  headerAction?: ReactNode;
}

interface PortalCardProps extends PortalCardBaseProps, Omit<HTMLMotionProps<'div'>, 'children'> {
  hoverEffect?: boolean;
}

const paddingClasses = {
  none: '',
  sm: 'p-4',
  md: 'p-6',
  lg: 'p-8',
};

export function PortalCard({
  children,
  className,
  hoverEffect = true,
  padding = 'md',
  header,
  headerAction,
  ...props
}: PortalCardProps) {
  return (
    <motion.div
      className={cn(
        "relative overflow-hidden rounded-2xl",
        "bg-white/[0.05] backdrop-blur-xl",
        "border border-white/20",
        "transition-all duration-300 ease-out",
        hoverEffect && [
          "hover:border-emerald-500/30",
          "hover:bg-white/[0.08]",
          "hover:shadow-[0_0_30px_-10px_rgba(16,185,129,0.2)]",
        ],
        className
      )}
      whileHover={hoverEffect ? { y: -2 } : undefined}
      transition={{ duration: 0.2 }}
      {...props}
    >
      {/* Subtle gradient overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-white/[0.02] to-transparent pointer-events-none" />

      {/* Content */}
      <div className="relative z-10">
        {header && (
          <div className={cn(
            "flex items-center justify-between",
            "border-b border-white/5",
            padding !== 'none' ? paddingClasses[padding] : 'p-6',
            "pb-4"
          )}>
            <h3 className="text-sm font-mono uppercase tracking-wider text-gray-300">
              {header}
            </h3>
            {headerAction}
          </div>
        )}
        <div className={cn(
          paddingClasses[padding],
          header && padding !== 'none' && 'pt-4'
        )}>
          {children}
        </div>
      </div>
    </motion.div>
  );
}

// Simple variant without motion for static cards
export function PortalCardStatic({
  children,
  className,
  padding = 'md',
  header,
  headerAction,
}: PortalCardBaseProps) {
  return (
    <div
      className={cn(
        "relative overflow-hidden rounded-2xl",
        "bg-white/[0.05] backdrop-blur-xl",
        "border border-white/20",
        className
      )}
    >
      {/* Subtle gradient overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-white/[0.02] to-transparent pointer-events-none" />

      {/* Content */}
      <div className="relative z-10">
        {header && (
          <div className={cn(
            "flex items-center justify-between",
            "border-b border-white/5",
            padding !== 'none' ? paddingClasses[padding] : 'p-6',
            "pb-4"
          )}>
            <h3 className="text-sm font-mono uppercase tracking-wider text-gray-300">
              {header}
            </h3>
            {headerAction}
          </div>
        )}
        <div className={cn(
          paddingClasses[padding],
          header && padding !== 'none' && 'pt-4'
        )}>
          {children}
        </div>
      </div>
    </div>
  );
}
