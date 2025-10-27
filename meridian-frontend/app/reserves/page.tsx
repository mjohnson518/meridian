'use client';

import { useEffect } from 'react';
import { SacredCard } from '@/components/sacred/Card';
import { SacredGrid } from '@/components/sacred/Grid';
import { SacredTable } from '@/components/sacred/Table';
import { SacredButton } from '@/components/sacred/Button';
import { Heading, MonoDisplay } from '@/components/sacred/Typography';
import { MetricDisplay } from '@/components/meridian/MetricDisplay';
import { ReserveRatioBar } from '@/components/meridian/ReserveRatioBar';
import { BondHolding } from '@/lib/api/client';
import { formatCurrency, formatPercentage, formatCompactNumber } from '@/lib/utils';
import { 
  useRealtimeReserves, 
  useRealtimeAttestation, 
  useRealtimePrice,
  useWebSocketConnection,
  usePriceUpdater
} from '@/hooks/useRealtimeData';

export default function ReservesPage() {
  const { reserves, loading, error, lastUpdate, refetch } = useRealtimeReserves('EUR');
  const { attestation, loading: attestationLoading, error: attestationError } = useRealtimeAttestation();
  const { price: eurPrice, timestamp: priceTimestamp } = useRealtimePrice('EUR-USD');
  const { connected, reconnecting } = useWebSocketConnection();
  const { updatePrice, updating } = usePriceUpdater();

  const bondColumns = [
    {
      header: 'ISIN',
      accessor: (row: BondHolding) => (
        <span className="font-mono text-xs">{row.isin}</span>
      ),
    },
    {
      header: 'Name',
      accessor: (row: BondHolding) => (
        <span className="text-sm">{row.name}</span>
      ),
    },
    {
      header: 'Maturity',
      accessor: (row: BondHolding) => (
        <span className="font-mono text-xs">{row.maturity}</span>
      ),
    },
    {
      header: 'Quantity',
      accessor: (row: BondHolding) => (
        <MonoDisplay value={row.quantity} precision={0} size="xs" />
      ),
      align: 'right' as const,
    },
    {
      header: 'Price',
      accessor: (row: BondHolding) => (
        <MonoDisplay value={row.price} precision={2} size="xs" />
      ),
      align: 'right' as const,
    },
    {
      header: 'Value',
      accessor: (row: BondHolding) => (
        <MonoDisplay value={row.value} currency="USD" precision={2} size="xs" />
      ),
      align: 'right' as const,
    },
    {
      header: 'Yield',
      accessor: (row: BondHolding) => (
        <MonoDisplay value={row.yield} suffix="%" precision={2} size="xs" />
      ),
      align: 'right' as const,
    },
    {
      header: 'Rating',
      accessor: (row: BondHolding) => (
        <span className="font-mono text-xs font-medium">{row.rating}</span>
      ),
      align: 'center' as const,
    },
  ];

  // Handle manual price update
  const handleUpdatePrice = async () => {
    try {
      await updatePrice('EUR');
      refetch(); // Refresh reserves after price update
    } catch (err) {
      console.error('Failed to update price:', err);
    }
  };

  if (loading || attestationLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-sacred-black"></div>
          <p className="mt-4 font-mono text-sm text-sacred-gray-600">
            Connecting to backend...
          </p>
        </div>
      </div>
    );
  }

  if (error && !reserves) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <p className="font-mono text-sm text-red-600 mb-4">Error: {error}</p>
          <p className="font-mono text-xs text-sacred-gray-600 mb-4">
            Using mock data. Backend may not be available.
          </p>
          <SacredButton onClick={refetch} variant="outline">
            Retry Connection
          </SacredButton>
        </div>
      </div>
    );
  }

  if (!reserves || !attestation) {
    return null;
  }

  return (
    <div className="sacred-container py-8">
      {/* Connection Status Bar */}
      <div className="mb-4 flex items-center justify-between p-2 bg-sacred-gray-100 rounded">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <div className={`w-2 h-2 rounded-full ${connected ? 'bg-emerald-600 animate-pulse' : reconnecting ? 'bg-amber-600' : 'bg-red-600'}`} />
            <span className="font-mono text-xs uppercase">
              {connected ? 'Live' : reconnecting ? 'Reconnecting...' : 'Offline'}
            </span>
          </div>
          <span className="font-mono text-xs text-sacred-gray-600">
            Last update: {lastUpdate.toLocaleTimeString()}
          </span>
          {eurPrice && (
            <span className="font-mono text-xs text-sacred-gray-600">
              EUR/USD: ${eurPrice}
            </span>
          )}
        </div>
        <div className="flex items-center space-x-2">
          <SacredButton
            onClick={handleUpdatePrice}
            size="sm"
            variant="outline"
            loading={updating}
            disabled={updating}
          >
            {updating ? 'Updating...' : 'Update Prices'}
          </SacredButton>
          <SacredButton onClick={refetch} size="sm" variant="outline">
            Refresh
          </SacredButton>
        </div>
      </div>

      {/* Header */}
      <div className="mb-8">
        <Heading level={1} className="text-3xl mb-2">
          Reserve Dashboard
        </Heading>
        <p className="text-sacred-gray-600">
          Real-time view of Meridian stablecoin reserves and attestation status
        </p>
        {error && (
          <p className="text-xs text-amber-600 font-mono mt-2">
            Note: Backend connection issue. Showing cached data.
          </p>
        )}
      </div>

      {/* Key Metrics */}
      <SacredGrid cols={4} gap={4} className="mb-8">
        <SacredCard>
          <MetricDisplay
            label="Total Reserves"
            value={reserves.totalValue}
            format="currency"
            trend={reserves.trend}
          />
        </SacredCard>
        
        <SacredCard>
          <MetricDisplay
            label="Reserve Ratio"
            value={reserves.ratio}
            format="percentage"
            threshold={100}
            status={reserves.ratio >= 100 ? 'healthy' : reserves.ratio >= 95 ? 'warning' : 'critical'}
          />
        </SacredCard>
        
        <SacredCard>
          <MetricDisplay
            label="Last Attestation"
            value={attestation.timestamp}
            format="timeago"
            status={attestation.status}
          />
        </SacredCard>
        
        <SacredCard>
          <MetricDisplay
            label="Currencies Live"
            value={reserves.activeCurrencies}
            format="number"
          />
        </SacredCard>
      </SacredGrid>

      {/* Reserve Ratio Bar */}
      <SacredCard className="mb-8">
        <div className="mb-4">
          <Heading level={3} className="text-lg font-mono uppercase">
            Reserve Health
          </Heading>
        </div>
        <ReserveRatioBar ratio={reserves.ratio} />
        <div className="mt-4 grid grid-cols-3 gap-4 text-sm">
          <div>
            <span className="font-mono text-xs uppercase text-sacred-gray-600">Minimum</span>
            <div className="font-mono text-lg">100%</div>
          </div>
          <div>
            <span className="font-mono text-xs uppercase text-sacred-gray-600">Current</span>
            <div className="font-mono text-lg">{reserves.ratio.toFixed(2)}%</div>
          </div>
          <div>
            <span className="font-mono text-xs uppercase text-sacred-gray-600">Buffer</span>
            <div className="font-mono text-lg text-emerald-600">
              +{Math.max(0, reserves.ratio - 100).toFixed(2)}%
            </div>
          </div>
        </div>
      </SacredCard>

      {/* Bond Holdings Table */}
      <SacredCard className="mb-8">
        <div className="mb-4 flex items-center justify-between">
          <Heading level={3} className="text-lg font-mono uppercase">
            Bond Holdings
          </Heading>
          {connected && (
            <span className="text-xs font-mono text-emerald-600">
              ● Real-time data
            </span>
          )}
        </div>
        <SacredTable
          columns={bondColumns}
          data={reserves.holdings}
          dense
        />
      </SacredCard>

      {/* Currency Breakdown & Attestation */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <SacredCard>
          <div className="mb-4">
            <Heading level={3} className="text-lg font-mono uppercase">
              Currency Allocation
            </Heading>
          </div>
          <div className="space-y-3">
            {reserves.currencies.map((currency, index) => (
              <div key={index} className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="w-3 h-3 rounded-full bg-sacred-black" />
                  <span className="font-mono text-sm">{currency.currency}</span>
                </div>
                <div className="text-right">
                  <div>
                    <MonoDisplay
                      value={currency.value}
                      currency="USD"
                      size="sm"
                    />
                  </div>
                  <div className="text-xs text-sacred-gray-500">
                    {currency.percentage.toFixed(1)}%
                  </div>
                </div>
              </div>
            ))}
          </div>
        </SacredCard>

        <SacredCard>
          <div className="mb-4">
            <Heading level={3} className="text-lg font-mono uppercase">
              Attestation Schedule
            </Heading>
          </div>
          <div className="space-y-4">
            <div>
              <span className="text-xs font-mono uppercase text-sacred-gray-600">
                Last Attestation
              </span>
              <div className="font-mono text-sm mt-1">
                {new Date(attestation.timestamp).toLocaleString()}
              </div>
            </div>
            <div>
              <span className="text-xs font-mono uppercase text-sacred-gray-600">
                Next Attestation
              </span>
              <div className="font-mono text-sm mt-1">
                {new Date(attestation.nextAttestation).toLocaleString()}
              </div>
            </div>
            <div>
              <span className="text-xs font-mono uppercase text-sacred-gray-600">
                Frequency
              </span>
              <div className="font-mono text-sm mt-1">
                Every 6 hours
              </div>
            </div>
            <div>
              <span className="text-xs font-mono uppercase text-sacred-gray-600">
                Oracle Provider
              </span>
              <div className="font-mono text-sm mt-1">
                Chainlink
              </div>
            </div>
            {attestation.status === 'warning' && (
              <div className="p-2 bg-amber-100 rounded">
                <span className="text-xs font-mono text-amber-800">
                  ⚠️ Attestation overdue. Expected within next hour.
                </span>
              </div>
            )}
            {attestation.status === 'critical' && (
              <div className="p-2 bg-red-100 rounded">
                <span className="text-xs font-mono text-red-800">
                  ⚠️ Attestation significantly overdue. Manual intervention may be required.
                </span>
              </div>
            )}
          </div>
        </SacredCard>
      </div>

      {/* Footer Info */}
      <div className="mt-8 p-4 bg-sacred-gray-100 rounded">
        <div className="flex items-center justify-between">
          <p className="text-xs font-mono text-sacred-gray-600">
            {connected 
              ? 'Connected to backend. Data updates automatically via WebSocket.'
              : 'Using cached data. Attempting to reconnect...'}
          </p>
          <p className="text-xs font-mono text-sacred-gray-600">
            All values in USD. Last refresh: {new Date().toLocaleTimeString()}
          </p>
        </div>
      </div>
    </div>
  );
}