'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
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

export function PortalHeader({ currentPath }: PortalHeaderProps) {
  const { user, logout } = useAuth();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <header className="sticky top-0 z-50 bg-[#050608]/80 backdrop-blur-xl border-b border-white/5">
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
                        ? "text-white"
                        : "text-gray-400 hover:text-white"
                    )}
                  >
                    {link.label}
                    {isActive && (
                      <motion.div
                        layoutId="activeNav"
                        className="absolute inset-0 bg-white/5 rounded-lg -z-10"
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
            {/* User Info */}
            <div className="hidden sm:flex items-center space-x-3">
              <div className="text-right">
                <div className="text-xs font-mono uppercase text-gray-500">
                  {user?.role || 'User'}
                </div>
                <div className="text-sm text-gray-300">
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
              className="md:hidden p-2 text-gray-400 hover:text-white transition-colors"
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
              className="md:hidden border-t border-white/5 py-4"
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
                          ? "bg-white/5 text-white"
                          : "text-gray-400 hover:bg-white/5 hover:text-white"
                      )}
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      {link.label}
                    </a>
                  );
                })}
                <div className="pt-4 border-t border-white/5">
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
