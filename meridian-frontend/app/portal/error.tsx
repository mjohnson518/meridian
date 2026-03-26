'use client';

import { useEffect } from 'react';
import { PortalButton } from '@/components/portal/PortalButton';

export default function PortalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error('[Portal Error]', error);
  }, [error]);

  return (
    <div className="min-h-screen flex items-center justify-center px-6">
      <div className="text-center max-w-md">
        <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-red-500/10 border border-red-500/30 mb-6">
          <svg className="w-8 h-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
          </svg>
        </div>
        <h2 className="text-xl font-heading font-bold text-white mb-2">
          Something went wrong
        </h2>
        <p className="text-sm text-gray-400 font-mono mb-2">
          {error.message || 'An unexpected error occurred'}
        </p>
        {error.digest && (
          <p className="text-xs text-gray-600 font-mono mb-6">
            Error ID: {error.digest}
          </p>
        )}
        <div className="flex gap-3 justify-center">
          <PortalButton onClick={reset} variant="primary" size="sm">
            Try again
          </PortalButton>
          <a href="/portal/dashboard">
            <PortalButton variant="outline" size="sm">
              Go to Dashboard
            </PortalButton>
          </a>
        </div>
      </div>
    </div>
  );
}
