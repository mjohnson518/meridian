'use client';

import { motion } from 'framer-motion';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { PortalHeader } from '@/components/portal/PortalHeader';
import { PortalCard } from '@/components/portal/PortalCard';
import { PortalButton } from '@/components/portal/PortalButton';
import { PortalMetricCard, PortalMetricGrid } from '@/components/portal/PortalMetricCard';
import { NoActivityEmptyState } from '@/components/portal/PortalEmptyState';
import { KYCStatus } from '@/lib/auth/types';

export default function PortalDashboard() {
  return (
    <ProtectedRoute>
      <DashboardContent />
    </ProtectedRoute>
  );
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
};

function DashboardContent() {
  const { user, logout } = useAuth();

  if (!user) return null;

  return (
    <div className="min-h-screen">
      <PortalHeader currentPath="/portal/dashboard" />

      <motion.div
        className="max-w-7xl mx-auto px-6 py-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* KYC Status Banner */}
        {user.kycStatus !== KYCStatus.APPROVED && (
          <motion.div
            variants={itemVariants}
            className="mb-8 p-4 rounded-2xl bg-amber-500/10 border border-amber-500/30"
          >
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-mono text-sm uppercase tracking-wider text-amber-400 mb-1">
                  KYC Verification Required
                </h3>
                <p className="text-xs text-amber-300/70">
                  {user.kycStatus === KYCStatus.NOT_STARTED &&
                    'Complete KYC verification to access mint/burn operations.'}
                  {user.kycStatus === KYCStatus.IN_PROGRESS &&
                    'Your KYC application is in progress. Continue where you left off.'}
                  {user.kycStatus === KYCStatus.PENDING_REVIEW &&
                    'Your KYC application is under review. We will notify you when approved.'}
                  {user.kycStatus === KYCStatus.REJECTED &&
                    'Your KYC application was rejected. Contact support for details.'}
                </p>
              </div>
              {(user.kycStatus === KYCStatus.NOT_STARTED || user.kycStatus === KYCStatus.IN_PROGRESS) && (
                <a href="/portal/onboarding">
                  <PortalButton variant="primary" size="sm">
                    {user.kycStatus === KYCStatus.NOT_STARTED ? 'Start KYC' : 'Continue KYC'}
                  </PortalButton>
                </a>
              )}
            </div>
          </motion.div>
        )}

        {/* Welcome Section */}
        <motion.div variants={itemVariants} className="mb-8">
          <h1 className="text-4xl font-heading font-bold mb-2">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Welcome back, {user.email.split('@')[0]}
            </span>
          </h1>
          <p className="text-gray-500">
            Manage your multi-currency stablecoin operations
          </p>
        </motion.div>

        {/* Key Metrics */}
        <motion.div variants={itemVariants} className="mb-8">
          <PortalMetricGrid columns={4}>
            <PortalMetricCard
              label="Total Deposited"
              value={0}
              format="currency"
              status="neutral"
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              }
            />
            <PortalMetricCard
              label="Total Minted"
              value={0}
              format="currency"
              status="neutral"
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
              }
            />
            <PortalMetricCard
              label="Active Currencies"
              value={0}
              format="number"
              status="neutral"
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              }
            />
            <PortalMetricCard
              label="KYC Status"
              value={user.kycStatus}
              status={
                user.kycStatus === KYCStatus.APPROVED ? 'healthy' :
                  user.kycStatus === KYCStatus.PENDING_REVIEW ? 'warning' :
                    'critical'
              }
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
              }
            />
          </PortalMetricGrid>
        </motion.div>

        {/* Quick Actions & Account Info */}
        <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
          {/* Quick Actions */}
          <PortalCard header="Quick Actions" hoverEffect={false}>
            <div className="space-y-3">
              <a href="/portal/mint" className="block">
                <PortalButton
                  variant={user.kycStatus === KYCStatus.APPROVED ? 'primary' : 'secondary'}
                  fullWidth
                  disabled={user.kycStatus !== KYCStatus.APPROVED}
                  rightIcon={
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                    </svg>
                  }
                >
                  Mint Stablecoins
                </PortalButton>
              </a>
              <a href="/portal/mint" className="block">
                <PortalButton
                  variant="outline"
                  fullWidth
                  disabled={user.kycStatus !== KYCStatus.APPROVED}
                  rightIcon={
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                    </svg>
                  }
                >
                  Burn Stablecoins
                </PortalButton>
              </a>
              <a href="/reserves" className="block">
                <PortalButton
                  variant="ghost"
                  fullWidth
                  rightIcon={
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                    </svg>
                  }
                >
                  View Public Reserves
                </PortalButton>
              </a>
            </div>
          </PortalCard>

          {/* Account Info */}
          <PortalCard header="Account Information" hoverEffect={false}>
            <div className="space-y-4">
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Organization
                </span>
                <span className="text-sm text-white">{user.organization}</span>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Role
                </span>
                <span className="text-sm text-white">{user.role}</span>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Wallet Address
                </span>
                <span className="text-xs font-mono text-gray-400 break-all">
                  {user.walletAddress || 'Not connected'}
                </span>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Member Since
                </span>
                <span className="text-sm text-white">
                  {new Date(user.createdAt).toLocaleDateString()}
                </span>
              </div>
            </div>
          </PortalCard>
        </motion.div>

        {/* Recent Activity */}
        <motion.div variants={itemVariants}>
          <PortalCard header="Recent Activity" hoverEffect={false}>
            <NoActivityEmptyState />
          </PortalCard>
        </motion.div>
      </motion.div>
    </div>
  );
}
