'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';
import { PortalButton } from './PortalButton';

interface PortalEmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: {
    label: string;
    href?: string;
    onClick?: () => void;
  };
  className?: string;
}

export function PortalEmptyState({
  icon,
  title,
  description,
  action,
  className,
}: PortalEmptyStateProps) {
  return (
    <div className={cn("text-center py-12 px-4", className)}>
      {icon && (
        <div className="flex justify-center mb-4 text-gray-600">
          {icon}
        </div>
      )}
      <h3 className="text-sm font-mono uppercase tracking-wider text-gray-400 mb-2">
        {title}
      </h3>
      {description && (
        <p className="text-xs text-gray-500 max-w-sm mx-auto mb-6">
          {description}
        </p>
      )}
      {action && (
        <div>
          {action.href ? (
            <a href={action.href}>
              <PortalButton variant="secondary" size="sm">
                {action.label}
              </PortalButton>
            </a>
          ) : (
            <PortalButton variant="secondary" size="sm" onClick={action.onClick}>
              {action.label}
            </PortalButton>
          )}
        </div>
      )}
    </div>
  );
}

// Common preset empty states
export function NoActivityEmptyState({ className }: { className?: string }) {
  return (
    <PortalEmptyState
      icon={
        <svg className="w-12 h-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
      }
      title="No recent activity"
      description="Start by minting your first stablecoins to see activity here."
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
    <PortalEmptyState
      icon={
        <svg className="w-12 h-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
        </svg>
      }
      title="No transactions yet"
      description="Your transaction history will appear here."
      className={className}
    />
  );
}

export function NoDataEmptyState({ message, className }: { message?: string; className?: string }) {
  return (
    <PortalEmptyState
      icon={
        <svg className="w-12 h-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
        </svg>
      }
      title={message || "No data available"}
      className={className}
    />
  );
}
