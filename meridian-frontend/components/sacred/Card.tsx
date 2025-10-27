'use client';

import { cn } from '@/lib/utils';

interface CardProps {
  children: React.ReactNode;
  className?: string;
  noPadding?: boolean;
  onClick?: () => void;
}

export const SacredCard = ({ children, className, noPadding = false, onClick }: CardProps) => {
  return (
    <div
      className={cn(
        'bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded',
        'transition-all duration-200',
        'shadow-sm dark:shadow-lg dark:shadow-black/20',
        !noPadding && 'p-6',
        onClick && 'cursor-pointer hover:border-gray-300 dark:hover:border-gray-600 hover-lift',
        !onClick && 'hover-lift',
        className
      )}
      onClick={onClick}
    >
      {children}
    </div>
  );
};
