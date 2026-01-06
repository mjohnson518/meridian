'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { UserRole } from '@/lib/auth/types';
import { PortalHeader } from '@/components/portal/PortalHeader';
import { PortalCard } from '@/components/portal/PortalCard';
import { PortalButton } from '@/components/portal/PortalButton';
import { PortalInput } from '@/components/portal/PortalInput';
import { PortalSelect } from '@/components/portal/PortalSelect';
import { PortalMetricCard, PortalMetricGrid } from '@/components/portal/PortalMetricCard';
import { PortalTable, PortalTableCard } from '@/components/portal/PortalTable';
import { PortalStatusBadge, PortalStatusDot } from '@/components/portal/PortalStatusBadge';
import { formatCurrency, formatTimestamp } from '@/lib/utils';

interface Transaction {
  id: string;
  timestamp: Date;
  type: 'MINT' | 'BURN' | 'TRANSFER';
  amount: number;
  currency: string;
  from: string;
  to: string;
  status: 'PENDING' | 'COMPLETED' | 'FLAGGED';
  riskScore: number;
}

const MOCK_TRANSACTIONS: Transaction[] = [
  {
    id: 'tx-001',
    timestamp: new Date(Date.now() - 3600000),
    type: 'MINT',
    amount: 1500000,
    currency: 'EUR',
    from: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    to: '0x89Ab...cDeF',
    status: 'COMPLETED',
    riskScore: 15,
  },
  {
    id: 'tx-002',
    timestamp: new Date(Date.now() - 7200000),
    type: 'MINT',
    amount: 250000,
    currency: 'GBP',
    from: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    to: '0x12Ab...34cD',
    status: 'COMPLETED',
    riskScore: 8,
  },
  {
    id: 'tx-003',
    timestamp: new Date(Date.now() - 10800000),
    type: 'TRANSFER',
    amount: 50000,
    currency: 'EUR',
    from: '0x89Ab...cDeF',
    to: '0xaB12...Cd34',
    status: 'FLAGGED',
    riskScore: 85,
  },
];

export default function CompliancePage() {
  return (
    <ProtectedRoute requiredRole={UserRole.COMPLIANCE}>
      <ComplianceDashboard />
    </ProtectedRoute>
  );
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.1 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
};

function ComplianceDashboard() {
  const { user } = useAuth();
  const [transactions] = useState<Transaction[]>(MOCK_TRANSACTIONS);
  const [selectedTx, setSelectedTx] = useState<Transaction | null>(null);
  const [filterType, setFilterType] = useState('all');

  const flaggedCount = transactions.filter(tx => tx.status === 'FLAGGED').length;
  const highRiskCount = transactions.filter(tx => tx.riskScore > 70).length;
  const totalVolume24h = transactions
    .filter(tx => tx.timestamp > new Date(Date.now() - 86400000))
    .reduce((sum, tx) => sum + tx.amount, 0);

  const transactionColumns = [
    {
      header: 'Time',
      accessor: (tx: Transaction) => (
        <span className="text-xs text-gray-400 font-mono">{formatTimestamp(tx.timestamp)}</span>
      ),
    },
    {
      header: 'Type',
      accessor: (tx: Transaction) => (
        <PortalStatusBadge
          status={tx.type === 'MINT' ? 'success' : tx.type === 'BURN' ? 'error' : 'neutral'}
          label={tx.type}
          size="sm"
        />
      ),
    },
    {
      header: 'Amount',
      accessor: (tx: Transaction) => (
        <span className="font-mono text-sm text-white">
          {formatCurrency(tx.amount)} <span className="text-gray-500">{tx.currency}</span>
        </span>
      ),
      align: 'right' as const,
    },
    {
      header: 'From',
      accessor: (tx: Transaction) => (
        <span className="text-xs font-mono text-gray-400">{tx.from.slice(0, 10)}...</span>
      ),
    },
    {
      header: 'To',
      accessor: (tx: Transaction) => (
        <span className="text-xs font-mono text-gray-400">{tx.to}</span>
      ),
    },
    {
      header: 'Risk',
      accessor: (tx: Transaction) => (
        <div className="flex items-center gap-2">
          <PortalStatusDot
            status={tx.riskScore < 30 ? 'success' : tx.riskScore < 70 ? 'warning' : 'error'}
            size="sm"
          />
          <span className="text-xs font-mono text-gray-300">{tx.riskScore}</span>
        </div>
      ),
      align: 'center' as const,
    },
    {
      header: 'Status',
      accessor: (tx: Transaction) => (
        <PortalStatusBadge
          status={tx.status === 'COMPLETED' ? 'success' : tx.status === 'FLAGGED' ? 'error' : 'pending'}
          label={tx.status}
          size="sm"
          pulse={tx.status === 'FLAGGED'}
        />
      ),
    },
  ];

  return (
    <div className="min-h-screen">
      <PortalHeader currentPath="/portal/compliance" />

      <motion.div
        className="max-w-7xl mx-auto px-6 py-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* Page Header */}
        <motion.div variants={itemVariants} className="mb-8">
          <h1 className="text-4xl font-heading font-bold mb-2">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Compliance Dashboard
            </span>
          </h1>
          <p className="text-gray-500">
            Transaction monitoring and AML/KYC oversight
          </p>
        </motion.div>

        {/* Alert Metrics */}
        <motion.div variants={itemVariants} className="mb-8">
          <PortalMetricGrid columns={4}>
            <PortalMetricCard
              label="Flagged Transactions"
              value={flaggedCount}
              format="number"
              status={flaggedCount > 0 ? 'warning' : 'healthy'}
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              }
            />
            <PortalMetricCard
              label="High Risk Count"
              value={highRiskCount}
              format="number"
              status={highRiskCount > 0 ? 'critical' : 'healthy'}
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              }
            />
            <PortalMetricCard
              label="24h Volume"
              value={totalVolume24h}
              format="currency"
              status="neutral"
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                </svg>
              }
            />
            <PortalMetricCard
              label="SARs Filed (30d)"
              value={0}
              format="number"
              status="healthy"
              icon={
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              }
            />
          </PortalMetricGrid>
        </motion.div>

        {/* Transaction Monitoring */}
        <motion.div variants={itemVariants} className="mb-8">
          <PortalTableCard
            title="Transaction Monitoring"
            action={
              <div className="flex items-center gap-3">
                <PortalSelect
                  options={[
                    { value: 'all', label: 'All Transactions' },
                    { value: 'flagged', label: 'Flagged Only' },
                    { value: 'high-risk', label: 'High Risk (>70)' },
                    { value: 'large', label: 'Large Amounts (>$10k)' },
                  ]}
                  value={filterType}
                  onChange={(e) => setFilterType(e.target.value)}
                  fullWidth={false}
                  className="w-48"
                />
                <PortalButton variant="outline" size="sm">
                  Export CSV
                </PortalButton>
              </div>
            }
            columns={transactionColumns}
            data={transactions}
            dense
            onRowClick={setSelectedTx}
          />
        </motion.div>

        {/* SAR Filing & Blacklist */}
        <motion.div variants={itemVariants} className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* SAR Filing Queue */}
          <PortalCard header="SAR Filing Queue" hoverEffect={false}>
            {flaggedCount > 0 ? (
              <div className="p-4 rounded-xl bg-amber-500/10 border border-amber-500/30">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-xs font-mono uppercase text-amber-400 mb-1">
                      Action Required
                    </div>
                    <p className="text-xs text-amber-300/70">
                      {flaggedCount} transaction(s) flagged for review
                    </p>
                  </div>
                  <PortalButton variant="primary" size="sm">
                    Review
                  </PortalButton>
                </div>
              </div>
            ) : (
              <div className="text-center py-8">
                <div className="text-4xl mb-3 text-emerald-500">✓</div>
                <p className="text-sm text-gray-500 font-mono">
                  No pending SAR filings
                </p>
              </div>
            )}
          </PortalCard>

          {/* Blacklist Management */}
          <PortalCard header="Blacklist Management" hoverEffect={false}>
            <div className="space-y-4">
              <div>
                <label className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-2">
                  Add Address to Blacklist
                </label>
                <div className="flex gap-2">
                  <PortalInput
                    placeholder="0x..."
                    className="flex-1"
                  />
                  <PortalButton variant="primary" size="sm">
                    Add
                  </PortalButton>
                </div>
              </div>

              <div>
                <label className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-2">
                  Currently Blacklisted
                </label>
                <div className="p-3 rounded-lg bg-white/[0.02] border border-white/5">
                  <p className="text-xs font-mono text-gray-500">
                    0 addresses blacklisted
                  </p>
                </div>
              </div>

              <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                <h4 className="text-xs font-mono uppercase text-gray-400 mb-2">
                  Sanctions Screening
                </h4>
                <p className="text-xs text-gray-500">
                  All addresses are automatically screened against OFAC, EU, and UN sanctions lists via Chainalysis integration.
                </p>
              </div>
            </div>
          </PortalCard>
        </motion.div>
      </motion.div>
    </div>
  );
}
