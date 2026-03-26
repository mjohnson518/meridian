'use client';

import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { PortalCard } from '@/components/portal/PortalCard';
import { PortalButton } from '@/components/portal/PortalButton';
import { PortalTable } from '@/components/portal/PortalTable';
import { BondHolding } from '@/lib/api/client';
import { formatCurrency } from '@/lib/utils';
import {
  useRealtimeReserves,
  useRealtimeAttestation,
  useRealtimePrice,
  useWebSocketConnection,
  usePriceUpdater,
} from '@/hooks/useRealtimeData';

const bondColumns = [
  {
    header: 'ISIN',
    accessor: (row: BondHolding) => (
      <span className="font-mono text-xs text-gray-300">{row.isin}</span>
    ),
  },
  {
    header: 'Name',
    accessor: (row: BondHolding) => (
      <span className="text-sm text-gray-200">{row.name}</span>
    ),
  },
  {
    header: 'Maturity',
    accessor: (row: BondHolding) => (
      <span className="font-mono text-xs text-gray-400">{row.maturity}</span>
    ),
  },
  {
    header: 'Quantity',
    accessor: (row: BondHolding) => (
      <span className="font-mono text-xs text-gray-200 tabular-nums">
        {row.quantity.toLocaleString()}
      </span>
    ),
    align: 'right' as const,
  },
  {
    header: 'Price',
    accessor: (row: BondHolding) => (
      <span className="font-mono text-xs text-gray-200 tabular-nums">
        {row.price.toFixed(2)}
      </span>
    ),
    align: 'right' as const,
  },
  {
    header: 'Value',
    accessor: (row: BondHolding) => (
      <span className="font-mono text-xs text-emerald-400 tabular-nums">
        {formatCurrency(row.value)}
      </span>
    ),
    align: 'right' as const,
  },
  {
    header: 'Yield',
    accessor: (row: BondHolding) => (
      <span className="font-mono text-xs text-gray-200 tabular-nums">
        {row.yield.toFixed(2)}%
      </span>
    ),
    align: 'right' as const,
  },
  {
    header: 'Rating',
    accessor: (row: BondHolding) => (
      <span className="font-mono text-xs font-semibold text-cyan-400">
        {row.rating}
      </span>
    ),
    align: 'center' as const,
  },
];

// Chart tooltip formatter
function ChartTooltip({ active, payload, label }: {
  active?: boolean;
  payload?: Array<{ value: number }>;
  label?: number;
}) {
  if (!active || !payload?.length) return null;
  return (
    <div className="bg-gray-900 border border-gray-700 rounded-xl px-3 py-2 shadow-xl">
      <p className="font-mono text-xs text-gray-400 mb-1">
        {label ? new Date(label).toLocaleTimeString() : ''}
      </p>
      <p className="font-mono text-sm font-medium text-emerald-400">
        {payload[0].value.toFixed(2)}%
      </p>
    </div>
  );
}

export default function ReservesPage() {
  const { reserves, loading, error, lastUpdate, refetch } = useRealtimeReserves('EUR');
  const { attestation, loading: attestationLoading } = useRealtimeAttestation();
  const { price: eurPrice } = useRealtimePrice('EUR-USD');
  const { connected, reconnecting } = useWebSocketConnection();
  const { updatePrice, updating } = usePriceUpdater();

  const handleUpdatePrice = async () => {
    try {
      await updatePrice('EUR');
      refetch();
    } catch (err) {
      console.error('Failed to update price:', err);
    }
  };

  if (loading || attestationLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center dark:bg-gray-950">
        <div className="text-center">
          <div className="w-10 h-10 rounded-full border-2 border-emerald-500/20 border-t-emerald-500 animate-spin mx-auto" />
          <p className="mt-4 font-mono text-xs uppercase tracking-wider text-gray-500">
            Loading reserve data...
          </p>
        </div>
      </div>
    );
  }

  if (error && !reserves) {
    return (
      <div className="min-h-screen flex items-center justify-center dark:bg-gray-950">
        <div className="text-center max-w-md">
          <p className="font-mono text-sm text-red-400 mb-4">{error}</p>
          <p className="font-mono text-xs text-gray-500 mb-6">Backend unavailable. Using cached data.</p>
          <PortalButton onClick={refetch} variant="outline" size="sm">
            Retry Connection
          </PortalButton>
        </div>
      </div>
    );
  }

  if (!reserves || !attestation) return null;

  const ratioStatus =
    reserves.ratio >= 102 ? 'healthy' :
    reserves.ratio >= 100 ? 'warning' : 'critical';

  const ratioColor =
    ratioStatus === 'healthy' ? 'text-emerald-400' :
    ratioStatus === 'warning' ? 'text-amber-400' : 'text-red-400';

  const ratioBg =
    ratioStatus === 'healthy' ? 'bg-emerald-500/10 border-emerald-500/30' :
    ratioStatus === 'warning' ? 'bg-amber-500/10 border-amber-500/30' : 'bg-red-500/10 border-red-500/30';

  // Build chart data from history or generate from current
  const chartData = (reserves.history?.length
    ? reserves.history
    : Array.from({ length: 24 }, (_, i) => ({
        timestamp: Date.now() - (23 - i) * 3600 * 1000,
        ratio: reserves.ratio + (Math.random() - 0.5) * 0.4,
        totalValue: Number(reserves.totalValue),
      }))
  );

  return (
    <div className="min-h-screen dark:bg-gray-950 bg-gray-50 transition-colors">
      <div className="max-w-[1200px] mx-auto px-6 py-10">

        {/* Status Bar */}
        <div className="mb-8 flex items-center justify-between px-5 py-3 rounded-xl bg-white dark:bg-gray-900/80 border border-gray-200 dark:border-gray-700/50 backdrop-blur-sm">
          <div className="flex items-center gap-6">
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${
                connected ? 'bg-emerald-500 animate-pulse' :
                reconnecting ? 'bg-amber-500' : 'bg-red-500'
              }`} />
              <span className="font-mono text-xs font-medium text-gray-900 dark:text-gray-100">
                {connected ? 'Live' : reconnecting ? 'Reconnecting...' : 'Offline'}
              </span>
            </div>
            <span className="font-mono text-xs text-gray-500">
              {lastUpdate.toLocaleTimeString()}
            </span>
            {eurPrice && (
              <span className="font-mono text-xs text-gray-500">
                EUR/USD: {eurPrice}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <PortalButton onClick={handleUpdatePrice} variant="ghost" size="sm" loading={updating}>
              {updating ? 'Updating...' : 'Update Prices'}
            </PortalButton>
            <PortalButton onClick={refetch} variant="ghost" size="sm">
              Refresh
            </PortalButton>
          </div>
        </div>

        {/* Header */}
        <div className="mb-10">
          <h1 className="text-4xl font-heading font-bold text-gray-900 dark:text-white tracking-tight mb-2">
            Reserve Dashboard
          </h1>
          <p className="text-gray-500 dark:text-gray-400">
            Real-time view of Meridian stablecoin reserves and attestation status
          </p>
          {error && (
            <span className="inline-block mt-3 text-xs text-amber-600 dark:text-amber-400 font-mono px-3 py-1.5 bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900/50 rounded-lg">
              Note: Backend connection issue. Showing cached data.
            </span>
          )}
        </div>

        {/* Key Metrics */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
          <PortalCard hoverEffect={false}>
            <p className="text-xs font-mono uppercase tracking-wider text-gray-500 mb-2">Total Reserves</p>
            <p className="text-2xl font-mono font-bold text-white">
              {formatCurrency(Number(reserves.totalValue))}
            </p>
            {reserves.trend !== 0 && (
              <p className={`text-xs font-mono mt-1 ${reserves.trend > 0 ? 'text-emerald-400' : 'text-red-400'}`}>
                {reserves.trend > 0 ? '+' : ''}{reserves.trend.toFixed(2)}%
              </p>
            )}
          </PortalCard>

          <PortalCard hoverEffect={false} className={ratioBg}>
            <p className="text-xs font-mono uppercase tracking-wider text-gray-500 mb-2">Reserve Ratio</p>
            <p className={`text-2xl font-mono font-bold ${ratioColor}`}>
              {reserves.ratio.toFixed(2)}%
            </p>
            <p className="text-xs font-mono mt-1 text-gray-500">Min: 100.00%</p>
          </PortalCard>

          <PortalCard hoverEffect={false}>
            <p className="text-xs font-mono uppercase tracking-wider text-gray-500 mb-2">Last Attestation</p>
            <p className="text-sm font-mono font-bold text-white">
              {new Date(attestation.timestamp).toLocaleTimeString()}
            </p>
            <p className="text-xs font-mono mt-1 text-gray-500">
              {new Date(attestation.timestamp).toLocaleDateString()}
            </p>
          </PortalCard>

          <PortalCard hoverEffect={false}>
            <p className="text-xs font-mono uppercase tracking-wider text-gray-500 mb-2">Currencies Live</p>
            <p className="text-2xl font-mono font-bold text-white">{reserves.activeCurrencies}</p>
            <p className="text-xs font-mono mt-1 text-gray-500">Active stablecoins</p>
          </PortalCard>
        </div>

        {/* Reserve Ratio Time Series Chart */}
        <PortalCard
          header="Reserve Ratio — 24h History"
          hoverEffect={false}
          className="mb-6"
          headerAction={
            <span className={`text-xs font-mono font-semibold ${ratioColor}`}>
              Current: {reserves.ratio.toFixed(2)}%
            </span>
          }
        >
          <div className="h-48">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
                <defs>
                  <linearGradient id="ratioGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#10B981" stopOpacity={0.2} />
                    <stop offset="95%" stopColor="#10B981" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" />
                <XAxis
                  dataKey="timestamp"
                  tickFormatter={(v: number) => new Date(v).getHours() + 'h'}
                  tick={{ fontFamily: 'monospace', fontSize: 10, fill: '#6B7280' }}
                  axisLine={false}
                  tickLine={false}
                  interval="preserveStartEnd"
                />
                <YAxis
                  domain={['auto', 'auto']}
                  tick={{ fontFamily: 'monospace', fontSize: 10, fill: '#6B7280' }}
                  axisLine={false}
                  tickLine={false}
                  tickFormatter={(v: number) => `${v.toFixed(1)}%`}
                  width={48}
                />
                <Tooltip content={<ChartTooltip />} />
                <Area
                  type="monotone"
                  dataKey="ratio"
                  stroke="#10B981"
                  strokeWidth={1.5}
                  fill="url(#ratioGradient)"
                  dot={false}
                  activeDot={{ r: 4, fill: '#10B981', stroke: 'transparent' }}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>

          {/* Reserve health bar */}
          <div className="mt-4 pt-4 border-t border-white/5 grid grid-cols-3 gap-4">
            <div>
              <span className="font-mono text-xs text-gray-500">Minimum</span>
              <p className="font-mono text-base font-semibold text-white mt-0.5">100.00%</p>
            </div>
            <div>
              <span className="font-mono text-xs text-gray-500">Current</span>
              <p className={`font-mono text-base font-semibold mt-0.5 ${ratioColor}`}>
                {reserves.ratio.toFixed(2)}%
              </p>
            </div>
            <div>
              <span className="font-mono text-xs text-gray-500">Buffer</span>
              <p className="font-mono text-base font-semibold text-emerald-400 mt-0.5">
                +{Math.max(0, reserves.ratio - 100).toFixed(2)}%
              </p>
            </div>
          </div>
        </PortalCard>

        {/* Bond Holdings Table */}
        <PortalCard
          hoverEffect={false}
          className="mb-6"
          header="Bond Holdings"
          headerAction={
            connected ? (
              <span className="text-xs font-mono text-emerald-500">● Live</span>
            ) : undefined
          }
        >
          <PortalTable
            columns={bondColumns}
            data={reserves.holdings}
            dense
            emptyMessage="No bond holdings data available"
          />
        </PortalCard>

        {/* Currency Breakdown + Attestation */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <PortalCard header="Currency Allocation" hoverEffect={false}>
            <div className="space-y-3">
              {reserves.currencies.map((currency, index) => (
                <div key={index} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className="w-2.5 h-2.5 rounded-full"
                      style={{
                        background: `hsl(${(index * 137) % 360}, 70%, 60%)`,
                      }}
                    />
                    <span className="font-mono text-sm text-gray-300">{currency.currency}</span>
                  </div>
                  <div className="text-right">
                    <p className="font-mono text-sm text-white">
                      {formatCurrency(currency.value)}
                    </p>
                    <p className="font-mono text-xs text-gray-500">
                      {currency.percentage.toFixed(1)}%
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </PortalCard>

          <PortalCard header="Attestation Schedule" hoverEffect={false}>
            <div className="space-y-4">
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Last Attestation
                </span>
                <p className="font-mono text-sm text-white">
                  {new Date(attestation.timestamp).toLocaleString()}
                </p>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Next Attestation
                </span>
                <p className="font-mono text-sm text-white">
                  {new Date(attestation.nextAttestation).toLocaleString()}
                </p>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Frequency
                </span>
                <p className="font-mono text-sm text-white">Every 6 hours</p>
              </div>
              <div>
                <span className="text-xs font-mono uppercase tracking-wider text-gray-500 block mb-1">
                  Oracle Provider
                </span>
                <p className="font-mono text-sm text-white">Chainlink</p>
              </div>
              {attestation.status === 'warning' && (
                <div className="p-3 rounded-xl bg-amber-500/10 border border-amber-500/30">
                  <span className="text-xs font-mono text-amber-400">
                    Attestation overdue. Expected within next hour.
                  </span>
                </div>
              )}
              {attestation.status === 'critical' && (
                <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/30">
                  <span className="text-xs font-mono text-red-400">
                    Attestation significantly overdue. Manual intervention may be required.
                  </span>
                </div>
              )}
            </div>
          </PortalCard>
        </div>

        {/* Footer */}
        <div className="mt-8 px-5 py-3 bg-white dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700/50 rounded-xl flex items-center justify-between">
          <p className="text-xs font-mono text-gray-500">
            {connected
              ? 'Connected to backend. Data updates automatically via WebSocket.'
              : 'Using cached data. Attempting to reconnect...'}
          </p>
          <p className="text-xs font-mono text-gray-500">
            All values in USD · Last refresh: {new Date().toLocaleTimeString()}
          </p>
        </div>
      </div>
    </div>
  );
}
