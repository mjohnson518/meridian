'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';
import { Button } from './Button';

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick?: () => void;
    href?: string;
  };
  className?: string;
  variant?: 'default' | 'sacred';
}

export function EmptyState({
  icon,
  title,
  description,
  action,
  className,
  variant = 'default',
}: EmptyStateProps) {
  const containerClasses = {
    default: 'text-center py-12 px-4',
    sacred: 'text-center py-12 px-4',
  };

  const titleClasses = {
    default: 'text-lg font-semibold text-gray-900 dark:text-white mb-2',
    sacred: 'text-sm font-mono uppercase tracking-wider text-sacred-gray-800 mb-2',
  };

  const descriptionClasses = {
    default: 'text-sm text-gray-500 dark:text-gray-400 max-w-sm mx-auto',
    sacred: 'text-xs font-mono text-sacred-gray-500 max-w-sm mx-auto',
  };

  const iconClasses = {
    default: 'text-gray-400 dark:text-gray-500 mb-4',
    sacred: 'text-sacred-gray-400 mb-4',
  };

  return (
    <div className={cn(containerClasses[variant], className)}>
      {icon && (
        <div className={cn('flex justify-center', iconClasses[variant])}>
          {icon}
        </div>
      )}
      <h3 className={titleClasses[variant]}>{title}</h3>
      {description && (
        <p className={descriptionClasses[variant]}>{description}</p>
      )}
      {action && (
        <div className="mt-6">
          {action.href ? (
            <a href={action.href}>
              <Button variant="secondary" size="sm">
                {action.label}
              </Button>
            </a>
          ) : (
            <Button variant="secondary" size="sm" onClick={action.onClick}>
              {action.label}
            </Button>
          )}
        </div>
      )}
    </div>
  );
}

// Sacred design system variant
interface SacredEmptyStateProps {
  title: string;
  description?: string;
  action?: {
    label: string;
    href?: string;
    onClick?: () => void;
  };
  className?: string;
}

export function SacredEmptyState({
  title,
  description,
  action,
  className,
}: SacredEmptyStateProps) {
  return (
    <div className={cn('text-center py-12', className)}>
      <p className="text-sm text-sacred-gray-500 font-mono mb-2">{title}</p>
      {description && (
        <p className="text-xs text-sacred-gray-400 font-mono">{description}</p>
      )}
      {action && (
        <div className="mt-4">
          {action.href ? (
            <a
              href={action.href}
              className="inline-flex items-center px-4 py-2 text-xs font-mono uppercase tracking-wider border border-sacred-gray-300 rounded hover:bg-sacred-gray-50 transition-colors"
            >
              {action.label}
            </a>
          ) : (
            <button
              onClick={action.onClick}
              className="inline-flex items-center px-4 py-2 text-xs font-mono uppercase tracking-wider border border-sacred-gray-300 rounded hover:bg-sacred-gray-50 transition-colors"
            >
              {action.label}
            </button>
          )}
        </div>
      )}
    </div>
  );
}

// Common empty state presets
export function NoDataEmptyState({ className }: { className?: string }) {
  return (
    <EmptyState
      icon={
        <svg className="w-12 h-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
        </svg>
      }
      title="No data available"
      description="There's nothing to display here yet."
      className={className}
    />
  );
}

export function NoActivityEmptyState({ className }: { className?: string }) {
  return (
    <SacredEmptyState
      title="No recent activity"
      description="Start by minting your first stablecoins."
      action={{
        label: 'Mint Stablecoins',
        href: '/portal/mint',
      }}
      className={className}
    />
  );
}

export function NoTransactionsEmptyState({ className }: { className?: string }) {
  return (
    <SacredEmptyState
      title="No transactions yet"
      description="Your transaction history will appear here."
      className={className}
    />
  );
}
