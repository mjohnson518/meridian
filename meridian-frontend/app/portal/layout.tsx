'use client';

import { AuthProvider } from '@/lib/auth/AuthContext';
import '@/app/globals.css';

export default function PortalLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <AuthProvider>
      <div className="min-h-screen bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-white transition-colors duration-300">
        {/* Subtle grid background overlay - dark mode only */}
        <div
          className="fixed inset-0 pointer-events-none opacity-30 hidden dark:block"
          style={{
            backgroundImage: `
              linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px),
              linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px)
            `,
            backgroundSize: '24px 24px',
            maskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
            WebkitMaskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
          }}
        />
        {/* Light mode subtle pattern */}
        <div
          className="fixed inset-0 pointer-events-none opacity-50 dark:hidden"
          style={{
            backgroundImage: `
              linear-gradient(rgba(0,0,0,0.02) 1px, transparent 1px),
              linear-gradient(90deg, rgba(0,0,0,0.02) 1px, transparent 1px)
            `,
            backgroundSize: '24px 24px',
            maskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
            WebkitMaskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
          }}
        />
        {/* Content */}
        <div className="relative z-10">
          {children}
        </div>
      </div>
    </AuthProvider>
  );
}

