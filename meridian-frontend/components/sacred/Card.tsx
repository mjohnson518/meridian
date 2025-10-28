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
        'bg-white dark:bg-[#141416] border border-gray-200 dark:border-gray-800 rounded-xl',
        'transition-all duration-300',
        'shadow-sm hover:shadow-md dark:shadow-none',
        !noPadding && 'p-6',
        onClick && 'cursor-pointer hover:border-gray-300 dark:hover:border-gray-700',
        className
      )}
      onClick={onClick}
    >
      {children}
    </div>
  );
};
