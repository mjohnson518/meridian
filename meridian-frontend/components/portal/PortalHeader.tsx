'use client';

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTheme } from 'next-themes';
import { useAuth } from '@/lib/auth/AuthContext';
import { PortalButton } from './PortalButton';
import { cn } from '@/lib/utils';

interface NavLink {
  href: string;
  label: string;
  active?: boolean;
}

interface PortalHeaderProps {
  currentPath?: string;
}

const navLinks: NavLink[] = [
  { href: '/portal/dashboard', label: 'Dashboard' },
  { href: '/portal/mint', label: 'Mint/Burn' },
  { href: '/portal/compliance', label: 'Compliance' },
  { href: '/portal/settings', label: 'Settings' },
];

// Theme Toggle Button Component
function ThemeToggleButton() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return <div className="w-9 h-9 rounded-lg bg-gray-200 dark:bg-gray-800 animate-pulse" />;
  }

  const isDark = theme === 'dark';

  return (
    <button
      onClick={() => setTheme(isDark ? 'light' : 'dark')}
      className={cn(
        "w-9 h-9 flex items-center justify-center rounded-lg",
        "bg-gray-100 dark:bg-gray-800/80",
        "border border-gray-200 dark:border-gray-700",
        "hover:bg-gray-200 dark:hover:bg-gray-700",
        "transition-all duration-200"
      )}
      aria-label={`Switch to ${isDark ? 'light' : 'dark'} mode`}
      title={`Switch to ${isDark ? 'light' : 'dark'} mode`}
    >
      {isDark ? (
        <svg className="w-4 h-4 text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
      ) : (
        <svg className="w-4 h-4 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
        </svg>
      )}
    </button>
  );
}

export function PortalHeader({ currentPath }: PortalHeaderProps) {
  const { user, logout } = useAuth();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <header className="sticky top-0 z-50 bg-white/80 dark:bg-gray-950/80 backdrop-blur-xl border-b border-gray-200 dark:border-gray-800">
      <div className="max-w-7xl mx-auto px-6">
        <nav className="flex items-center justify-between h-16">
          {/* Logo */}
          <div className="flex items-center space-x-8">
            <a
              href="/portal/dashboard"
              className="font-heading text-xl font-bold bg-gradient-to-r from-emerald-400 to-teal-500 bg-clip-text text-transparent"
            >
              MERIDIAN
            </a>

            {/* Desktop Navigation */}
            <div className="hidden md:flex items-center space-x-1">
              {navLinks.map((link) => {
                const isActive = currentPath === link.href;
                return (
                  <a
                    key={link.href}
                    href={link.href}
                    className={cn(
                      "relative px-4 py-2 text-sm font-mono uppercase tracking-wider transition-colors duration-200",
                      isActive
                        ? "text-gray-900 dark:text-white"
                        : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                    )}
                  >
                    {link.label}
                    {isActive && (
                      <motion.div
                        layoutId="activeNav"
                        className="absolute inset-0 bg-gray-100 dark:bg-white/10 rounded-lg -z-10"
                        transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
                      />
                    )}
                  </a>
                );
              })}
            </div>
          </div>

          {/* User Section */}
          <div className="flex items-center space-x-4">
            {/* Theme Toggle */}
            <ThemeToggleButton />

            {/* User Info */}
            <div className="hidden sm:flex items-center space-x-3">
              <div className="text-right">
                <div className="text-xs font-mono uppercase text-gray-500 dark:text-gray-500">
                  {user?.role || 'User'}
                </div>
                <div className="text-sm text-gray-700 dark:text-gray-300">
                  {user?.organization || user?.email?.split('@')[0] || 'Organization'}
                </div>
              </div>
              {/* Avatar placeholder */}
              <div className="w-8 h-8 rounded-full bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center text-xs font-bold text-white">
                {user?.email?.[0]?.toUpperCase() || 'U'}
              </div>
            </div>

            {/* Sign Out Button */}
            <PortalButton
              variant="ghost"
              size="sm"
              onClick={logout}
              className="hidden sm:flex"
            >
              Sign Out
            </PortalButton>

            {/* Mobile Menu Button */}
            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="md:hidden p-2 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
              aria-label="Toggle menu"
            >
              <svg
                className="w-6 h-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                {mobileMenuOpen ? (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                ) : (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                )}
              </svg>
            </button>
          </div>
        </nav>

        {/* Mobile Menu */}
        <AnimatePresence>
          {mobileMenuOpen && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              transition={{ duration: 0.2 }}
              className="md:hidden border-t border-gray-200 dark:border-gray-800 py-4"
            >
              <div className="flex flex-col space-y-2">
                {navLinks.map((link) => {
                  const isActive = currentPath === link.href;
                  return (
                    <a
                      key={link.href}
                      href={link.href}
                      className={cn(
                        "px-4 py-3 text-sm font-mono uppercase tracking-wider rounded-lg transition-colors",
                        isActive
                          ? "bg-gray-100 dark:bg-white/10 text-gray-900 dark:text-white"
                          : "text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-white/5 hover:text-gray-900 dark:hover:text-white"
                      )}
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      {link.label}
                    </a>
                  );
                })}
                <div className="pt-4 border-t border-gray-200 dark:border-gray-800">
                  <PortalButton
                    variant="ghost"
                    size="sm"
                    onClick={logout}
                    fullWidth
                  >
                    Sign Out
                  </PortalButton>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </header>
  );
}
