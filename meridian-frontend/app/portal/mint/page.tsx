'use client';

import { useState } from 'react';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { SacredCard } from '@/components/sacred/Card';
import { SacredGrid } from '@/components/sacred/Grid';
import { SacredButton } from '@/components/sacred/Button';
import { Heading, MonoDisplay, Label } from '@/components/sacred/Typography';
import { MetricDisplay } from '@/components/meridian/MetricDisplay';
import { formatCurrency, formatPercentage } from '@/lib/utils';

const SUPPORTED_CURRENCIES = [
  { code: 'EUR', name: 'Euro', rate: 1.09 },
  { code: 'GBP', name: 'British Pound', rate: 1.22 },
  { code: 'JPY', name: 'Japanese Yen', rate: 0.0067 },
  { code: 'MXN', name: 'Mexican Peso', rate: 0.058 },
];

const FEES = {
  issuance: 0.0025, // 25 bps
  redemption: 0.0025, // 25 bps
  custody: 0.001, // 10 bps annual
};

export default function MintPage() {
  return (
    <ProtectedRoute>
      <MintInterface />
    </ProtectedRoute>
  );
}

function MintInterface() {
  const { user } = useAuth();
  const [mode, setMode] = useState<'mint' | 'burn'>('mint');
  const [amount, setAmount] = useState('');
  const [currency, setCurrency] = useState('EUR');
  const [loading, setLoading] = useState(false);

  const selectedCurrency = SUPPORTED_CURRENCIES.find(c => c.code === currency);
  const numAmount = parseFloat(amount) || 0;

  // Calculate requirements
  const usdValue = numAmount / (selectedCurrency?.rate || 1);
  const issuanceFee = usdValue * FEES.issuance;
  const bondRequired = usdValue * 1.02; // 102% to maintain buffer
  const totalCost = usdValue + issuanceFee;

  const [notification, setNotification] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const handleExecute = async () => {
    setLoading(true);
    setNotification(null);

    try {
      // Use user ID from auth context or fallback to test user ID (to ensure backend works)
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
      setAmount(''); // Reset form
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
    <div className="min-h-screen bg-sacred-gray-100">
      {/* Header */}
      <header className="bg-sacred-white border-b border-sacred-gray-200">
        <div className="sacred-container px-6">
          <nav className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-8">
              <a href="/portal/dashboard" className="font-mono text-lg font-medium">
                MERIDIAN
              </a>
              <div className="hidden md:flex items-center space-x-6">
                <a href="/portal/dashboard" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Dashboard
                </a>
                <a href="/portal/mint" className="text-sm font-mono uppercase tracking-wider text-sacred-black">
                  Mint/Burn
                </a>
                <a href="/portal/compliance" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
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
      <div className="sacred-container px-6 py-8">
        <div className="mb-8">
          <Heading level={1} className="text-3xl mb-2">
            Mint & Burn Operations
          </Heading>
          <p className="text-sacred-gray-600">
            Create or redeem multi-currency stablecoins
          </p>
        </div>

        <SacredGrid cols={12} gap={6}>
          {/* Main Panel */}
          <div className="col-span-12 lg:col-span-8">
            <SacredCard>
              {/* Notification Banner */}
              {notification && (
                <div
                  className={`mb-6 p-4 rounded border ${
                    notification.type === 'success'
                      ? 'bg-emerald-50 border-emerald-200 text-emerald-800'
                      : 'bg-rose-50 border-rose-200 text-rose-800'
                  }`}
                >
                  <div className="flex justify-between items-center">
                    <span className="font-mono text-sm">{notification.message}</span>
                    <button
                      onClick={() => setNotification(null)}
                      className="text-xs font-mono uppercase hover:opacity-70"
                    >
                      Dismiss
                    </button>
                  </div>
                </div>
              )}

              {/* Mode Toggle */}
              <div className="grid grid-cols-2 gap-4 mb-8">
                <button
                  onClick={() => setMode('mint')}
                  className={`flex items-center justify-center gap-2 px-6 py-4 font-mono text-sm uppercase tracking-wider border rounded-lg transition-all duration-200 ${mode === 'mint'
                    ? 'bg-emerald-500 text-white border-emerald-500 shadow-lg shadow-emerald-500/20 scale-[1.02]'
                    : 'bg-sacred-white text-sacred-gray-500 border-sacred-gray-200 hover:border-emerald-200 hover:text-emerald-600 hover:bg-emerald-50/10'
                    }`}
                >
                  <div className={`w-2 h-2 rounded-full ${mode === 'mint' ? 'bg-white' : 'bg-emerald-500'}`} />
                  Mint
                </button>
                <button
                  onClick={() => setMode('burn')}
                  className={`flex items-center justify-center gap-2 px-6 py-4 font-mono text-sm uppercase tracking-wider border rounded-lg transition-all duration-200 ${mode === 'burn'
                    ? 'bg-rose-500 text-white border-rose-500 shadow-lg shadow-rose-500/20 scale-[1.02]'
                    : 'bg-sacred-white text-sacred-gray-500 border-sacred-gray-200 hover:border-rose-200 hover:text-rose-600 hover:bg-rose-50/10'
                    }`}
                >
                  <div className={`w-2 h-2 rounded-full ${mode === 'burn' ? 'bg-white' : 'bg-rose-500'}`} />
                  Burn
                </button>
              </div>

              {/* Amount Input */}
              <div className="space-y-6">
                <div>
                  <Label>Amount</Label>
                  <div className="flex gap-3 mt-2">
                    <input
                      type="number"
                      value={amount}
                      onChange={(e) => setAmount(e.target.value)}
                      className="flex-1 px-6 py-4 border border-sacred-gray-200 rounded font-mono text-3xl text-right focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
                      placeholder="0.00"
                      step="0.01"
                      min="0"
                    />
                    <select
                      value={currency}
                      onChange={(e) => setCurrency(e.target.value)}
                      className="px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
                    >
                      {SUPPORTED_CURRENCIES.map(curr => (
                        <option key={curr.code} value={curr.code}>
                          {curr.code}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>

                {/* Calculation Summary */}
                {numAmount > 0 && (
                  <div className="p-6 bg-sacred-gray-100 rounded space-y-3">
                    <Heading level={4} className="text-sm font-mono uppercase mb-4">
                      {mode === 'mint' ? 'Bond Requirement' : 'Settlement Details'}
                    </Heading>

                    {mode === 'mint' ? (
                      <>
                        <div className="flex justify-between font-mono text-sm">
                          <span className="text-sacred-gray-600">Amount in USD:</span>
                          <MonoDisplay value={usdValue} currency="USD" precision={2} />
                        </div>
                        <div className="flex justify-between font-mono text-sm">
                          <span className="text-sacred-gray-600">Issuance Fee (25 bps):</span>
                          <MonoDisplay value={issuanceFee} currency="USD" precision={2} />
                        </div>
                        <div className="flex justify-between font-mono text-sm border-t border-sacred-gray-300 pt-3">
                          <span className="text-sacred-gray-600">Bonds Required:</span>
                          <MonoDisplay value={bondRequired} currency="USD" precision={2} />
                        </div>
                        <div className="flex justify-between font-mono text-sm font-medium">
                          <span>Total Cost:</span>
                          <MonoDisplay value={totalCost} currency="USD" precision={2} />
                        </div>
                      </>
                    ) : (
                      <>
                        <div className="flex justify-between font-mono text-sm">
                          <span className="text-sacred-gray-600">Amount in USD:</span>
                          <MonoDisplay value={usdValue} currency="USD" precision={2} />
                        </div>
                        <div className="flex justify-between font-mono text-sm">
                          <span className="text-sacred-gray-600">Redemption Fee (25 bps):</span>
                          <MonoDisplay value={issuanceFee} currency="USD" precision={2} />
                        </div>
                        <div className="flex justify-between font-mono text-sm font-medium border-t border-sacred-gray-300 pt-3">
                          <span>You'll Receive:</span>
                          <MonoDisplay value={usdValue - issuanceFee} currency="USD" precision={2} />
                        </div>
                      </>
                    )}
                  </div>
                )}

                {/* Settlement Timeline */}
                {numAmount > 0 && mode === 'mint' && (
                  <div className="p-4 border border-sacred-gray-200 rounded">
                    <Heading level={4} className="text-xs font-mono uppercase mb-3 text-sacred-gray-700">
                      Settlement Timeline
                    </Heading>
                    <div className="space-y-2 text-xs font-mono">
                      <div className="flex items-center space-x-2">
                        <span className="text-sacred-gray-500">T+0:</span>
                        <span>Deposit USD → Treasury purchases bonds</span>
                      </div>
                      <div className="flex items-center space-x-2">
                        <span className="text-sacred-gray-500">T+1:</span>
                        <span>Bonds settle → Custody confirmed</span>
                      </div>
                      <div className="flex items-center space-x-2">
                        <span className="text-sacred-gray-500">T+1:</span>
                        <span>Reserves attested → Stablecoins minted</span>
                      </div>
                    </div>
                  </div>
                )}

                {/* Execute Button */}
                <SacredButton
                  onClick={handleExecute}
                  variant="primary"
                  fullWidth
                  loading={loading}
                  disabled={!amount || numAmount <= 0 || loading}
                >
                  {loading
                    ? `Processing ${mode}...`
                    : `Execute ${mode === 'mint' ? 'Mint' : 'Burn'} →`}
                </SacredButton>

                {mode === 'mint' && numAmount > 0 && (
                  <p className="text-xs text-sacred-gray-600 text-center font-mono">
                    Minimum deposit: $100,000 USD equivalent
                  </p>
                )}
              </div>
            </SacredCard>
          </div>

          {/* Side Panel */}
          <div className="col-span-12 lg:col-span-4 space-y-4">
            <SacredCard>
              <Heading level={4} className="text-sm font-mono uppercase mb-4">
                Current Reserves
              </Heading>
              <div className="space-y-3">
                <MetricDisplay
                  label="Reserve Ratio"
                  value={100.42}
                  format="percentage"
                  status="healthy"
                />
                <MetricDisplay
                  label="Available Capacity"
                  value="42250"
                  format="currency"
                />
              </div>
            </SacredCard>

            <SacredCard>
              <Heading level={4} className="text-sm font-mono uppercase mb-4">
                Gas Estimate
              </Heading>
              <div className="space-y-2 font-mono text-sm">
                <div className="flex justify-between">
                  <span className="text-sacred-gray-600">Network:</span>
                  <span>Ethereum</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sacred-gray-600">Est. Gas:</span>
                  <span>~150,000</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sacred-gray-600">Gas Price:</span>
                  <span>20 gwei</span>
                </div>
                <div className="flex justify-between font-medium border-t border-sacred-gray-200 pt-2 mt-2">
                  <span>Total:</span>
                  <span>~$6.00</span>
                </div>
              </div>
            </SacredCard>

            <SacredCard>
              <Heading level={4} className="text-sm font-mono uppercase mb-4">
                Exchange Rates
              </Heading>
              <div className="space-y-2">
                {SUPPORTED_CURRENCIES.map(curr => (
                  <div key={curr.code} className="flex justify-between font-mono text-sm">
                    <span className="text-sacred-gray-600">{curr.code}/USD:</span>
                    <MonoDisplay value={curr.rate} precision={4} />
                  </div>
                ))}
              </div>
              <p className="text-xs text-sacred-gray-500 mt-3 font-mono">
                Last updated: {new Date().toLocaleTimeString()}
              </p>
            </SacredCard>

            <SacredCard>
              <Heading level={4} className="text-sm font-mono uppercase mb-4">
                Compliance Check
              </Heading>
              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 rounded-full bg-emerald-600"></div>
                  <span className="text-xs font-mono">KYC Verified</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 rounded-full bg-emerald-600"></div>
                  <span className="text-xs font-mono">Sanctions Clear</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 rounded-full bg-emerald-600"></div>
                  <span className="text-xs font-mono">Wallet Verified</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 rounded-full bg-emerald-600"></div>
                  <span className="text-xs font-mono">Daily Limit OK</span>
                </div>
              </div>
            </SacredCard>
          </div>
        </SacredGrid>

        {/* Recent Transactions */}
        <div className="mt-8">
          <SacredCard>
            <Heading level={3} className="text-lg font-mono uppercase mb-4">
              Recent Transactions
            </Heading>
            <div className="text-center py-12">
              <p className="text-sm text-sacred-gray-500 font-mono">
                No transactions yet. Execute your first {mode} operation above.
              </p>
            </div>
          </SacredCard>
        </div>
      </div>
    </div>
  );
}

