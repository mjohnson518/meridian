'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { PortalHeader } from '@/components/portal/PortalHeader';
import { PortalCard } from '@/components/portal/PortalCard';
import { PortalButton } from '@/components/portal/PortalButton';
import { PortalAmountInput } from '@/components/portal/PortalInput';
import { PortalMetricCard } from '@/components/portal/PortalMetricCard';
import { PortalStatusDot } from '@/components/portal/PortalStatusBadge';
import { NoTransactionsEmptyState } from '@/components/portal/PortalEmptyState';
import { formatCurrency, formatPercentage } from '@/lib/utils';
import { cn } from '@/lib/utils';

const SUPPORTED_CURRENCIES = [
  { code: 'EUR', name: 'Euro', rate: 1.09 },
  { code: 'GBP', name: 'British Pound', rate: 1.22 },
  { code: 'JPY', name: 'Japanese Yen', rate: 0.0067 },
  { code: 'MXN', name: 'Mexican Peso', rate: 0.058 },
];

const FEES = {
  issuance: 0.0025,
  redemption: 0.0025,
  custody: 0.001,
};

export default function MintPage() {
  return (
    <ProtectedRoute>
      <MintInterface />
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

function MintInterface() {
  const { user } = useAuth();
  const [mode, setMode] = useState<'mint' | 'burn'>('mint');
  const [amount, setAmount] = useState('');
  const [currency, setCurrency] = useState('EUR');
  const [loading, setLoading] = useState(false);
  const [notification, setNotification] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const selectedCurrency = SUPPORTED_CURRENCIES.find(c => c.code === currency);
  const numAmount = parseFloat(amount) || 0;

  const usdValue = numAmount / (selectedCurrency?.rate || 1);
  const fee = usdValue * FEES.issuance;
  const bondRequired = usdValue * 1.02;
  const totalCost = usdValue + fee;

  const handleExecute = async () => {
    setLoading(true);
    setNotification(null);

    try {
      const userId = user?.id ? parseInt(user.id) : 1;

      if (mode === 'mint') {
        await import('@/lib/api/realtime-client').then(m => m.realtimeApi.mint(currency, amount, userId));
      } else {
        await import('@/lib/api/realtime-client').then(m => m.realtimeApi.burn(currency, amount, userId));
      }

      setNotification({
        type: 'success',
        message: `${mode === 'mint' ? 'Mint' : 'Burn'} request submitted successfully! Settlement in T+1.`
      });
      setAmount('');
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : 'Please try again.';
      setNotification({
        type: 'error',
        message: `Operation failed: ${message}`
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen">
      <PortalHeader currentPath="/portal/mint" />

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
              Mint & Burn Operations
            </span>
          </h1>
          <p className="text-gray-500">
            Create or redeem multi-currency stablecoins
          </p>
        </motion.div>

        <div className="grid grid-cols-12 gap-6">
          {/* Main Panel */}
          <motion.div variants={itemVariants} className="col-span-12 lg:col-span-8">
            <PortalCard hoverEffect={false} padding="lg">
              {/* Notification Banner */}
              {notification && (
                <motion.div
                  initial={{ opacity: 0, y: -10 }}
                  animate={{ opacity: 1, y: 0 }}
                  className={cn(
                    "mb-6 p-4 rounded-xl border",
                    notification.type === 'success'
                      ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
                      : 'bg-red-500/10 border-red-500/30 text-red-400'
                  )}
                >
                  <div className="flex justify-between items-center">
                    <span className="font-mono text-sm">{notification.message}</span>
                    <button
                      onClick={() => setNotification(null)}
                      className="text-xs font-mono uppercase hover:opacity-70 transition-opacity"
                    >
                      Dismiss
                    </button>
                  </div>
                </motion.div>
              )}

              {/* Mode Toggle */}
              <div className="grid grid-cols-2 gap-4 mb-8">
                <button
                  onClick={() => setMode('mint')}
                  className={cn(
                    "relative flex items-center justify-center gap-3 px-6 py-4",
                    "font-mono text-sm uppercase tracking-wider",
                    "rounded-xl border transition-all duration-300",
                    mode === 'mint'
                      ? 'bg-emerald-500/10 border-emerald-500/50 text-emerald-400 shadow-[0_0_20px_-5px_rgba(16,185,129,0.3)]'
                      : 'bg-white/[0.02] border-white/10 text-gray-400 hover:border-emerald-500/30 hover:text-emerald-400'
                  )}
                >
                  <div className={cn(
                    "w-2 h-2 rounded-full transition-colors",
                    mode === 'mint' ? 'bg-emerald-400' : 'bg-gray-600'
                  )} />
                  Mint
                </button>
                <button
                  onClick={() => setMode('burn')}
                  className={cn(
                    "relative flex items-center justify-center gap-3 px-6 py-4",
                    "font-mono text-sm uppercase tracking-wider",
                    "rounded-xl border transition-all duration-300",
                    mode === 'burn'
                      ? 'bg-red-500/10 border-red-500/50 text-red-400 shadow-[0_0_20px_-5px_rgba(239,68,68,0.3)]'
                      : 'bg-white/[0.02] border-white/10 text-gray-400 hover:border-red-500/30 hover:text-red-400'
                  )}
                >
                  <div className={cn(
                    "w-2 h-2 rounded-full transition-colors",
                    mode === 'burn' ? 'bg-red-400' : 'bg-gray-600'
                  )} />
                  Burn
                </button>
              </div>

              {/* Amount Input */}
              <div className="space-y-6">
                <PortalAmountInput
                  label="Amount"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.00"
                  currency={currency}
                  onCurrencyChange={setCurrency}
                  currencies={SUPPORTED_CURRENCIES}
                />

                {/* Calculation Summary */}
                {numAmount > 0 && (
                  <motion.div
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: 'auto' }}
                    className="p-6 rounded-xl bg-white/[0.02] border border-white/5 space-y-3"
                  >
                    <h4 className="text-xs font-mono uppercase tracking-wider text-gray-500 mb-4">
                      {mode === 'mint' ? 'Bond Requirement' : 'Settlement Details'}
                    </h4>

                    <div className="flex justify-between font-mono text-sm">
                      <span className="text-gray-500">Amount in USD:</span>
                      <span className="text-white">{formatCurrency(usdValue)}</span>
                    </div>
                    <div className="flex justify-between font-mono text-sm">
                      <span className="text-gray-500">
                        {mode === 'mint' ? 'Issuance' : 'Redemption'} Fee (25 bps):
                      </span>
                      <span className="text-white">{formatCurrency(fee)}</span>
                    </div>

                    <div className="h-px bg-white/10 my-3" />

                    {mode === 'mint' ? (
                      <>
                        <div className="flex justify-between font-mono text-sm">
                          <span className="text-gray-500">Bonds Required:</span>
                          <span className="text-white">{formatCurrency(bondRequired)}</span>
                        </div>
                        <div className="flex justify-between font-mono text-sm font-medium">
                          <span className="text-white">Total Cost:</span>
                          <span className="text-emerald-400">{formatCurrency(totalCost)}</span>
                        </div>
                      </>
                    ) : (
                      <div className="flex justify-between font-mono text-sm font-medium">
                        <span className="text-white">You'll Receive:</span>
                        <span className="text-emerald-400">{formatCurrency(usdValue - fee)}</span>
                      </div>
                    )}
                  </motion.div>
                )}

                {/* Settlement Timeline */}
                {numAmount > 0 && mode === 'mint' && (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="p-4 rounded-xl border border-white/10"
                  >
                    <h4 className="text-xs font-mono uppercase tracking-wider text-gray-500 mb-4">
                      Settlement Timeline
                    </h4>
                    <div className="space-y-3">
                      {[
                        { time: 'T+0', desc: 'Deposit USD → Treasury purchases bonds' },
                        { time: 'T+1', desc: 'Bonds settle → Custody confirmed' },
                        { time: 'T+1', desc: 'Reserves attested → Stablecoins minted' },
                      ].map((step, i) => (
                        <div key={i} className="flex items-center gap-3 text-xs font-mono">
                          <span className="text-gray-500 w-8">{step.time}:</span>
                          <span className="text-gray-300">{step.desc}</span>
                        </div>
                      ))}
                    </div>
                  </motion.div>
                )}

                {/* Execute Button */}
                <PortalButton
                  onClick={handleExecute}
                  variant={mode === 'mint' ? 'primary' : 'danger'}
                  fullWidth
                  loading={loading}
                  disabled={!amount || numAmount <= 0 || loading}
                  className="py-4"
                >
                  {loading
                    ? `Processing ${mode}...`
                    : `Execute ${mode === 'mint' ? 'Mint' : 'Burn'}`}
                </PortalButton>

                {mode === 'mint' && numAmount > 0 && (
                  <p className="text-xs text-gray-600 text-center font-mono">
                    Minimum deposit: $100,000 USD equivalent
                  </p>
                )}
              </div>
            </PortalCard>
          </motion.div>

          {/* Side Panel */}
          <motion.div variants={itemVariants} className="col-span-12 lg:col-span-4 space-y-4">
            {/* Current Reserves */}
            <PortalCard header="Current Reserves" hoverEffect={false}>
              <div className="space-y-4">
                <div>
                  <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                    Reserve Ratio
                  </span>
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-2xl text-white">{formatPercentage(100.42)}</span>
                    <PortalStatusDot status="success" />
                  </div>
                </div>
                <div>
                  <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                    Available Capacity
                  </span>
                  <span className="font-mono text-lg text-white">{formatCurrency(42250)}</span>
                </div>
              </div>
            </PortalCard>

            {/* Gas Estimate */}
            <PortalCard header="Gas Estimate" hoverEffect={false}>
              <div className="space-y-2 font-mono text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-500">Network:</span>
                  <span className="text-white">Ethereum</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-500">Est. Gas:</span>
                  <span className="text-white">~150,000</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-500">Gas Price:</span>
                  <span className="text-white">20 gwei</span>
                </div>
                <div className="h-px bg-white/10 my-2" />
                <div className="flex justify-between font-medium">
                  <span className="text-white">Total:</span>
                  <span className="text-emerald-400">~$6.00</span>
                </div>
              </div>
            </PortalCard>

            {/* Exchange Rates */}
            <PortalCard header="Exchange Rates" hoverEffect={false}>
              <div className="space-y-2">
                {SUPPORTED_CURRENCIES.map(curr => (
                  <div key={curr.code} className="flex justify-between font-mono text-sm">
                    <span className="text-gray-500">{curr.code}/USD:</span>
                    <span className="text-white">{curr.rate.toFixed(4)}</span>
                  </div>
                ))}
              </div>
              <p className="text-xs text-gray-600 mt-4 font-mono">
                Last updated: {new Date().toLocaleTimeString()}
              </p>
            </PortalCard>

            {/* Compliance Check */}
            <PortalCard header="Compliance Check" hoverEffect={false}>
              <div className="space-y-2">
                {[
                  { label: 'KYC Verified', status: 'success' as const },
                  { label: 'Sanctions Clear', status: 'success' as const },
                  { label: 'Wallet Verified', status: 'success' as const },
                  { label: 'Daily Limit OK', status: 'success' as const },
                ].map((item) => (
                  <div key={item.label} className="flex items-center gap-2">
                    <PortalStatusDot status={item.status} size="sm" />
                    <span className="text-xs font-mono text-gray-300">{item.label}</span>
                  </div>
                ))}
              </div>
            </PortalCard>
          </motion.div>
        </div>

        {/* Recent Transactions */}
        <motion.div variants={itemVariants} className="mt-8">
          <PortalCard header="Recent Transactions" hoverEffect={false}>
            <NoTransactionsEmptyState />
          </PortalCard>
        </motion.div>
      </motion.div>
    </div>
  );
}
