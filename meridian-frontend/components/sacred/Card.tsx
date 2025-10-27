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
        'bg-sacred-white border border-sacred-gray-200 rounded',
        !noPadding && 'p-6',
        onClick && 'cursor-pointer hover:border-sacred-gray-300 transition-colors',
        className
      )}
      onClick={onClick}
    >
      {children}
    </div>
  );
};
