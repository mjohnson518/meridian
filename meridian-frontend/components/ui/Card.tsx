'use client';

import { motion } from 'framer-motion';
import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface CardProps {
  children: ReactNode;
  className?: string;
  hover?: boolean;
  padding?: 'sm' | 'md' | 'lg' | 'xl';
}

export function Card({ children, className, hover = true, padding = 'lg' }: CardProps) {
  const paddingClasses = {
    sm: 'p-4 sm:p-5',
    md: 'p-6 sm:p-8',
    lg: 'p-8 sm:p-10',
    xl: 'p-10 sm:p-12',
  };

  return (
    <motion.div
      className={cn(
        'bg-white dark:bg-black border border-gray-200 dark:border-gray-800 rounded-xl shadow-card',
        paddingClasses[padding],
        hover && 'hover-lift cursor-pointer',
        className
      )}
      whileHover={hover ? { y: -2 } : undefined}
      transition={{ duration: 0.2, ease: [0.4, 0, 0.2, 1] }}
    >
      {children}
    </motion.div>
  );
}
