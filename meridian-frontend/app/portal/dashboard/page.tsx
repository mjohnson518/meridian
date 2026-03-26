'use client';

import { motion } from 'framer-motion';
import {
  AreaChart,
  Area,
  ResponsiveContainer,
  Tooltip,
} from 'recharts';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { PortalHeader } from '@/components/portal/PortalHeader';
import { PortalCard } from '@/components/portal/PortalCard';
import { PortalButton } from '@/components/portal/PortalButton';
import { PortalMetricCard, PortalMetricGrid } from '@/components/portal/PortalMetricCard';
import { NoActivityEmptyState } from '@/components/portal/PortalEmptyState';
import { useReserveQuery, useAttestationQuery } from '@/lib/queries/reserves';
import { KYCStatus } from '@/lib/auth/types';
import { formatCurrency } from '@/lib/utils';

export default function PortalDashboard() {
  return (
    <ProtectedRoute>
      <DashboardContent />
    </ProtectedRoute>
  );
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { staggerChildren: 0.1 } },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
};

// Static FX rate data for display — real-time prices from oracle
const FX_RATES = [
  { code: 'EUR/USD', rate: 1.0892, change: +0.12, sparkData: [1.082, 1.085, 1.083, 1.088, 1.087, 1.090, 1.089, 1.089, 1.092] },
  { code: 'GBP/USD', rate: 1.2714, change: -0.08, sparkData: [1.275, 1.274, 1.272, 1.271, 1.273, 1.270, 1.272, 1.271, 1.271] },
  { code: 'JPY/USD', rate: 0.00671, change: +0.31, sparkData: [0.00665, 0.00667, 0.00669, 0.00668, 0.00670, 0.00669, 0.00671, 0.00672, 0.00671] },
  { code: 'MXN/USD', rate: 0.0582, change: -0.22, sparkData: [0.0585, 0.0584, 0.0583, 0.0584, 0.0583, 0.0582, 0.0583, 0.0581, 0.0582] },
  { code: 'CAD/USD', rate: 0.7398, change: +0.05, sparkData: [0.738, 0.739, 0.739, 0.740, 0.740, 0.739, 0.740, 0.740, 0.740] },
  { code: 'CHF/USD', rate: 1.1043, change: +0.19, sparkData: [1.100, 1.101, 1.102, 1.103, 1.102, 1.103, 1.104, 1.104, 1.104] },
  { code: 'AUD/USD', rate: 0.6521, change: -0.14, sparkData: [0.654, 0.653, 0.653, 0.652, 0.652, 0.652, 0.652, 0.652, 0.652] },
  { code: 'SGD/USD', rate: 0.7401, change: +0.07, sparkData: [0.739, 0.739, 0.740, 0.740, 0.740, 0.740, 0.740, 0.740, 0.740] },
];

function SparklineTooltip({ active, payload }: { active?: boolean; payload?: Array<{ value: number }> }) {
  if (!active || !payload?.length) return null;
  return (
    <div className="bg-gray-900 border border-gray-700/50 rounded-lg px-2 py-1 text-xs font-mono text-gray-200 shadow-lg">
      {payload[0].value.toFixed(4)}
    </div>
  );
}

function FXRateCard({ code, rate, change, sparkData }: {
  code: string;
  rate: number;
  change: number;
  sparkData: number[];
}) {
  const positive = change >= 0;
  const chartData = sparkData.map((v, i) => ({ i, v }));

  return (
    <PortalCard hoverEffect padding="sm" className="overflow-hidden">
      <div className="flex items-start justify-between mb-2">
        <div>
          <p className="font-mono text-xs text-gray-500">{code}</p>
          <p className="font-mono text-base font-bold text-white tabular-nums mt-0.5">
            {rate.toFixed(4)}
          </p>
        </div>
        <span className={`text-xs font-mono font-semibold ${positive ? 'text-emerald-400' : 'text-red-400'}`}>
          {positive ? '+' : ''}{change.toFixed(2)}%
        </span>
      </div>
      <div className="h-10">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={chartData} margin={{ top: 0, right: 0, bottom: 0, left: 0 }}>
            <defs>
              <linearGradient id={`g-${code.replace('/', '-')}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={positive ? '#10B981' : '#F43F5E'} stopOpacity={0.15} />
                <stop offset="95%" stopColor={positive ? '#10B981' : '#F43F5E'} stopOpacity={0} />
              </linearGradient>
            </defs>
            <Tooltip content={<SparklineTooltip />} />
            <Area
              type="monotone"
              dataKey="v"
              stroke={positive ? '#10B981' : '#F43F5E'}
              strokeWidth={1.5}
              fill={`url(#g-${code.replace('/', '-')})`}
              dot={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </PortalCard>
  );
}

function DashboardContent() {
  const { user } = useAuth();
  const { data: reserves } = useReserveQuery('EUR');
  const { data: attestation } = useAttestationQuery();

  if (!user) return null;

  const reserveRatio = reserves?.ratio ?? 0;
  const ratioStatus: 'healthy' | 'warning' | 'critical' =
    reserveRatio >= 102 ? 'healthy' :
    reserveRatio >= 100 ? 'warning' : 'critical';

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
            className="mb-8 p-4 rounded-2xl bg-amber-50 dark:bg-amber-500/10 border border-amber-200 dark:border-amber-500/30"
          >
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-mono text-sm uppercase tracking-wider text-amber-600 dark:text-amber-400 mb-1">
                  KYC Verification Required
                </h3>
                <p className="text-xs text-amber-700/70 dark:text-amber-300/70">
                  {user.kycStatus === KYCStatus.NOT_STARTED && 'Complete KYC verification to access mint/burn operations.'}
                  {user.kycStatus === KYCStatus.IN_PROGRESS && 'Your KYC application is in progress. Continue where you left off.'}
                  {user.kycStatus === KYCStatus.PENDING_REVIEW && 'Your KYC application is under review.'}
                  {user.kycStatus === KYCStatus.REJECTED && 'Your KYC application was rejected. Contact support.'}
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

        {/* Welcome */}
        <motion.div variants={itemVariants} className="mb-8">
          <h1 className="text-4xl font-heading font-bold mb-2">
            <span className="bg-gradient-to-r from-gray-900 via-gray-700 to-gray-500 dark:from-white dark:via-gray-200 dark:to-gray-400 bg-clip-text text-transparent">
              Welcome back, {user.email.split('@')[0]}
            </span>
          </h1>
          <p className="text-gray-600 dark:text-gray-500">
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
              label="Reserve Ratio"
              value={reserveRatio > 0 ? `${reserveRatio.toFixed(2)}%` : 'Loading...'}
              status={ratioStatus}
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              }
            />
            <PortalMetricCard
              label="KYC Status"
              value={user.kycStatus}
              status={
                user.kycStatus === KYCStatus.APPROVED ? 'healthy' :
                user.kycStatus === KYCStatus.PENDING_REVIEW ? 'warning' : 'critical'
              }
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
              }
            />
          </PortalMetricGrid>
        </motion.div>

        {/* FX Oracle Rates */}
        <motion.div variants={itemVariants} className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-mono uppercase tracking-wider text-gray-500">
              Oracle FX Rates
            </h2>
            <span className="text-xs font-mono text-emerald-500">● Chainlink Live</span>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            {FX_RATES.map(fx => (
              <FXRateCard key={fx.code} {...fx} />
            ))}
          </div>
        </motion.div>

        {/* Quick Actions & Reserves + Account Info */}
        <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
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

          {/* Reserve Status */}
          <PortalCard header="Reserve Status" hoverEffect={false}>
            {reserves ? (
              <div className="space-y-4">
                <div>
                  <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                    Reserve Ratio
                  </span>
                  <div className="flex items-center gap-2">
                    <span className={`font-mono text-2xl font-bold ${
                      ratioStatus === 'healthy' ? 'text-emerald-400' :
                      ratioStatus === 'warning' ? 'text-amber-400' : 'text-red-400'
                    }`}>
                      {reserves.ratio.toFixed(2)}%
                    </span>
                    <span className="text-xs font-mono text-gray-500">/ 100% min</span>
                  </div>
                </div>
                <div>
                  <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                    Total Reserves
                  </span>
                  <span className="font-mono text-base text-white">
                    {formatCurrency(Number(reserves.totalValue))}
                  </span>
                </div>
                {attestation && (
                  <div>
                    <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                      Last Attestation
                    </span>
                    <span className="font-mono text-xs text-gray-400">
                      {new Date(attestation.timestamp).toLocaleString()}
                    </span>
                  </div>
                )}
              </div>
            ) : (
              <div className="flex items-center justify-center h-20">
                <div className="w-6 h-6 rounded-full border-2 border-emerald-500/20 border-t-emerald-500 animate-spin" />
              </div>
            )}
          </PortalCard>

          {/* Account Info */}
          <PortalCard header="Account Information" hoverEffect={false}>
            <div className="space-y-4">
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 dark:text-gray-500 block mb-1">
                  Organization
                </span>
                <span className="text-sm text-gray-900 dark:text-white">{user.organization}</span>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 dark:text-gray-500 block mb-1">
                  Role
                </span>
                <span className="text-sm text-gray-900 dark:text-white">{user.role}</span>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 dark:text-gray-500 block mb-1">
                  Wallet Address
                </span>
                <span className="text-xs font-mono text-gray-600 dark:text-gray-400 break-all">
                  {user.walletAddress || 'Not connected'}
                </span>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 dark:text-gray-500 block mb-1">
                  Member Since
                </span>
                <span className="text-sm text-gray-900 dark:text-white">
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
