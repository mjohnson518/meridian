'use client';

import { useState } from 'react';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { UserRole } from '@/lib/auth/types';
import { SacredCard } from '@/components/sacred/Card';
import { SacredGrid } from '@/components/sacred/Grid';
import { SacredButton } from '@/components/sacred/Button';
import { SacredTable } from '@/components/sacred/Table';
import { Heading, MonoDisplay, Label } from '@/components/sacred/Typography';
import { MetricDisplay } from '@/components/meridian/MetricDisplay';
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

interface SARReport {
  id: string;
  transactionId: string;
  filedDate: Date;
  status: 'DRAFT' | 'FILED' | 'ACKNOWLEDGED';
  reason: string;
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

function ComplianceDashboard() {
  const { user } = useAuth();
  const [transactions] = useState<Transaction[]>(MOCK_TRANSACTIONS);
  const [selectedTx, setSelectedTx] = useState<Transaction | null>(null);

  const flaggedCount = transactions.filter(tx => tx.status === 'FLAGGED').length;
  const highRiskCount = transactions.filter(tx => tx.riskScore > 70).length;
  const totalVolume24h = transactions
    .filter(tx => tx.timestamp > new Date(Date.now() - 86400000))
    .reduce((sum, tx) => sum + tx.amount, 0);

  const transactionColumns = [
    {
      header: 'TIME',
      accessor: (tx: Transaction) => (
        <span className="text-xs">{formatTimestamp(tx.timestamp)}</span>
      ),
    },
    {
      header: 'TYPE',
      accessor: (tx: Transaction) => (
        <span className={`text-xs font-medium ${
          tx.type === 'MINT' ? 'text-emerald-600' :
          tx.type === 'BURN' ? 'text-red-600' :
          'text-sacred-gray-700'
        }`}>{tx.type}</span>
      ),
    },
    {
      header: 'AMOUNT',
      accessor: (tx: Transaction) => (
        <MonoDisplay value={tx.amount} currency={tx.currency} precision={2} size="xs" />
      ),
      align: 'right' as const,
    },
    {
      header: 'FROM',
      accessor: (tx: Transaction) => (
        <span className="text-xs font-mono">{tx.from.slice(0, 10)}...</span>
      ),
    },
    {
      header: 'TO',
      accessor: (tx: Transaction) => (
        <span className="text-xs font-mono">{tx.to.slice(0, 10)}...</span>
      ),
    },
    {
      header: 'RISK',
      accessor: (tx: Transaction) => (
        <div className="flex items-center space-x-1">
          <div className={`w-2 h-2 rounded-full ${
            tx.riskScore < 30 ? 'bg-emerald-600' :
            tx.riskScore < 70 ? 'bg-amber-600' :
            'bg-red-600'
          }`} />
          <span className="text-xs">{tx.riskScore}</span>
        </div>
      ),
      align: 'center' as const,
    },
    {
      header: 'STATUS',
      accessor: (tx: Transaction) => (
        <span className={`text-xs font-mono ${
          tx.status === 'COMPLETED' ? 'text-emerald-600' :
          tx.status === 'FLAGGED' ? 'text-red-600' :
          'text-amber-600'
        }`}>{tx.status}</span>
      ),
    },
  ];

  return (
    <div className="min-h-screen bg-sacred-gray-100">
      {/* Header */}
      <header className="bg-sacred-white border-b border-sacred-gray-200">
        <div className="sacred-container">
          <nav className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-8">
              <a href="/portal/dashboard" className="font-mono text-lg font-medium">
                MERIDIAN
              </a>
              <div className="hidden md:flex items-center space-x-6">
                <a href="/portal/dashboard" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Dashboard
                </a>
                <a href="/portal/mint" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Mint/Burn
                </a>
                <a href="/portal/compliance" className="text-sm font-mono uppercase tracking-wider text-sacred-black">
                  Compliance
                </a>
                <a href="/portal/settings" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Settings
                </a>
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <div className="text-xs font-mono uppercase text-sacred-gray-600">
                {user?.organization}
              </div>
            </div>
          </nav>
        </div>
      </header>

      {/* Main Content */}
      <div className="sacred-container py-8">
        <div className="mb-8">
          <Heading level={1} className="text-3xl mb-2">
            Compliance Dashboard
          </Heading>
          <p className="text-sacred-gray-600">
            Transaction monitoring and AML/KYC oversight
          </p>
        </div>

        {/* Alert Metrics */}
        <SacredGrid cols={4} gap={4} className="mb-8">
          <SacredCard>
            <MetricDisplay
              label="Flagged Transactions"
              value={flaggedCount}
              format="number"
              status={flaggedCount > 0 ? 'warning' : 'healthy'}
            />
          </SacredCard>
          
          <SacredCard>
            <MetricDisplay
              label="High Risk Count"
              value={highRiskCount}
              format="number"
              status={highRiskCount > 0 ? 'critical' : 'healthy'}
            />
          </SacredCard>
          
          <SacredCard>
            <MetricDisplay
              label="24h Volume"
              value={totalVolume24h}
              format="currency"
            />
          </SacredCard>
          
          <SacredCard>
            <MetricDisplay
              label="SARs Filed (30d)"
              value={0}
              format="number"
            />
          </SacredCard>
        </SacredGrid>

        {/* Transaction Monitoring */}
        <SacredCard className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <Heading level={3} className="text-lg font-mono uppercase">
              Transaction Monitoring
            </Heading>
            <div className="flex items-center space-x-2">
              <select className="px-3 py-1 border border-sacred-gray-200 rounded font-mono text-xs">
                <option>All Transactions</option>
                <option>Flagged Only</option>
                <option>High Risk (&gt;70)</option>
                <option>Large Amounts (&gt;$10k)</option>
              </select>
              <SacredButton variant="outline" size="sm">
                Export CSV
              </SacredButton>
            </div>
          </div>

          <SacredTable
            columns={transactionColumns}
            data={transactions}
            dense
            onRowClick={setSelectedTx}
          />
        </SacredCard>

        {/* SAR Filing */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <SacredCard>
            <Heading level={3} className="text-lg font-mono uppercase mb-4">
              SAR Filing Queue
            </Heading>
            
            <div className="space-y-3">
              {flaggedCount > 0 ? (
                <div className="p-4 bg-amber-50 border border-amber-200 rounded">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-xs font-mono uppercase text-amber-800 mb-1">
                        Action Required
                      </div>
                      <p className="text-xs text-amber-700">
                        {flaggedCount} transaction(s) flagged for review
                      </p>
                    </div>
                    <SacredButton variant="primary" size="sm">
                      Review →
                    </SacredButton>
                  </div>
                </div>
              ) : (
                <div className="text-center py-8">
                  <div className="text-4xl mb-2">✓</div>
                  <p className="text-sm text-sacred-gray-500 font-mono">
                    No pending SAR filings
                  </p>
                </div>
              )}
            </div>
          </SacredCard>

          <SacredCard>
            <Heading level={3} className="text-lg font-mono uppercase mb-4">
              Blacklist Management
            </Heading>
            
            <div className="space-y-4">
              <div>
                <Label>Add Address to Blacklist</Label>
                <div className="flex gap-2 mt-2">
                  <input
                    type="text"
                    placeholder="0x..."
                    className="flex-1 px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm"
                  />
                  <SacredButton variant="primary" size="sm">
                    Add
                  </SacredButton>
                </div>
              </div>

              <div>
                <Label>Currently Blacklisted</Label>
                <div className="mt-2 p-3 bg-sacred-gray-100 rounded">
                  <p className="text-xs font-mono text-sacred-gray-600">
                    0 addresses blacklisted
                  </p>
                </div>
              </div>

              <div className="text-xs text-sacred-gray-600">
                <p className="mb-2">
                  <strong className="font-mono">Sanctions Screening:</strong>
                </p>
                <p>
                  All addresses are automatically screened against OFAC, EU, and UN sanctions lists via Chainalysis integration.
                </p>
              </div>
            </div>
          </SacredCard>
        </div>
      </div>
    </div>
  );
}

